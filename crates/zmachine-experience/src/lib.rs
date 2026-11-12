//! The experience layer: what a phone's screen shows of a game, and what a tap
//! sends it.
//!
//! Everything a shell draws is decided here, so that the iPhone and Android apps
//! share one tested implementation of behaviour and contain presentation only.
//! The contract is `specs/004-first-demo/contracts/experience-api.md`.
#![forbid(unsafe_code)]

mod compass;
mod json;
mod strip;
mod transcript;

#[cfg(test)]
mod compass_tests;
#[cfg(test)]
mod json_tests;

pub use compass::{Compass, CompassPlace, Extra, NOTE_NONE, NOTE_UNKNOWN, Place, PlaceState};
pub use transcript::{Source, Turn};

use zmachine_core::{Event, HostError, Key, LoadOptions, Machine, StatusKind, StatusLine};
use zmachine_introspect::Introspector;

/// Where a game's randomness comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seed {
    /// A repeatable game: 0 to 2,147,483,647.
    Fixed(u32),
    /// A number from the host's random source. The layer has none of its own.
    Fresh(u32),
}

/// The game and turn a command was made against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stamp {
    pub generation: u64,
    pub turn_count: usize,
}

/// Why a command did not reach the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Refused {
    /// It was made against a turn or a game that is no longer the current one.
    Stale,
    /// The game has ended or faulted.
    NotWaiting,
    /// The compass does not offer that place now.
    NotOffered,
    /// The line holds a line break.
    BadLine,
}

/// Why a game could not be started.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartError {
    pub message: String,
}

/// What the game wants next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Waiting {
    /// A command, at the game's command prompt.
    Command,
    /// Some other line of text: the answer to a question.
    Answer,
    /// Nothing: the game has ended.
    Ended,
    /// Nothing: the story file failed.
    Fault,
}

impl Waiting {
    pub fn as_str(self) -> &'static str {
        match self {
            Waiting::Command => "command",
            Waiting::Answer => "answer",
            Waiting::Ended => "ended",
            Waiting::Fault => "fault",
        }
    }
}

/// The game's status line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub place: String,
    pub score: Option<i16>,
    pub moves: Option<u16>,
    pub hours: Option<u16>,
    pub minutes: Option<u16>,
}

/// How many instructions a game may run without stopping before the layer gives
/// it up as not responding.
pub const INSTRUCTION_CEILING: u64 = 50_000_000;

/// How many instructions to run between checks on the ceiling.
const SLICE: u64 = 1_000_000;

/// The largest fixed seed the core accepts.
const MAX_SEED: u32 = 2_147_483_647;

/// One game being played.
#[derive(Clone, Debug)]
pub struct Session {
    /// The story file, kept for starting a new game.
    story: Vec<u8>,
    machine: Machine,
    introspector: Introspector,
    turns: Vec<Turn>,
    generation: u64,
    waiting: Waiting,
    /// The game wants a single key, not a line.
    wants_key: bool,
    /// Where the game first asked for a line: its command prompt (research R6).
    command_address: Option<u32>,
    message: Option<String>,
    status: Option<Status>,
    compass: Compass,
    ceiling: u64,
}

fn load(story: &[u8], seed: Seed) -> Result<Machine, StartError> {
    let options = match seed {
        Seed::Fixed(seed) if seed > MAX_SEED => {
            return Err(StartError {
                message: format!("a fixed seed must be 0 to {MAX_SEED}, not {seed}"),
            });
        }
        Seed::Fixed(seed) => LoadOptions {
            seed: Some(seed),
            entropy: 0,
        },
        Seed::Fresh(entropy) => LoadOptions {
            seed: None,
            entropy,
        },
    };
    Machine::load(story.to_vec(), options).map_err(|e| StartError {
        message: format!("The story file is {e}."),
    })
}

impl Session {
    /// Loads the story file and runs it to its first stop.
    pub fn new(story: Vec<u8>, seed: Seed) -> Result<Session, StartError> {
        Session::with_pack(story, seed, None)
    }

    /// As [`Session::new`], with the text of a game pack for the story
    /// (docs/introspection.md, section 12). A pack that cannot be read, or is
    /// another story's, changes nothing.
    pub fn with_pack(
        story: Vec<u8>,
        seed: Seed,
        pack: Option<&str>,
    ) -> Result<Session, StartError> {
        let machine = load(&story, seed)?;
        // The interface is made once for the story file, so a pack in force
        // stays in force through every new game.
        let introspector = match pack {
            Some(text) => Introspector::with_pack(&machine.view(), text),
            None => Introspector::new(&machine.view()),
        };
        let mut session = Session {
            story,
            machine,
            introspector,
            turns: Vec::new(),
            generation: 1,
            waiting: Waiting::Command,
            wants_key: false,
            command_address: None,
            message: None,
            status: None,
            compass: Compass::idle(),
            ceiling: INSTRUCTION_CEILING,
        };
        session.open();
        Ok(session)
    }

    /// Lowers or raises the instruction ceiling. A host has no need to; tests do.
    pub fn set_instruction_ceiling(&mut self, instructions: u64) {
        self.ceiling = instructions;
    }

    /// Runs a freshly loaded game to its first stop and records the opening.
    fn open(&mut self) {
        let text = self.advance();
        self.turns = vec![Turn {
            number: 0,
            command: None,
            source: None,
            text,
        }];
    }

    /// Runs the game until it wants input, ends or fails, and returns what it
    /// printed. Requests to save, restore or keep a transcript are declined here
    /// and are not stops (research R7).
    fn advance(&mut self) -> String {
        let mut text = String::new();
        let mut run = 0u64;
        self.wants_key = false;
        loop {
            let event = match self.machine.run(SLICE) {
                Ok(event) => event,
                // The layer answers every event in turn, so this cannot happen;
                // if it does, the game cannot go on.
                Err(_) => {
                    self.fail("The game cannot continue.".to_string());
                    break;
                }
            };
            text.push_str(&self.machine.take_output());
            self.machine.take_transcript();
            self.machine.take_diagnostics();
            if let Some(line) = self.machine.take_status() {
                self.status = Some(status_of(line));
            }
            // An answer the layer gives is one the core's interface offers every
            // host; none can be refused here, since each matches its event.
            let _ = match event {
                Event::BudgetExhausted => {
                    run += SLICE;
                    if run >= self.ceiling {
                        self.fail("The game is not responding.".to_string());
                        break;
                    }
                    continue;
                }
                Event::SaveRequested(_) => self.machine.save_result(false),
                Event::RestoreRequested => self.machine.provide_restore(None),
                Event::TranscriptRequested => self.machine.transcript_result(false),
                Event::LineWanted { .. } => {
                    let here = self.machine.waiting_at();
                    let command = *self.command_address.get_or_insert(here.unwrap_or(0));
                    self.waiting = if here == Some(command) {
                        Waiting::Command
                    } else {
                        Waiting::Answer
                    };
                    break;
                }
                Event::KeyWanted => {
                    self.wants_key = true;
                    self.waiting = Waiting::Answer;
                    break;
                }
                Event::Finished => {
                    self.waiting = Waiting::Ended;
                    break;
                }
                Event::Fault(fault) => {
                    self.fail(format!("The story file stopped: {fault}."));
                    break;
                }
            };
            run = 0;
        }
        self.compass = match self.waiting {
            Waiting::Command => {
                let view = self.machine.view();
                // A version 3 dictionary keeps six characters of a word; later
                // versions nine (Standard §13.3, §13.4).
                let word_len = if view.version() <= 3 { 6 } else { 9 };
                Compass::of(&self.introspector.exits(&view), word_len)
            }
            _ => Compass::idle(),
        };
        text
    }

    fn fail(&mut self, message: String) {
        self.waiting = Waiting::Fault;
        self.message = Some(message);
    }

    /// The one way a line reaches the game (constitution Principle III).
    fn give(&mut self, line: &str, source: Source) {
        let given = if self.wants_key {
            // The game wants one key: the first character, or return.
            let key = line.chars().next().map_or(Key::Return, Key::Char);
            match self.machine.provide_key(key) {
                Err(HostError::UnrepresentableKey) => self.machine.provide_key(Key::Char('?')),
                other => other,
            }
        } else {
            self.machine.provide_line(line)
        };
        if given.is_err() {
            self.fail("The game cannot continue.".to_string());
            return;
        }
        let text = self.advance();
        self.turns.push(Turn {
            number: self.turns.len(),
            command: Some(line.to_string()),
            source: Some(source),
            text,
        });
    }

    /// Checks that a command was made against the current turn of a game that is
    /// waiting.
    fn check(&self, at: Stamp) -> Result<(), Refused> {
        if at != self.stamp() {
            return Err(Refused::Stale);
        }
        match self.waiting {
            Waiting::Command | Waiting::Answer => Ok(()),
            Waiting::Ended | Waiting::Fault => Err(Refused::NotWaiting),
        }
    }

    /// Gives the game a line the player typed.
    pub fn send_line(&mut self, at: Stamp, line: &str) -> Result<(), Refused> {
        self.check(at)?;
        if line.contains(['\n', '\r']) {
            return Err(Refused::BadLine);
        }
        self.give(line, Source::Typed);
        Ok(())
    }

    /// The player tapped a place on the compass.
    pub fn tap(&mut self, at: Stamp, place: Place) -> Result<(), Refused> {
        self.check(at)?;
        let command = self
            .compass
            .place(place)
            .and_then(|p| p.command.clone())
            .ok_or(Refused::NotOffered)?;
        self.give(&command, Source::Tap);
        Ok(())
    }

    /// The player tapped an extra, by its index.
    pub fn tap_extra(&mut self, at: Stamp, index: usize) -> Result<(), Refused> {
        self.check(at)?;
        let command = self
            .compass
            .extras
            .get(index)
            .map(|e| e.command.clone())
            .ok_or(Refused::NotOffered)?;
        self.give(&command, Source::Tap);
        Ok(())
    }

    /// Abandons the game and starts a new one from the same story file.
    pub fn new_game(&mut self, seed: Seed) {
        self.generation += 1;
        self.message = None;
        self.status = None;
        self.command_address = None;
        match load(&self.story, seed) {
            Ok(machine) => {
                self.machine = machine;
                self.open();
            }
            // The story file loaded before; only a bad seed can bring this.
            Err(error) => {
                self.turns = vec![Turn {
                    number: 0,
                    command: None,
                    source: None,
                    text: String::new(),
                }];
                self.compass = Compass::idle();
                self.fail(error.message);
            }
        }
    }

    /// Marks the session as broken: it reports a fault with this message,
    /// offers nothing and refuses commands until a new game. For a caller that
    /// could not let a call into the session finish.
    pub fn mark_failed(&mut self, message: &str) {
        self.compass = Compass::idle();
        self.fail(message.to_string());
    }

    pub fn stamp(&self) -> Stamp {
        Stamp {
            generation: self.generation,
            turn_count: self.turns.len(),
        }
    }

    pub fn waiting(&self) -> Waiting {
        self.waiting
    }

    /// The message for the player when the story file has failed.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn compass(&self) -> &Compass {
        &self.compass
    }

    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }

    /// Everything the screen needs, with the turns from `since` onward, as JSON.
    pub fn snapshot_json(&self, since: usize) -> String {
        json::snapshot(self, since)
    }
}

fn status_of(line: StatusLine) -> Status {
    let (score, moves, hours, minutes) = match line.kind {
        StatusKind::Score { score, turns } => (Some(score), Some(turns), None, None),
        StatusKind::Time { hours, minutes } => (None, None, Some(hours), Some(minutes)),
    };
    Status {
        place: line.location,
        score,
        moves,
        hours,
        minutes,
    }
}
