//! The walkthrough driver shared by the integration tests
//! (specs/003-introspection/plan.md, "How the evidence is gathered").
//!
//! It replays a command script through the core and, every time the game stops
//! for its host, hands the machine to a hook by shared reference together with the
//! game's own text since the last stop. Tests ask their queries in the hook.
#![allow(dead_code)]

use zmachine_core::{Event, Key, LoadOptions, Machine, Settings};

pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// A game the repository holds a full walkthrough for.
#[derive(Clone, Copy, Debug)]
pub struct Game {
    pub name: &'static str,
    pub story: &'static str,
    pub walkthrough: &'static str,
    pub reference: &'static str,
    pub seed: u32,
    /// The command that lists what the player carries.
    pub inventory: &'static str,
    /// The game's whole reply when the player carries nothing.
    pub empty_handed: &'static str,
}

pub const ZORK1: Game = Game {
    name: "zork1",
    story: "stories/zork1/zork1.z3",
    walkthrough: "stories/zork1/fidelity/walkthrough.txt",
    reference: "stories/zork1/fidelity/reference.scr",
    seed: 11,
    inventory: "inventory",
    empty_handed: "You are empty-handed.",
};

pub const ADVENT5: Game = Game {
    name: "advent.z5",
    story: "stories/advent/advent.z5",
    walkthrough: "stories/advent/fidelity/walkthrough.txt",
    reference: "stories/advent/fidelity/reference_z5.scr",
    seed: 1,
    inventory: "inventory",
    empty_handed: "You're carrying nothing.",
};

pub const ADVENT8: Game = Game {
    name: "advent.z8",
    story: "stories/advent/advent.z8",
    reference: "stories/advent/fidelity/reference_z8.scr",
    ..ADVENT5
};

pub const GAMES: [Game; 3] = [ZORK1, ADVENT5, ADVENT8];

pub fn read(relative: &str) -> Vec<u8> {
    std::fs::read(format!("{ROOT}/{relative}")).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

pub fn read_text(relative: &str) -> String {
    String::from_utf8(read(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

pub fn load(story: &str, seed: u32) -> Machine {
    let options = LoadOptions { seed: Some(seed), entropy: 0 };
    // The reference transcripts were made by an interpreter that calls itself `F`.
    let settings = Settings { interpreter_version: b'F', ..Settings::default() };
    Machine::load_with(read(story), options, settings).unwrap_or_else(|e| panic!("{story}: {e}"))
}

/// What the game is waiting for at a stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wants {
    Line,
    Key,
    TranscriptName,
    SaveName,
    RestoreName,
    /// The game has ended, or faulted.
    Nothing,
}

/// One stop of the game for its host.
#[derive(Clone, Debug)]
pub struct Stop {
    /// Stops are counted from 1.
    pub number: usize,
    pub wants: Wants,
    /// The game wants a line and its text ends with the `>` prompt.
    pub command_prompt: bool,
    /// The lower window's text since the previous stop.
    pub text: String,
    /// The first row of the screen canvas (versions 5 and 8), without trailing spaces.
    pub status_row: String,
    /// The script line (from 1) about to be given in answer, and its text.
    pub line: Option<(usize, String)>,
}

/// What a whole replay produced.
#[derive(Debug, Default)]
pub struct Replay {
    pub transcript: String,
    pub stops: usize,
    pub command_prompts: usize,
    /// Each save file the game wrote, with the name the script gave it.
    pub saves: Vec<(String, Vec<u8>)>,
}

/// Replays a script, calling `hook` at every stop before the stop is answered.
pub fn replay(story: &str, script: &str, seed: u32, mut hook: impl FnMut(&Machine, &Stop)) -> Replay {
    let mut machine = load(story, seed);
    let script = read_text(script);
    let mut lines = script.lines().enumerate().map(|(i, l)| (i + 1, l.to_string()));
    let mut replay = Replay::default();
    loop {
        let event = loop {
            match machine.run(1_000_000).expect("the driver answers in turn") {
                Event::BudgetExhausted => continue,
                other => break other,
            }
        };
        let text = machine.take_output();
        replay.transcript.push_str(&machine.take_transcript());
        let wants = match &event {
            Event::LineWanted { .. } => Wants::Line,
            Event::KeyWanted => Wants::Key,
            Event::TranscriptRequested => Wants::TranscriptName,
            Event::SaveRequested(_) => Wants::SaveName,
            Event::RestoreRequested => Wants::RestoreName,
            _ => Wants::Nothing,
        };
        let line = if wants == Wants::Nothing { None } else { lines.next() };
        replay.stops += 1;
        let stop = Stop {
            number: replay.stops,
            wants,
            command_prompt: wants == Wants::Line && text.trim_end_matches(' ').ends_with('>'),
            status_row: machine.screen().row_text(1).trim_end().to_string(),
            text,
            line: line.clone(),
        };
        replay.command_prompts += stop.command_prompt as usize;
        hook(&machine, &stop);
        let Some((_, answer)) = line else {
            // The game ended, or the script did while the game was waiting.
            return replay;
        };
        match event {
            Event::LineWanted { .. } => machine.provide_line(&answer).unwrap(),
            Event::KeyWanted => {
                let key = answer.chars().next().map_or(Key::Return, Key::Char);
                machine.provide_key(key).unwrap();
            }
            Event::TranscriptRequested => machine.transcript_result(true).unwrap(),
            Event::SaveRequested(bytes) => {
                replay.saves.push((answer, bytes));
                machine.save_result(true).unwrap();
            }
            Event::RestoreRequested => machine.provide_restore(None).unwrap(),
            _ => unreachable!("a stop that wants nothing has no line"),
        }
    }
}

/// Replays a game's walkthrough.
pub fn walk(game: Game, hook: impl FnMut(&Machine, &Stop)) -> Replay {
    replay(game.story, game.walkthrough, game.seed, hook)
}

/// Gives a copy of the machine one command and returns the copy's reply, without
/// the prompt that follows it. The machine itself is not given the command.
pub fn ask(machine: &Machine, command: &str) -> String {
    let mut copy = machine.clone();
    copy.provide_line(command).expect("the machine is waiting for a line");
    loop {
        match copy.run(1_000_000).expect("the copy runs") {
            Event::BudgetExhausted => continue,
            _ => break,
        }
    }
    let reply = copy.take_output();
    reply.trim_end().trim_end_matches('>').trim().to_string()
}

/// The first line of a reply that is not blank.
pub fn first_line(text: &str) -> &str {
    text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("")
}
