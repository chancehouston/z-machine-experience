//! Bringing a game to places: the driver, a walkthrough, and the crawl
//! (specs/005-corpus-testing/research.md R3, R5, R6).

use zmachine_core::{LoadError, Machine};

/// What a game is waiting for when it stops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wants {
    /// A line, at the command prompt.
    Command,
    /// A line anywhere else, a key, or a save, restore or transcript name.
    Answer,
    /// Nothing: the game has ended or faulted.
    Nothing,
}

/// Runs a story and answers what it asks, telling a command prompt from a
/// question by where in its code the game is waiting.
pub struct Driver {
    pub machine: Machine,
    /// Where the game was waiting at its first line prompt.
    pub command_address: Option<u32>,
    /// The game's transcript stream so far.
    pub transcript: String,
}

impl Driver {
    pub fn new(story: Vec<u8>, seed: u32) -> Result<Driver, LoadError> {
        let _ = (story, seed);
        unimplemented!("specs/005-corpus-testing/tasks.md T006")
    }

    /// Runs the game to its next stop.
    pub fn stop(&mut self) -> Wants {
        Wants::Nothing
    }

    /// Answers the stop the game is at.
    pub fn answer(&mut self, _line: &str) {}
}

/// What replaying a script produced.
#[derive(Debug, Default)]
pub struct Replay {
    pub transcript: String,
    pub command_prompts: usize,
    pub stops: usize,
}

/// Replays a script, calling `each_place` at every command prompt with the
/// driver and the script line (numbered from 1) about to be given.
pub fn walkthrough(
    _story: Vec<u8>,
    _script: &str,
    _seed: u32,
    _each_place: impl FnMut(&Driver, Option<usize>),
) -> Replay {
    Replay::default()
}
