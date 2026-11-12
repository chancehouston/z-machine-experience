//! Bringing a game to places: the driver, a walkthrough, and the crawl
//! (specs/005-corpus-testing/research.md R3, R5, R6).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use zmachine_core::{Event, Key, LoadError, LoadOptions, Machine, Settings};
use zmachine_introspect::{Family, Introspector};

use crate::directions::{Direction, directions};
use crate::ledger::Ledger;
use crate::manifest::Story;
use crate::report::{Case, Outcomes, Reached, Report};
use crate::trial::{LIMIT, Outcome, room, trial};
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
    /// A room and direction whose trials are to be shown, and what they showed.
    watch: Option<(u16, u8)>,
    shown: Vec<Shown>,
}

/// One trial as a person would have seen it: for `corpus explain`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    /// `line N` of the walkthrough, or `crawl`.
    pub origin: String,
    pub word: String,
    pub reported: Reported,
    pub outcome: Outcome,
    /// What the game printed in reply.
    pub text: String,
}

/// The story's own tools, made once.
struct Tools {
    introspector: Introspector,
    directions: Vec<Direction>,
}

impl Tally {
    /// Runs a trial of every direction at a place, `most` trials at the most.
    /// Returns the copies that moved, each with the room it reached, in the
    /// order tried.
    fn place(
        &mut self,
        tools: &Tools,
        machine: &Machine,
        address: u32,
        most: usize,
        origin: &str,
    ) -> Vec<(u16, Machine)> {
        let stop_at = self.trials.saturating_add(most);
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
            if self.trials == stop_at {
                break;
            }
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
            if self.watch == Some((case.room, case.direction)) {
                let mut copy = machine.clone();
                let text = match copy.provide_line(word).and_then(|()| copy.run(LIMIT)) {
                    Ok(_) => copy.take_output(),
                    Err(_) => String::new(),
                };
                self.shown.push(Shown {
                    origin: origin.to_string(),
                    word: word.clone(),
                    reported: said,
                    outcome,
                    text,
                });
            }
            *self.cases.entry(case).or_default() += 1;
            if let Outcome::Moved(room) = outcome {
                moved.push((room, copy));
            }
        }
        moved
    }
}

/// Runs one story: trials at every command prompt of its walkthrough, if it has
/// one, and then the crawl outward from the places reached (research R5).
/// `bytes` is the story file and `script` its walkthrough.
pub fn run_story(story: &Story, bytes: Vec<u8>, script: Option<&str>, ledger: &Ledger) -> Report {
    run_story_traced(story, bytes, script, ledger).0
}

/// The family as the corpus's files write it.
pub fn family(introspector: &Introspector) -> &'static str {
    match introspector.family() {
        Family::Zil => "zil",
        Family::Inform6 => "inform",
        Family::Unrecognised => "none",
    }
}

fn tools(machine: &Machine, pack: Option<&str>) -> Tools {
    let view = machine.view();
    let introspector = match pack {
        Some(text) => Introspector::with_pack(&view, text),
        None => Introspector::new(&view),
    };
    let directions = directions(&introspector, &view);
    Tools {
        introspector,
        directions,
    }
}

/// As [`run_story`], and also the rooms of the crawl's places in the order they
/// were taken.
pub fn run_story_traced(
    story: &Story,
    bytes: Vec<u8>,
    script: Option<&str>,
    ledger: &Ledger,
) -> (Report, Vec<u16>) {
    let (report, order, _) = run_story_watching(story, bytes, script, ledger, None);
    (report, order)
}

/// As [`run_story_traced`], and also every trial from `watch`'s room in its
/// direction, with what the game printed.
pub fn run_story_watching(
    story: &Story,
    bytes: Vec<u8>,
    script: Option<&str>,
    ledger: &Ledger,
    watch: Option<(u16, u8)>,
) -> (Report, Vec<u16>, Vec<Shown>) {
    run_story_with(story, bytes, script, ledger, None, watch)
}

/// As [`run_story_watching`], with a game pack's text given to the
/// introspection interface (specs/006-game-packs/research.md R10, R11).
pub fn run_story_with(
    story: &Story,
    bytes: Vec<u8>,
    script: Option<&str>,
    ledger: &Ledger,
    pack: Option<&str>,
    watch: Option<(u16, u8)>,
) -> (Report, Vec<u16>, Vec<Shown>) {
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
    // The family is a fact about the file, whether or not any place is reached.
    if let Ok(machine) = load(bytes.clone(), story.seed) {
        let made = tools(&machine, pack);
        report.family = family(&made.introspector).to_string();
        report.pack = made.introspector.pack();
        if report.pack == zmachine_introspect::PackState::InForce {
            report.entries = pack
                .and_then(|text| zmachine_introspect::Pack::parse(text).ok())
                .map_or(0, |p| {
                    p.open_attribute.is_some() as usize + p.untrusted_variables.len()
                });
        }
    }
    let mut tally = Tally {
        watch,
        ..Tally::default()
    };
    let mut made: Option<Tools> = None;
    // Places waiting for the crawl: each a machine at a command prompt, in a
    // room no place has been in.
    let mut queue: VecDeque<(u16, Machine)> = VecDeque::new();
    let mut address = None;

    match script {
        Some(script) => {
            // Rooms a trial reached before the walkthrough did, in the order met.
            let mut beside: Vec<(u16, Machine)> = Vec::new();
            let replay = walkthrough(bytes, script, story.seed, |driver, line| {
                let tools = made.get_or_insert_with(|| tools(&driver.machine, pack));
                let Some(at) = driver.command_address else {
                    return;
                };
                address = Some(at);
                let origin =
                    line.map_or("after the last line".to_string(), |n| format!("line {n}"));
                for (room, copy) in tally.place(tools, &driver.machine, at, usize::MAX, &origin) {
                    if !beside.iter().any(|(seen, _)| *seen == room) {
                        beside.push((room, copy));
                    }
                }
            });
            report.walkthrough = replay.command_prompts;
            for (room, copy) in beside {
                if tally.visited.insert(room) {
                    queue.push_back((room, copy));
                }
            }
        }
        None => {
            // The opening lines answer what the game asks before its first
            // command prompt, which is where it waits after them (research R6).
            if let Ok(mut driver) = Driver::new(bytes, story.seed) {
                let mut waiting = true;
                for line in &story.opening {
                    waiting = driver.stop() != Wants::Nothing;
                    driver.command_address = None;
                    driver.answer(line);
                }
                if waiting && driver.stop() == Wants::Command {
                    made = Some(tools(&driver.machine, pack));
                    address = driver.command_address;
                    if let (Some(tools), Some(_)) = (&made, address) {
                        // A place whose room is unknown is still a place.
                        let here = room(&driver.machine, &tools.introspector).unwrap_or(0);
                        tally.visited.insert(here);
                        queue.push_back((here, driver.machine));
                    }
                }
            }
        }
    }

    let mut order = Vec::new();
    if let (Some(tools), Some(address)) = (&made, address) {
        while !queue.is_empty() {
            if report.crawl == report.budget_rooms {
                report.reached = Reached::Rooms;
                break;
            }
            let left = report.budget_trials.saturating_sub(tally.trials);
            if left == 0 {
                report.reached = Reached::Trials;
                break;
            }
            let Some((here, machine)) = queue.pop_front() else {
                break;
            };
            report.crawl += 1;
            order.push(here);
            let before = tally.trials + tally.wordless;
            let moved = tally.place(tools, &machine, address, left, "crawl");
            if tally.trials + tally.wordless - before < tools.directions.len() {
                // The trials ran out part of the way through this place.
                report.reached = Reached::Trials;
                break;
            }
            for (room, copy) in moved {
                if tally.visited.insert(room) {
                    queue.push_back((room, copy));
                }
            }
        }
        // A room put on the queue and never taken was not visited.
        for (room, _) in &queue {
            tally.visited.remove(room);
        }
    }
    report.places = report.walkthrough + report.crawl;
    let shown = std::mem::take(&mut tally.shown);
    (tally.into_report(report, ledger), order, shown)
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
            let known = ledger.knows(&report.id, &case);
            let m = &mut report.marked;
            match (judge(&case), case.reported) {
                (Verdict::Confirmed, said) => {
                    match said {
                        Reported::Fixed(_) => report.fixed_confirmed += count,
                        Reported::Open(_) => m.open_confirmed += count,
                        Reported::Closed(_) => m.closed_confirmed += count,
                        _ => report.none_confirmed += count,
                    }
                    continue;
                }
                (Verdict::Wrong, Reported::Fixed(_)) if known => report.fixed_wrong_known += count,
                (Verdict::Wrong, Reported::Fixed(_)) => report.fixed_wrong_unexplained += count,
                (Verdict::Wrong, Reported::Open(_)) if known => m.open_wrong_known += count,
                (Verdict::Wrong, Reported::Open(_)) => m.open_wrong_unexplained += count,
                (Verdict::Wrong, Reported::Closed(_)) if known => m.closed_wrong_known += count,
                (Verdict::Wrong, Reported::Closed(_)) => m.closed_wrong_unexplained += count,
                (Verdict::Wrong, _) if known => report.none_wrong_known += count,
                (Verdict::Wrong, _) => report.none_wrong_unexplained += count,
                (Verdict::NoneOther, Reported::Closed(_)) => m.closed_other += count,
                (Verdict::NoneOther, _) => report.none_other += count,
                (Verdict::Conditional, Reported::Distrusted) => outcomes(&mut m.distrusted),
                (Verdict::Conditional, _) => outcomes(&mut report.conditional),
                (Verdict::Unknown, _) => outcomes(&mut report.unknown),
                (Verdict::NoVerdict, _) => report.no_verdict += count,
            }
            report.detail.push((case, count));
        }
        report
    }
}
