//! The whole game through the experience layer (specs/004-first-demo/research.md
//! R14; FR-031; SC-004, SC-005, SC-006). What the layer records must be what a
//! terminal would have shown, which is the reference transcript from its first
//! command on.

mod common;

use common::*;
use zmachine_experience::{Session, Waiting};

const WALKTHROUGH: &str = "stories/zork1/fidelity/walkthrough.txt";
const REFERENCE: &str = "stories/zork1/fidelity/reference.scr";
const SEED: u32 = 11;

/// The walkthrough's commands: its lines after the two that start a transcript,
/// which the layer has no use for.
pub fn commands() -> Vec<String> {
    let script = read_text(WALKTHROUGH);
    let mut lines = script.lines();
    assert_eq!(lines.next(), Some("script"));
    lines.next().expect("the transcript's file name");
    lines.map(str::to_string).collect()
}

/// The reference transcript from its first command line to its end.
fn reference_from_first_command(first: &str) -> String {
    let reference = read_text(REFERENCE);
    let marker = format!("\n>{first}\n");
    let at = reference
        .find(&marker)
        .expect("the first command is in the reference");
    reference[at + 1..].to_string()
}

/// What a terminal would have shown from the first command on: the prompt, then
/// for each turn the command, a line feed, and what the game printed.
fn shown(session: &Session) -> String {
    let mut text = String::from(">");
    for turn in &session.turns()[1..] {
        text.push_str(
            turn.command
                .as_deref()
                .expect("every turn after the opening has a command"),
        );
        text.push('\n');
        text.push_str(&turn.text);
    }
    text
}

/// Compares, reporting the first difference with its surroundings.
fn assert_same(ours: &str, reference: &str) {
    if ours == reference {
        return;
    }
    let at = ours
        .bytes()
        .zip(reference.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(ours.len().min(reference.len()));
    let from = at.saturating_sub(120);
    let end = |s: &str| (at + 120).min(s.len());
    panic!(
        "the text differs from the reference at byte {at} ({} bytes against {}):\n--- reference ---\n{}\n--- ours ---\n{}",
        ours.len(),
        reference.len(),
        &reference[from..end(reference)],
        &ours[from..end(ours)]
    );
}

// R14, FR-031: every command typed. The layer's turns, laid end to end, are the
// reference transcript from its first command to its end, byte for byte.
#[test]
fn the_walkthrough_typed_gives_the_reference_text() {
    let commands = commands();
    assert_eq!(commands.len(), 331);
    let mut session = game(SEED);
    assert!(
        session.turns()[0].text.ends_with('>'),
        "the opening ends with the prompt"
    );
    for command in &commands {
        say(&mut session, command);
    }
    assert_eq!(
        session.waiting(),
        Waiting::Ended,
        "the walkthrough ends by quitting"
    );
    assert_eq!(session.turns().len(), 332);
    assert_same(
        &shown(&session),
        &reference_from_first_command(&commands[0]),
    );
}

use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_experience::{Place, PlaceState, Source};
use zmachine_introspect::{ExitKind, Introspector};

/// The twelve places and their English words, written here independently of the
/// layer, for checking it.
const PLACES: [&str; 12] = [
    "north",
    "northeast",
    "east",
    "southeast",
    "south",
    "southwest",
    "west",
    "northwest",
    "up",
    "down",
    "in",
    "out",
];

/// An identical game kept beside the session: the same story, seed and lines,
/// driven through the core directly.
struct Shadow {
    machine: Machine,
    introspector: Introspector,
    /// What the game printed since the last line it was given.
    text: String,
}

impl Shadow {
    fn new() -> Shadow {
        let options = LoadOptions {
            seed: Some(SEED),
            entropy: 0,
        };
        let machine = Machine::load(read("stories/zork1/zork1.z3"), options).unwrap();
        let introspector = Introspector::new(&machine.view());
        let mut shadow = Shadow {
            machine,
            introspector,
            text: String::new(),
        };
        shadow.run();
        shadow
    }

    fn run(&mut self) {
        self.text.clear();
        loop {
            let event = self.machine.run(1_000_000).unwrap();
            self.text.push_str(&self.machine.take_output());
            match event {
                Event::BudgetExhausted => {}
                Event::SaveRequested(_) => self.machine.save_result(false).unwrap(),
                Event::RestoreRequested => self.machine.provide_restore(None).unwrap(),
                Event::TranscriptRequested => self.machine.transcript_result(false).unwrap(),
                _ => return,
            }
        }
    }

    fn give(&mut self, line: &str) {
        self.machine.provide_line(line).unwrap();
        self.run();
    }

    /// The exits the introspection interface reports, as (place or word, state,
    /// destination name), by the test's own mapping from words to places.
    fn exits(&self) -> Vec<(String, PlaceState, Option<String>)> {
        let view = self.machine.view();
        let exits = self
            .introspector
            .exits(&view)
            .value
            .expect("the exits are known");
        assert!(exits.unknown.is_empty());
        let mut out: Vec<(String, PlaceState, Option<String>)> = exits
            .exits
            .iter()
            .map(|exit| {
                let place = PLACES.iter().find(|word| {
                    let cut: String = word.chars().take(6).collect();
                    exit.words.contains(&cut)
                });
                let name = match place {
                    Some(word) => word.to_string(),
                    None => exit.words.iter().max_by_key(|w| w.len()).unwrap().clone(),
                };
                match exit.kind {
                    ExitKind::Fixed => (
                        name,
                        PlaceState::Open,
                        exit.destination_name.clone().and_then(Result::ok),
                    ),
                    ExitKind::Conditional => (name, PlaceState::Maybe, None),
                }
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

/// What the session's compass lights, in the same form.
fn lit(session: &Session) -> Vec<(String, PlaceState, Option<String>)> {
    let compass = session.compass();
    let mut out: Vec<(String, PlaceState, Option<String>)> = compass
        .places
        .iter()
        .filter(|p| matches!(p.state, PlaceState::Open | PlaceState::Maybe))
        .map(|p| (p.place.id().to_string(), p.state, p.leads_to.clone()))
        .chain(
            compass
                .extras
                .iter()
                .map(|e| (e.command.clone(), e.state, e.leads_to.clone())),
        )
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Sends a command as a tap if the compass offers it, and otherwise types it.
/// Returns whether it went as a tap.
fn tap_or_type(session: &mut Session, command: &str) -> bool {
    let at = session.stamp();
    if let Some(place) = Place::from_id(command)
        && session.tap(at, place).is_ok()
    {
        return true;
    }
    if let Some(index) = session
        .compass()
        .extras
        .iter()
        .position(|e| e.command == command)
    {
        session.tap_extra(at, index).unwrap();
        return true;
    }
    session.send_line(at, command).unwrap();
    false
}

// R14, FR-031, SC-004, SC-006: the walkthrough with a tap wherever the compass
// offered the move. The text is the reference's, byte for byte; at least 150
// commands go as taps; and every tap's command is in the turn it produced.
#[test]
fn the_walkthrough_by_taps_gives_the_reference_text() {
    let commands = commands();
    let mut session = game(SEED);
    let mut taps = 0;
    let mut typed_directions = Vec::new();
    for (n, command) in commands.iter().enumerate() {
        let was_tap = tap_or_type(&mut session, command);
        taps += was_tap as usize;
        let turn = session.turns().last().unwrap();
        assert_eq!(
            turn.command.as_deref(),
            Some(command.as_str()),
            "what was sent is what is shown"
        );
        assert_eq!(
            turn.source,
            Some(if was_tap { Source::Tap } else { Source::Typed })
        );
        if !was_tap && (PLACES.contains(&command.as_str()) || command == "land") {
            typed_directions.push((n + 3, command.clone()));
        }
    }
    println!(
        "{taps} of {} commands went as taps; directions that had to be typed: {typed_directions:?}",
        commands.len()
    );
    assert_same(
        &shown(&session),
        &reference_from_first_command(&commands[0]),
    );
    assert!(taps >= 150, "SC-004 asks for at least 150 taps: {taps}");
    // Recorded: the taps, and the direction commands the compass did not offer.
    assert_eq!(taps, 186);
    // `east` in the Drafty Room and `west` in the Stone Barrow: the game takes
    // them, but neither room's table lists them as an exit.
    assert_eq!(
        typed_directions,
        [(250, "east".to_string()), (332, "west".to_string())]
    );
}

// SC-005, FR-008, research R6: at every stop of the same replay, the compass is
// what the introspection interface reports for an identical game kept beside
// it; and the layer's idea of a command prompt is checked against the game's
// own text.
#[test]
fn the_compass_agrees_with_an_identical_game_at_every_prompt() {
    let commands = commands();
    let mut session = game(SEED);
    let mut shadow = Shadow::new();
    let mut compared = 0;
    let mut not_command = Vec::new();
    for (n, command) in commands.iter().enumerate() {
        // The state before this command is given: walkthrough line n + 3.
        let prompt = shadow.text.ends_with("\n\n>");
        match session.waiting() {
            Waiting::Command => {
                assert!(
                    prompt,
                    "line {}: the layer says command, the text ends {:?}",
                    n + 3,
                    &shadow.text[shadow.text.len().saturating_sub(30)..]
                );
                assert_eq!(lit(&session), shadow.exits(), "line {} ({command})", n + 3);
                assert!(session.compass().offered && session.compass().known);
                compared += 1;
            }
            other => {
                assert!(!session.compass().offered, "line {}", n + 3);
                if prompt {
                    // An ordinary prompt by its text that the layer does not call
                    // a command prompt: research R6 expects exactly one.
                    not_command.push((n + 3, other, session.status().map(|s| s.place.clone())));
                }
            }
        }
        tap_or_type(&mut session, command);
        shadow.give(command);
        assert_eq!(
            session.turns().last().unwrap().text,
            shadow.text,
            "line {}: the two games have parted",
            n + 3
        );
    }
    println!(
        "compass compared at {compared} command prompts; ordinary prompts not called commands: {not_command:?}"
    );
    assert!(compared > 300, "SC-005 asks for more than 300: {compared}");
    assert_eq!(not_command.len(), 1, "{not_command:?}");
    assert_eq!(not_command[0].2.as_deref(), Some("Loud Room"));
}

// SC-005: at the walkthrough lines that have hand-made exit lists, written from
// the game's source, the compass agrees with those.
#[test]
fn the_compass_agrees_with_the_hand_made_exit_lists() {
    // (line, room, [(direction, "fixed <room>" or "conditional")])
    let mut entries: Vec<(usize, String, Vec<(String, String)>)> = Vec::new();
    for raw in read_text("stories/zork1/introspection/exits.txt").lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("at line ") {
            let (at, room) = rest.split_once(" | ").unwrap();
            entries.push((
                at.trim().parse().unwrap(),
                room.trim().to_string(),
                Vec::new(),
            ));
        } else {
            let (direction, what) = line.trim().split_once(':').unwrap();
            entries
                .last_mut()
                .unwrap()
                .2
                .push((direction.trim().to_string(), what.trim().to_string()));
        }
    }
    assert!(entries.len() >= 50);
    let commands = commands();
    let mut session = game(SEED);
    let mut checked = 0;
    let mut at_questions = Vec::new();
    for (n, command) in commands.iter().enumerate() {
        for (line, room, expected) in entries.iter().filter(|e| e.0 == n + 3) {
            if session.waiting() != Waiting::Command {
                at_questions.push(*line);
                continue;
            }
            assert_eq!(&session.status().unwrap().place, room, "line {line}");
            let mut want: Vec<(String, PlaceState, Option<String>)> = expected
                .iter()
                .map(|(direction, what)| match what.strip_prefix("fixed ") {
                    Some(to) => (direction.clone(), PlaceState::Open, Some(to.to_string())),
                    None => (direction.clone(), PlaceState::Maybe, None),
                })
                .collect();
            want.sort_by(|a, b| a.0.cmp(&b.0));
            assert_eq!(lit(&session), want, "line {line} ({room})");
            checked += 1;
        }
        tap_or_type(&mut session, command);
    }
    println!(
        "{checked} hand-made entries checked; entries at a stop that is not a command prompt: {at_questions:?}"
    );
    assert!(checked >= 55, "{checked}");
    assert!(at_questions.len() <= 1, "{at_questions:?}");
}
