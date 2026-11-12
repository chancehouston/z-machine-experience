//! The experience layer: what a phone's screen shows of a game, and what a tap
//! sends it.
//!
//! Everything a shell draws is decided here, so that the iPhone and Android apps
//! share one tested implementation of behaviour and contain presentation only.
//! The contract is `specs/004-first-demo/contracts/experience-api.md`.
#![forbid(unsafe_code)]

mod compass;
mod json;
mod transcript;

pub use compass::{Compass, CompassPlace, Extra, NOTE_NONE, NOTE_UNKNOWN, Place, PlaceState};
pub use transcript::{Source, Turn};

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

/// One game being played.
#[derive(Clone, Debug)]
pub struct Session {
    turns: Vec<Turn>,
    generation: u64,
    waiting: Waiting,
    message: Option<String>,
    status: Option<Status>,
    compass: Compass,
}

impl Session {
    /// Loads the story file and runs it to its first stop.
    pub fn new(_story: Vec<u8>, _seed: Seed) -> Result<Session, StartError> {
        Err(StartError { message: "not yet built".to_string() })
    }

    /// Gives the game a line the player typed.
    pub fn send_line(&mut self, _at: Stamp, _line: &str) -> Result<(), Refused> {
        Err(Refused::NotWaiting)
    }

    /// The player tapped a place on the compass.
    pub fn tap(&mut self, _at: Stamp, _place: Place) -> Result<(), Refused> {
        Err(Refused::NotWaiting)
    }

    /// The player tapped an extra, by its index.
    pub fn tap_extra(&mut self, _at: Stamp, _index: usize) -> Result<(), Refused> {
        Err(Refused::NotWaiting)
    }

    /// Abandons the game and starts a new one from the same story file.
    pub fn new_game(&mut self, _seed: Seed) {}

    pub fn stamp(&self) -> Stamp {
        Stamp { generation: self.generation, turn_count: self.turns.len() }
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
