//! One direction tried on a copy of a game
//! (specs/005-corpus-testing/research.md R1, R2).

use zmachine_core::{Event, Machine};
use zmachine_introspect::Introspector;

/// The most instructions a copy may run before it is counted as a fault.
pub const LIMIT: u64 = 50_000_000;

/// What a copy did when given a direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Outcome {
    /// At a command prompt, in this room, which is not the room it started in.
    Moved(u16),
    /// At a command prompt, in the same room.
    Stayed,
    /// Waiting for something that is not a command.
    Question,
    /// The game quit.
    Ended,
    /// The machine faulted, or ran past the limit without stopping.
    Fault,
    /// At a command prompt, but the location is unknown before or after.
    Lost,
}

/// The room the player is in, as the introspection interface reports it.
pub fn room(machine: &Machine, introspector: &Introspector) -> Option<u16> {
    let view = machine.view();
    introspector.location(&view).value.ok().map(|l| l.room)
}

/// Tries `word` on a copy of `machine`, which must be waiting at a command
/// prompt, and returns what the copy did, and the copy. The copy's output is
/// thrown away unread: only where the player is counts. `machine` itself is not
/// touched.
pub fn trial(
    machine: &Machine,
    introspector: &Introspector,
    command_address: u32,
    word: &str,
    limit: u64,
) -> (Outcome, Machine) {
    let before = room(machine, introspector);
    let mut copy = machine.clone();
    if copy.provide_line(word).is_err() {
        return (Outcome::Fault, copy);
    }
    let event = copy.run(limit);
    copy.take_output();
    copy.take_transcript();
    let outcome = match event {
        Ok(Event::LineWanted { .. }) if copy.waiting_at() == Some(command_address) => {
            match (before, room(&copy, introspector)) {
                (Some(before), Some(after)) if before == after => Outcome::Stayed,
                (Some(_), Some(after)) => Outcome::Moved(after),
                _ => Outcome::Lost,
            }
        }
        Ok(
            Event::LineWanted { .. }
            | Event::KeyWanted
            | Event::SaveRequested(_)
            | Event::RestoreRequested
            | Event::TranscriptRequested,
        ) => Outcome::Question,
        Ok(Event::Finished) => Outcome::Ended,
        Ok(Event::Fault(_) | Event::BudgetExhausted) | Err(_) => Outcome::Fault,
    };
    (outcome, copy)
}
