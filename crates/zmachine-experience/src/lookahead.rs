//! Look-ahead: an unsettled way tried on throwaway copies of the game
//! (specs/009-compass-lookahead/contracts/lookahead.md, cited L§;
//! research.md R2 to R7).
//!
//! The game being played is only ever read and cloned here. A copy is given
//! the text a touch would type, run to its next stop, asked where its player
//! is, and dropped; nothing it printed is read.

use zmachine_core::{Event, Machine};
use zmachine_introspect::{Introspector, Tri};

/// How many copies a way is tried on (research R2).
pub const COPIES: u32 = 3;

/// How far copies may run, counted in instructions so that the same prompt
/// always gives the same compass (research R7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// The most one copy may run; past it, its outcome is nothing (L§18).
    pub copy: u64,
    /// The most all copies at one prompt may run; a way not begun when it is
    /// spent has no finding (L§19).
    pub total: u64,
}

/// The limits a session uses, set from a measurement of the two walkthroughs
/// the project holds (`tests/speed.rs`, `what_look_ahead_runs_over_the_
/// walkthroughs`; specs/009-compass-lookahead/tasks.md T018). The longest
/// single copy seen ran 27 thousand instructions, and the busiest prompt 207
/// thousand in all. One copy may run twenty times the first; the total is the
/// same figure, which holds the busiest prompt two and a half times over and
/// keeps a prompt with a runaway copy to about two such runs.
pub const LIMITS: Limits = Limits {
    copy: 540_000,
    total: 540_000,
};

/// Limits under which nothing is tried: every way is left as the interface
/// reported it.
pub const OFF: Limits = Limits { copy: 0, total: 0 };

/// Instructions a copy runs between checks on its limit. What a copy ran is
/// counted in whole steps.
const STEP: u64 = 1_000;

/// What one copy did (data-model "Outcome").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Back at the command prompt, in this room, which is not the room it
    /// began in.
    Walked(u16),
    /// Back at the command prompt, in the room it began in.
    Stayed,
    /// Anything else.
    Nothing,
}

/// What look-ahead found for one way (data-model "Finding").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    /// Every copy walked to this room.
    Walked {
        room: u16,
        /// The room's name, as the first copy's `location` gives it.
        name: Option<String>,
        /// Whether that copy's `location` says the room is not lit.
        dark: bool,
    },
    /// Every copy stayed where it was.
    Refused,
    /// The copies differed, or one came to nothing, or the limit was spent.
    None,
}

/// The value for chance given to copy `copy` (1 or 2) at a prompt reached
/// after `sent` commands (L§5). It depends on nothing else, and the values
/// for the two copies differ.
pub fn chance(sent: u64, copy: u32) -> u32 {
    // A mixing of all the bits, so that the two copies' draws are as
    // unrelated to each other as to the game's own.
    let mut mixed = (sent as u32) ^ ((sent >> 32) as u32) ^ copy.wrapping_mul(0x9E37_79B9);
    mixed ^= mixed >> 16;
    mixed = mixed.wrapping_mul(0x85EB_CA6B);
    mixed ^= mixed >> 13;
    mixed = mixed.wrapping_mul(0xC2B2_AE35);
    mixed ^= mixed >> 16;
    // The copy's number in the low bits, so that two copies differ; and kept
    // clear of the small values a game uses for its predictable mode.
    ((mixed & 0x7FFF_FFFC) | (copy & 3)) | 0x1000
}

/// Look-ahead at one command prompt.
pub struct LookAhead<'a> {
    machine: &'a Machine,
    introspector: &'a Introspector,
    command_address: u32,
    sent: u64,
    limits: Limits,
    spent: u64,
    most: u64,
}

impl<'a> LookAhead<'a> {
    /// `machine` must be waiting at its command prompt, which is at
    /// `command_address`; `sent` is how many commands the game has been given.
    pub fn new(
        machine: &'a Machine,
        introspector: &'a Introspector,
        command_address: u32,
        sent: u64,
        limits: Limits,
    ) -> LookAhead<'a> {
        LookAhead {
            machine,
            introspector,
            command_address,
            sent,
            limits,
            spent: 0,
            most: 0,
        }
    }

    /// Instructions all copies have run so far, in whole steps.
    pub fn spent(&self) -> u64 {
        self.spent
    }

    /// The most instructions any one copy has run, in whole steps.
    pub fn most(&self) -> u64 {
        self.most
    }

    fn room(&self, machine: &Machine) -> Option<u16> {
        self.introspector
            .location(&machine.view())
            .value
            .ok()
            .map(|l| l.room)
    }

    /// One copy given `command`, with the game's own chance or another
    /// (L§4, 6). Returns what it did, and the copy.
    pub fn copy(&mut self, command: &str, chance: Option<u32>) -> (Outcome, Machine) {
        let before = self.room(self.machine);
        let mut copy = self.machine.clone();
        if let Some(value) = chance {
            copy.supply_chance(value);
        }
        if copy.provide_line(command).is_err() {
            return (Outcome::Nothing, copy);
        }
        let mut ran = 0;
        let outcome = loop {
            let event = copy.run(STEP);
            ran += STEP;
            match event {
                Ok(Event::BudgetExhausted) if ran < self.limits.copy => continue,
                Ok(Event::LineWanted { .. }) if copy.waiting_at() == Some(self.command_address) => {
                    break match (before, self.room(&copy)) {
                        (Some(before), Some(after)) if before == after => Outcome::Stayed,
                        (Some(_), Some(after)) => Outcome::Walked(after),
                        _ => Outcome::Nothing,
                    };
                }
                // A question, a key, a save, a restore, a transcript, the end,
                // a fault, or the limit: the copy is given nothing more.
                _ => break Outcome::Nothing,
            }
        };
        self.spent += ran;
        self.most = self.most.max(ran);
        (outcome, copy)
    }

    /// What the copies find for the way that sends `command` (L§5, 7, 19).
    pub fn finding(&mut self, command: &str) -> Finding {
        if self.spent >= self.limits.total {
            return Finding::None;
        }
        let (first, arrived) = self.copy(command, None);
        if first == Outcome::Nothing {
            return Finding::None;
        }
        for copy in 1..COPIES {
            let (other, _) = self.copy(command, Some(chance(self.sent, copy)));
            if other != first {
                return Finding::None;
            }
        }
        match first {
            Outcome::Walked(room) => {
                let location = self.introspector.location(&arrived.view()).value.ok();
                Finding::Walked {
                    room,
                    name: location.as_ref().and_then(|l| l.name.clone().ok()),
                    dark: location.is_some_and(|l| l.lit == Tri::No),
                }
            }
            Outcome::Stayed => Finding::Refused,
            Outcome::Nothing => Finding::None,
        }
    }
}
