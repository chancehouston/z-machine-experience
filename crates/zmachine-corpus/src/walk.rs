//! Bringing a game to places: the driver, a walkthrough, and the crawl
//! (specs/005-corpus-testing/research.md R3, R5, R6).

use std::collections::{BTreeMap, BTreeSet};

use zmachine_core::{Event, Key, LoadError, LoadOptions, Machine, Settings};
use zmachine_introspect::{Family, Introspector};

use crate::directions::{Direction, directions};
use crate::ledger::Ledger;
use crate::manifest::Story;
use crate::report::{Case, Outcomes, Report};
use crate::trial::{LIMIT, Outcome, trial};
use crate::verdict::{Reported, Verdict, judge, reported};

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

/// The crawl's budget where the manifest gives none (research R5).
pub const ROOMS: usize = 400;
pub const TRIALS: usize = 40_000;

/// What the trials of a run have found so far.
#[derive(Default)]
struct Tally {
    cases: BTreeMap<Case, usize>,
    wordless: usize,
    trials: usize,
    /// Rooms a place has been in.
    visited: BTreeSet<u16>,
    /// Rooms a fixed exit, reported at a place, leads to.
    named: BTreeSet<u16>,
}

/// The story's own tools, made once.
struct Tools {
    introspector: Introspector,
    directions: Vec<Direction>,
}

impl Tally {
    /// Runs a trial of every direction at a place. Returns the copies that
    /// moved, each with the room it reached, in the order tried.
    fn place(&mut self, tools: &Tools, machine: &Machine, address: u32) -> Vec<(u16, Machine)> {
        let view = machine.view();
        let here = tools
            .introspector
            .location(&view)
            .value
            .ok()
            .map(|l| l.room);
        let exits = tools.introspector.exits(&view).value;
        if let Some(room) = here {
            self.visited.insert(room);
        }
        let mut moved = Vec::new();
        for direction in &tools.directions {
            let Some(word) = &direction.word else {
                self.wordless += 1;
                continue;
            };
            let said = reported(&exits, direction.number);
            if let Reported::Fixed(room) = said {
                self.named.insert(room);
            }
            let (outcome, copy) = trial(machine, &tools.introspector, address, word, LIMIT);
            self.trials += 1;
            let case = Case {
                room: here.unwrap_or(0),
                direction: direction.number,
                reported: said,
                outcome,
            };
            *self.cases.entry(case).or_default() += 1;
            if let Outcome::Moved(room) = outcome {
                moved.push((room, copy));
            }
        }
        moved
    }
}

/// Runs one story: trials at every command prompt of its walkthrough, if it has
/// one. `bytes` is the story file and `script` its walkthrough.
pub fn run_story(story: &Story, bytes: Vec<u8>, script: Option<&str>, ledger: &Ledger) -> Report {
    let mut report = Report {
        id: story.id.clone(),
        sha256: story.sha256.clone(),
        version: bytes.first().copied().unwrap_or(0),
        family: "none".to_string(),
        seed: story.seed,
        budget_rooms: story.rooms.unwrap_or(ROOMS),
        budget_trials: story.trials.unwrap_or(TRIALS),
        ..Report::default()
    };
    let mut tally = Tally::default();
    let mut tools: Option<Tools> = None;
    let replay = walkthrough(bytes, script.unwrap_or(""), story.seed, |driver, _| {
        let tools = tools.get_or_insert_with(|| {
            let view = driver.machine.view();
            let introspector = Introspector::new(&view);
            let directions = directions(&introspector, &view);
            Tools {
                introspector,
                directions,
            }
        });
        if let Some(address) = driver.command_address {
            tally.place(tools, &driver.machine, address);
        }
    });
    report.walkthrough = replay.command_prompts;
    report.places = report.walkthrough + report.crawl;
    if let Some(tools) = &tools {
        report.family = match tools.introspector.family() {
            Family::Zil => "zil",
            Family::Inform6 => "inform",
            Family::Unrecognised => "none",
        }
        .to_string();
    }
    tally.into_report(report, ledger)
}

impl Tally {
    /// Fills a report's figures from the cases counted.
    fn into_report(self, mut report: Report, ledger: &Ledger) -> Report {
        report.trials = self.trials;
        report.wordless = self.wordless;
        report.rooms_visited = self.visited.len();
        report.named_not_reached = self.named.difference(&self.visited).count();
        for (case, count) in self.cases {
            let outcomes = |o: &mut Outcomes| match case.outcome {
                Outcome::Moved(_) => o.moved += count,
                Outcome::Stayed => o.stayed += count,
                Outcome::Question => o.question += count,
                Outcome::Ended => o.ended += count,
                Outcome::Fault => o.fault += count,
                Outcome::Lost => {}
            };
            let fixed = matches!(case.reported, Reported::Fixed(_));
            match judge(&case) {
                Verdict::Confirmed if fixed => {
                    report.fixed_confirmed += count;
                    continue;
                }
                Verdict::Confirmed => {
                    report.none_confirmed += count;
                    continue;
                }
                Verdict::Wrong => match (fixed, ledger.knows(&report.id, &case)) {
                    (true, true) => report.fixed_wrong_known += count,
                    (true, false) => report.fixed_wrong_unexplained += count,
                    (false, true) => report.none_wrong_known += count,
                    (false, false) => report.none_wrong_unexplained += count,
                },
                Verdict::NoneOther => report.none_other += count,
                Verdict::Conditional => outcomes(&mut report.conditional),
                Verdict::Unknown => outcomes(&mut report.unknown),
                Verdict::NoVerdict => report.no_verdict += count,
            }
            report.detail.push((case, count));
        }
        report
    }
}
