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

use zmachine_introspect::{ExitKind, Exits};

/// Renders an exits answer in the form of the hand-made files: one
/// `word: fixed <room>` or `word: conditional` per exit, by its longest word.
fn describe(exits: &Exits) -> Vec<String> {
    let mut out: Vec<String> = exits
        .exits
        .iter()
        .map(|exit| {
            let word = exit
                .words
                .iter()
                .max_by_key(|w| w.len())
                .cloned()
                .unwrap_or_else(|| format!("#{}", exit.direction));
            match exit.kind {
                ExitKind::Fixed => format!(
                    "{word}: fixed {}",
                    exit.destination_name.clone().map_or("?".to_string(), |n| n
                        .unwrap_or_else(|e| format!("<{e:?}>")))
                ),
                ExitKind::Conditional => format!("{word}: conditional"),
            }
        })
        .collect();
    out.extend(
        exits
            .unknown
            .iter()
            .map(|u| format!("{:?}: unknown {:?}", u.words, u.reason)),
    );
    out
}

/// SC-004: at each walkthrough line a hand-made entry names, the reported exits
/// are the expected ones, direction for direction. `word_len` is how many
/// characters of a word the story file's dictionary keeps. Returns the number of
/// entries checked and the problems found.
pub fn check_expected_exits(game: Game, exits_file: &str, word_len: usize) -> (usize, Vec<String>) {
    let expected = expectations(exits_file);
    let mut problems = Vec::new();
    let mut checked = 0;
    walk(game, |machine, stop| {
        let Some((line, _)) = &stop.line else { return };
        for entry in expected
            .iter()
            .filter(|e| e.line == *line && stop.wants == Wants::Line)
        {
            checked += 1;
            let here = format!("{} line {line} ({})", game.name, entry.room);
            let view = machine.view();
            let i = Introspector::new(&view);
            let name = i.location(&view).value.ok().and_then(|l| l.name.ok());
            if name.as_deref() != Some(entry.room.as_str()) {
                problems.push(format!("{here}: the location is {name:?}"));
                continue;
            }
            let answer = i.exits(&view);
            if answer.exactness != zmachine_introspect::Exactness::Exact {
                problems.push(format!("{here}: not exact"));
            }
            let exits = match answer.value {
                Ok(exits) => exits,
                Err(reason) => {
                    problems.push(format!("{here}: exits unknown: {reason:?}"));
                    continue;
                }
            };
            let reported = describe(&exits);
            let mut matched = vec![false; exits.exits.len()];
            for (direction, what) in &entry.items {
                let cut: String = direction.chars().take(word_len).collect();
                let Some(index) = exits.exits.iter().position(|e| e.words.contains(&cut)) else {
                    problems.push(format!(
                        "{here}: no exit {direction} ({what}); reported {reported:?}"
                    ));
                    continue;
                };
                matched[index] = true;
                let exit = &exits.exits[index];
                let ok = match what.strip_prefix("fixed ") {
                    Some(room) => {
                        exit.kind == ExitKind::Fixed
                            && exit
                                .destination_name
                                .clone()
                                .and_then(Result::ok)
                                .as_deref()
                                == Some(room)
                            && exit.destination.is_some()
                    }
                    None => {
                        what == "conditional"
                            && exit.kind == ExitKind::Conditional
                            && exit.destination.is_none()
                            && exit.destination_name.is_none()
                    }
                };
                if !ok {
                    problems.push(format!(
                        "{here}: {direction} should be {what}; reported {reported:?}"
                    ));
                }
            }
            for (index, exit) in exits.exits.iter().enumerate() {
                if !matched[index] {
                    problems.push(format!(
                        "{here}: an exit that is not expected: {:?}; reported {reported:?}",
                        exit.words
                    ));
                }
            }
            if !exits.unknown.is_empty() {
                problems.push(format!("{here}: directions unknown: {reported:?}"));
            }
        }
    });
    (checked, problems)
}

/// What following the reported exits found over a whole walkthrough.
#[derive(Debug, Default)]
pub struct MoveReport {
    /// Moves made through a reported fixed exit that arrived where it said.
    pub arrived: usize,
    /// Commands naming a reported fixed exit after which the player had not moved.
    pub stayed: Vec<String>,
    /// Commands naming a reported conditional exit.
    pub conditional: usize,
    /// Reported exits with no word.
    pub wordless: usize,
    pub exits_seen: usize,
    pub problems: Vec<String>,
}

/// SC-003, FR-038: every move through an exit reported with a destination arrives
/// at that destination.
pub fn check_moves(game: Game, word_len: usize) -> MoveReport {
    let mut report = MoveReport::default();
    let mut previous: Option<(u16, Exits, String)> = None;
    walk(game, |machine, stop| {
        if !stop.command_prompt {
            return;
        }
        let view = machine.view();
        let i = Introspector::new(&view);
        let here = format!("{} stop {} (line {:?})", game.name, stop.number, stop.line);
        let (Ok(location), answer) = (i.location(&view).value, i.exits(&view)) else {
            report.problems.push(format!("{here}: location unknown"));
            return;
        };
        let exits = match answer.value {
            Ok(exits) => exits,
            Err(reason) => {
                report
                    .problems
                    .push(format!("{here}: exits unknown: {reason:?}"));
                return;
            }
        };
        if let Some((room, before, command)) = &previous {
            let word: String = command
                .trim()
                .to_ascii_lowercase()
                .chars()
                .take(word_len)
                .collect();
            if let Some(exit) = before.exits.iter().find(|e| e.words.contains(&word)) {
                match (exit.kind, exit.destination) {
                    (ExitKind::Fixed, Some(destination)) if destination == location.room => report.arrived += 1,
                    (ExitKind::Fixed, Some(_)) if location.room == *room => report.stayed.push(format!("{here}: {command:?}")),
                    (ExitKind::Fixed, destination) => report.problems.push(format!("{here}: {command:?} was reported to lead to {destination:?} ({:?}) but the player is in {} {:?}", exit.destination_name, location.room, location.name)),
                    (ExitKind::Conditional, _) => report.conditional += 1,
                }
            }
        }
        report.exits_seen += exits.exits.len();
        report.wordless += exits.exits.iter().filter(|e| e.words.is_empty()).count();
        // No direction twice, nor in both lists (data-model.md validation rules).
        let mut directions: Vec<u8> = exits
            .exits
            .iter()
            .map(|e| e.direction)
            .chain(exits.unknown.iter().map(|u| u.direction))
            .collect();
        directions.sort_unstable();
        if directions.windows(2).any(|pair| pair[0] == pair[1]) {
            report
                .problems
                .push(format!("{here}: a direction is reported twice"));
        }
        let command = stop.line.clone().map(|(_, text)| text).unwrap_or_default();
        previous = Some((location.room, exits, command));
    });
    report
}

/// What a wandering tour found.
#[derive(Debug, Default)]
pub struct TourReport {
    pub arrived: usize,
    pub rooms: std::collections::BTreeSet<u16>,
    pub exits_tried: std::collections::BTreeSet<(u16, u8)>,
    pub stayed: Vec<String>,
    /// Moves after which the player was not in the room the exit named, but in
    /// one with the same printed name: the game's own code, run on arrival, moved
    /// the player on (specification section 8.3). The destination's name is kept.
    pub redirected: Vec<String>,
    pub problems: Vec<String>,
    /// Why the tour ended before its last step, if it did.
    pub ended: Option<String>,
}

/// Wanders a game beyond its walkthrough: after `prefix`, at each prompt takes
/// one of the fixed exits the query reports, types its word, and checks that the
/// game puts the player where the exit said (SC-003, FR-038). The choice among
/// exits is a fixed function of the step, so every run is the same.
pub fn tour(game: Game, prefix: &[&str], steps: usize) -> TourReport {
    let mut machine = load(game.story, game.seed);
    let mut report = TourReport::default();
    let mut prefix = prefix.iter();
    let mut pending: Option<(u16, u16, Option<String>, String)> = None;
    let mut step = 0usize;
    loop {
        let event = loop {
            match machine.run(1_000_000).expect("the driver answers in turn") {
                Event::BudgetExhausted => continue,
                other => break other,
            }
        };
        let text = machine.take_output();
        if !matches!(event, Event::LineWanted { .. }) || !text.trim_end_matches(' ').ends_with('>')
        {
            report.ended = Some(format!(
                "step {step}: {event:?} after {:?}",
                first_line(&text)
            ));
            return report;
        }
        if let Some(command) = prefix.next() {
            machine.provide_line(command).unwrap();
            continue;
        }
        let view = machine.view();
        let i = Introspector::new(&view);
        let (Ok(location), Ok(exits)) = (i.location(&view).value, i.exits(&view).value) else {
            report
                .problems
                .push(format!("step {step}: location or exits unknown"));
            return report;
        };
        report.rooms.insert(location.room);
        if let Some((from, destination, destination_name, command)) = pending.take() {
            if location.room == destination {
                report.arrived += 1;
            } else if destination_name.is_some() && location.name.clone().ok() == destination_name {
                report.redirected.push(destination_name.unwrap());
            } else if location.room == from {
                report.stayed.push(format!(
                    "step {step}: {command:?} from {from}: {:?}",
                    first_line(&text)
                ));
            } else {
                report.problems.push(format!("step {step}: {command:?} from {from} was to lead to {destination}, but the player is in {} {:?}", location.room, location.name));
            }
        }
        if step == steps {
            return report;
        }
        let fixed: Vec<_> = exits
            .exits
            .iter()
            .filter(|e| e.kind == ExitKind::Fixed && !e.words.is_empty())
            .collect();
        // Prefer an exit to a room not yet seen, then one not yet tried from this
        // room; otherwise one chosen by a fixed scramble of the step number, so
        // that the tour does not settle into a loop.
        let unseen = fixed
            .iter()
            .find(|e| !report.rooms.contains(&e.destination.unwrap()));
        let untried = fixed
            .iter()
            .find(|e| !report.exits_tried.contains(&(location.room, e.direction)));
        let scramble = (step as u64)
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
            >> 33;
        let command = match unseen
            .or(untried)
            .or(fixed.get(scramble as usize % fixed.len().max(1)))
        {
            Some(exit) => {
                report.exits_tried.insert((location.room, exit.direction));
                let word = exit.words.iter().max_by_key(|w| w.len()).unwrap().clone();
                let name = exit.destination_name.clone().and_then(Result::ok);
                pending = Some((location.room, exit.destination.unwrap(), name, word.clone()));
                word
            }
            None => "look".to_string(),
        };
        machine.provide_line(&command).unwrap();
        step += 1;
    }
}

use zmachine_introspect::{Exactness, Reach, Scope};

/// What the scope checks found over a whole walkthrough.
#[derive(Debug, Default)]
pub struct ScopeReport {
    pub prompts: usize,
    /// Hand-made entries checked.
    pub entries: usize,
    pub most_objects: usize,
    pub nested: usize,
    pub player_unknown: usize,
    pub problems: Vec<String>,
}

/// SC-006 and the rules every scope answer keeps (specification section 9): at
/// each walkthrough line a hand-made entry names, everything it lists is in scope
/// and nothing it excludes is; and at every command prompt the answer is
/// approximate and well formed. `also` is given each answer for family checks.
pub fn check_scopes(game: Game, scope_file: &str, mut also: impl FnMut(&Scope, &str) -> Vec<String>) -> ScopeReport {
    let expected = expectations(scope_file);
    let mut report = ScopeReport::default();
    walk(game, |machine, stop| {
        if stop.wants != Wants::Line {
            return;
        }
        let view = machine.view();
        let i = Introspector::new(&view);
        let here = format!("{} stop {} (line {:?})", game.name, stop.number, stop.line.as_ref().map(|l| l.0));
        let answer = i.scope(&view);
        if answer.exactness != Exactness::Approximate {
            report.problems.push(format!("{here}: scope is not labelled approximate"));
        }
        let Ok(location) = i.location(&view).value else {
            return;
        };
        let scope = match answer.value {
            Ok(scope) => scope,
            Err(reason) => {
                report.problems.push(format!("{here}: scope unknown: {reason:?}"));
                return;
            }
        };
        if stop.command_prompt {
            report.prompts += 1;
        }
        report.most_objects = report.most_objects.max(scope.objects.len());
        report.player_unknown += scope.player.is_err() as usize;
        // No object twice; the room is not an entry.
        let mut numbers: Vec<u16> = scope.objects.iter().map(|e| e.object).collect();
        numbers.sort_unstable();
        if numbers.windows(2).any(|pair| pair[0] == pair[1]) {
            report.problems.push(format!("{here}: an object is listed twice"));
        }
        if numbers.contains(&location.room) {
            report.problems.push(format!("{here}: the room is an entry"));
        }
        for entry in &scope.objects {
            match (entry.reach, entry.within) {
                (Reach::Inside | Reach::On, Some(within)) => {
                    report.nested += 1;
                    if !numbers.contains(&within) {
                        report.problems.push(format!("{here}: {:?} is within {within}, which is not listed", entry.name));
                    }
                }
                (Reach::Inside | Reach::On, None) => report.problems.push(format!("{here}: {:?} is within nothing", entry.name)),
                (_, Some(_)) => report.problems.push(format!("{here}: {:?} is {:?} yet within something", entry.name, entry.reach)),
                _ => {}
            }
        }
        // What is carried is the inventory, and the reverse.
        let mut carried: Vec<u16> = scope.objects.iter().filter(|e| matches!(e.reach, Reach::Carried | Reach::Worn)).map(|e| e.object).collect();
        carried.sort_unstable();
        match i.inventory(&view).value {
            Ok(inventory) => {
                let mut held: Vec<u16> = inventory.iter().map(|c| c.object).collect();
                held.sort_unstable();
                if held != carried || scope.player.is_err() {
                    report.problems.push(format!("{here}: carried {carried:?} against inventory {held:?}"));
                }
            }
            Err(reason) => {
                if scope.player != Err(reason) || !carried.is_empty() {
                    report.problems.push(format!("{here}: the player is unknown ({reason:?}) but scope says {:?} and carries {carried:?}", scope.player));
                }
            }
        }
        report.problems.extend(also(&scope, &here));
        // The hand-made expectations for this line.
        let names: Vec<String> = scope.objects.iter().filter_map(|e| e.name.clone().ok()).collect();
        let Some((line, _)) = &stop.line else { return };
        for entry in expected.iter().filter(|e| e.line == *line) {
            report.entries += 1;
            if location.name.as_deref() != Ok(entry.room.as_str()) {
                report.problems.push(format!("{here}: expected to be in {:?}, the location is {:?}", entry.room, location.name));
                continue;
            }
            for (key, list) in &entry.items {
                for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
                    let listed = names.iter().any(|n| n == name);
                    match key.as_str() {
                        "has" if !listed => report.problems.push(format!("{here} ({}): {name:?} should be in scope; listed {names:?}", entry.room)),
                        "not" if listed => report.problems.push(format!("{here} ({}): {name:?} should not be in scope", entry.room)),
                        "has" | "not" => {}
                        other => report.problems.push(format!("{here}: unknown key {other:?}")),
                    }
                }
            }
        }
    });
    report
}
