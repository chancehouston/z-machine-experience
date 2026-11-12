//! Bringing a game to places: the driver, a walkthrough, and the crawl
//! (specs/005-corpus-testing/research.md R3, R5, R6).

use zmachine_core::{Event, Key, LoadError, LoadOptions, Machine, Settings};

/// Loads a story as the project's reference transcripts were made: a fixed seed,
/// and an interpreter that calls itself `F`.
pub fn load(story: Vec<u8>, seed: u32) -> Result<Machine, LoadError> {
    let options = LoadOptions {
        seed: Some(seed),
        entropy: 0,
    };
    let settings = Settings {
        interpreter_version: b'F',
        ..Settings::default()
    };
    Machine::load_with(story, options, settings)
}

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
/// question by where in its code the game is waiting: the command prompt is
/// where it waited at its first line prompt (research R3).
#[derive(Clone, Debug)]
pub struct Driver {
    pub machine: Machine,
    /// Where the game was waiting at its first line prompt.
    pub command_address: Option<u32>,
    /// The game's transcript stream so far.
    pub transcript: String,
    /// What the game stopped for, until it is answered.
    event: Option<Event>,
}

impl Driver {
    pub fn new(story: Vec<u8>, seed: u32) -> Result<Driver, LoadError> {
        Ok(Driver::from_machine(load(story, seed)?, None))
    }

    /// A driver for a machine that is already running, such as a trial's copy.
    pub fn from_machine(machine: Machine, command_address: Option<u32>) -> Driver {
        Driver {
            machine,
            command_address,
            transcript: String::new(),
            event: None,
        }
    }

    /// Runs the game to its next stop and says what it wants. The game's
    /// output is thrown away; its transcript stream is kept.
    pub fn stop(&mut self) -> Wants {
        let event = loop {
            match self.machine.run(1_000_000) {
                Ok(Event::BudgetExhausted) => continue,
                Ok(event) => break event,
                Err(_) => {
                    self.event = None;
                    return Wants::Nothing;
                }
            }
        };
        self.machine.take_output();
        self.transcript.push_str(&self.machine.take_transcript());
        let wants = match &event {
            Event::LineWanted { .. } => {
                let here = self.machine.waiting_at();
                if *self.command_address.get_or_insert(here.unwrap_or(0)) == here.unwrap_or(0) {
                    Wants::Command
                } else {
                    Wants::Answer
                }
            }
            Event::KeyWanted
            | Event::SaveRequested(_)
            | Event::RestoreRequested
            | Event::TranscriptRequested => Wants::Answer,
            Event::Finished | Event::Fault(_) | Event::BudgetExhausted => Wants::Nothing,
        };
        self.event = (wants != Wants::Nothing).then_some(event);
        wants
    }

    /// Answers the stop the game is at: a line with the line, a key with its
    /// first character, and a request for a save, restore or transcript as the
    /// earlier features' driver does (a save and a transcript succeed; a restore
    /// is declined). Does nothing when the game is not waiting.
    pub fn answer(&mut self, line: &str) {
        let answered = match self.event.take() {
            Some(Event::LineWanted { .. }) => self.machine.provide_line(line),
            Some(Event::KeyWanted) => {
                let key = line.chars().next().map_or(Key::Return, Key::Char);
                self.machine.provide_key(key)
            }
            Some(Event::TranscriptRequested) => self.machine.transcript_result(true),
            Some(Event::SaveRequested(_)) => self.machine.save_result(true),
            Some(Event::RestoreRequested) => self.machine.provide_restore(None),
            _ => Ok(()),
        };
        // A refused answer leaves the machine as it was; the next stop reports it.
        let _ = answered;
    }
}

/// What replaying a script produced.
#[derive(Debug, Default)]
pub struct Replay {
    pub transcript: String,
    pub command_prompts: usize,
    pub stops: usize,
    /// The driver as the script left it.
    pub driver: Option<Driver>,
}

/// Replays a script, one line to each stop, calling `each_place` at every
/// command prompt with the driver and the script line (numbered from 1) about
/// to be given, if there is one. Stops when the game does or the script ends.
pub fn walkthrough(
    story: Vec<u8>,
    script: &str,
    seed: u32,
    mut each_place: impl FnMut(&Driver, Option<usize>),
) -> Replay {
    let mut replay = Replay::default();
    let Ok(mut driver) = Driver::new(story, seed) else {
        return replay;
    };
    let mut lines = script.lines().enumerate().map(|(i, l)| (i + 1, l));
    loop {
        let wants = driver.stop();
        replay.stops += 1;
        let line = if wants == Wants::Nothing {
            None
        } else {
            lines.next()
        };
        if wants == Wants::Command {
            replay.command_prompts += 1;
            each_place(&driver, line.map(|(number, _)| number));
        }
        let Some((_, answer)) = line else {
            break;
        };
        driver.answer(answer);
    }
    replay.transcript = driver.transcript.clone();
    replay.driver = Some(driver);
    replay
}
