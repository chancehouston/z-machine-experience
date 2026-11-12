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
    let options = LoadOptions {
        seed: Some(seed),
        entropy: 0,
    };
    // The reference transcripts were made by an interpreter that calls itself `F`.
    let settings = Settings {
        interpreter_version: b'F',
        ..Settings::default()
    };
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
pub fn replay(
    story: &str,
    script: &str,
    seed: u32,
    mut hook: impl FnMut(&Machine, &Stop),
) -> Replay {
    let mut machine = load(story, seed);
    let script = read_text(script);
    let mut lines = script
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.to_string()));
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
        let line = if wants == Wants::Nothing {
            None
        } else {
            lines.next()
        };
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
    copy.provide_line(command)
        .expect("the machine is waiting for a line");
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
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
}

// ---------------------------------------------------------------------------
// Checks of the answers against the games' own text
// (specs/003-introspection/plan.md, "How the evidence is gathered")
// ---------------------------------------------------------------------------

use zmachine_introspect::{Carried, Introspector, Location, Tri, Unknown};

/// One entry of a hand-made `exits.txt` or `scope.txt`: the walkthrough line the
/// game is waiting for, the room, and the indented `key: value` lines under it.
#[derive(Clone, Debug)]
pub struct Expectation {
    pub line: usize,
    pub room: String,
    pub items: Vec<(String, String)>,
}

/// Reads a hand-made expectations file (see the README beside each).
pub fn expectations(relative: &str) -> Vec<Expectation> {
    let mut entries: Vec<Expectation> = Vec::new();
    for (number, raw) in read_text(relative).lines().enumerate() {
        let line = raw.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("at line ") {
            let (at, room) = rest
                .split_once(" | ")
                .unwrap_or_else(|| panic!("{relative}:{}: {raw:?}", number + 1));
            entries.push(Expectation {
                line: at.trim().parse().unwrap(),
                room: room.trim().to_string(),
                items: vec![],
            });
        } else {
            let (key, value) = line
                .trim()
                .split_once(':')
                .unwrap_or_else(|| panic!("{relative}:{}: {raw:?}", number + 1));
            let entry = entries
                .last_mut()
                .unwrap_or_else(|| panic!("{relative}:{}: no entry yet", number + 1));
            entry
                .items
                .push((key.trim().to_string(), value.trim().to_string()));
        }
    }
    entries
}

/// Room names taken from the hand-made exit expectations: the rooms the entries
/// are in and the rooms their fixed exits lead to. Independent of the queries.
pub fn known_room_names(exits_file: &str) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    for entry in expectations(exits_file) {
        names.insert(entry.room);
        for (_, value) in entry.items {
            if let Some(room) = value.strip_prefix("fixed ") {
                names.insert(room.to_string());
            }
        }
    }
    names
}

/// What the location checks found over a whole walkthrough.
#[derive(Debug, Default)]
pub struct LocationReport {
    pub prompts: usize,
    pub headings: usize,
    pub changes: usize,
    pub dark: usize,
    pub lit: usize,
    pub held: usize,
    pub problems: Vec<String>,
}

/// SC-002, FR-037: at every command prompt the location is known and agrees with
/// the game, by checks that do not read what the query reads. `says_dark` tells
/// whether the game's reply or the answer says the room is dark.
pub fn check_locations(
    game: Game,
    exits_file: &str,
    says_dark: impl Fn(&Stop, &Location) -> bool,
) -> LocationReport {
    let rooms = known_room_names(exits_file);
    let mut report = LocationReport::default();
    let mut previous: Option<(u16, String)> = None;
    walk(game, |machine, stop| {
        if !stop.command_prompt {
            return;
        }
        report.prompts += 1;
        let view = machine.view();
        let answer = Introspector::new(&view).location(&view);
        let here = format!("{} stop {} (line {:?})", game.name, stop.number, stop.line);
        if answer.exactness != zmachine_introspect::Exactness::Exact {
            report.problems.push(format!("{here}: not exact"));
        }
        let location = match answer.value {
            Ok(location) => location,
            Err(reason) => {
                report
                    .problems
                    .push(format!("{here}: location unknown: {reason:?}"));
                return;
            }
        };
        let name = location.name.clone().unwrap_or_else(|e| format!("<{e:?}>"));
        match location.lit {
            Tri::No => report.dark += 1,
            Tri::Yes => report.lit += 1,
            Tri::Unknown => {}
        }
        report.held += matches!(location.holder, Ok(Some(_))) as usize;
        // A reply that opens with a room's name names the reported room.
        let heading = first_line(&stop.text);
        if rooms.contains(heading) {
            report.headings += 1;
            if heading != name {
                report.problems.push(format!(
                    "{here}: the game says {heading:?}, the answer {name:?}"
                ));
            }
        }
        // A change of room is announced: the game prints the room's name on a line
        // of its own (followed by ", in the <vehicle>" when the player is in one),
        // or says it is dark. Two rooms that share a printed name are one place to
        // the player, and a move between them need not be announced.
        if previous
            .as_ref()
            .is_some_and(|(room, was)| *room != location.room && *was != name)
        {
            report.changes += 1;
            let in_vehicle = format!("{name}, in the ");
            let announced = stop
                .text
                .lines()
                .any(|l| l.trim() == name || l.trim().starts_with(&in_vehicle));
            if !announced && !says_dark(stop, &location) {
                report.problems.push(format!(
                    "{here}: moved to {name:?} unannounced: {:?}",
                    first_line(&stop.text)
                ));
            }
        }
        // Versions 5 and 8: the status row the library draws from its own variable.
        if game.name != "zork1" {
            let row = stop.status_row.trim_start();
            let expected = if location.lit == Tri::No {
                "Darkness"
            } else {
                name.as_str()
            };
            if !row.starts_with(expected) {
                report.problems.push(format!(
                    "{here}: the status row is {row:?}, the answer {expected:?}"
                ));
            }
        }
        previous = Some((location.room, name));
    });
    report
}

fn flatten<'a>(carried: &'a [Carried], out: &mut Vec<&'a Carried>) {
    for item in carried {
        out.push(item);
        flatten(&item.contents, out);
    }
}

/// What the inventory checks found over a whole walkthrough.
#[derive(Debug, Default)]
pub struct InventoryReport {
    pub prompts: usize,
    pub compared: usize,
    pub empty: usize,
    pub most_carried: usize,
    /// Stops where the inventory was unknown, with the reason.
    pub unknown: Vec<(usize, Unknown)>,
    /// Stops where the game's reply to `inventory` was not an inventory.
    pub not_an_inventory: Vec<usize>,
    pub problems: Vec<String>,
}

/// SC-005: at every command prompt, what the query reports is what the game says
/// when a copy of it is asked. `heading` begins the game's listing.
pub fn check_inventories(game: Game, heading: &str) -> InventoryReport {
    let mut report = InventoryReport::default();
    walk(game, |machine, stop| {
        if !stop.command_prompt {
            return;
        }
        report.prompts += 1;
        let reply = ask(machine, game.inventory);
        // Only the listing itself: the copy's turn may go on to other things.
        let reply = match reply.split_once("\n\n") {
            Some((listing, _)) => listing.to_string(),
            None => reply,
        };
        let listing = reply.starts_with(heading);
        if !listing && reply != game.empty_handed {
            report.not_an_inventory.push(stop.number);
            return;
        }
        let view = machine.view();
        let answer = Introspector::new(&view).inventory(&view);
        let here = format!("{} stop {} (line {:?})", game.name, stop.number, stop.line);
        let carried = match answer.value {
            Ok(carried) => carried,
            Err(reason) => {
                report.unknown.push((stop.number, reason));
                return;
            }
        };
        report.compared += 1;
        report.most_carried = report.most_carried.max(carried.len());
        if carried.is_empty() || !listing {
            report.empty += 1;
            if !(carried.is_empty() && !listing) {
                report.problems.push(format!(
                    "{here}: answer {carried:?} against reply {reply:?}"
                ));
            }
            return;
        }
        let mut all = Vec::new();
        flatten(&carried, &mut all);
        let names: Vec<String> = all.iter().filter_map(|c| c.name.clone().ok()).collect();
        // Everything the game lists is something reported. The listing is the
        // run of indented lines under the heading; whatever follows it is what
        // else happened on the copy's turn, which is not the inventory.
        for line in reply
            .lines()
            .skip(1)
            .take_while(|l| l.starts_with("  ") || l.ends_with("contains:"))
        {
            let item = line.trim();
            if item.ends_with(':') {
                continue;
            }
            if !names
                .iter()
                .any(|n| !n.is_empty() && item.contains(n.as_str()))
            {
                report.problems.push(format!(
                    "{here}: the game lists {item:?}, not among {names:?}"
                ));
            }
        }
        // Everything reported as carried directly is something the game lists.
        for item in &carried {
            match &item.name {
                Ok(name) if reply.contains(name.as_str()) => {}
                Ok(name) => report
                    .problems
                    .push(format!("{here}: {name:?} is reported but not in {reply:?}")),
                Err(_) => {}
            }
        }
    });
    report
}
