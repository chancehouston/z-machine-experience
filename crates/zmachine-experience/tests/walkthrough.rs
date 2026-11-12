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
    let at = reference.find(&marker).expect("the first command is in the reference");
    reference[at + 1..].to_string()
}

/// What a terminal would have shown from the first command on: the prompt, then
/// for each turn the command, a line feed, and what the game printed.
fn shown(session: &Session) -> String {
    let mut text = String::from(">");
    for turn in &session.turns()[1..] {
        text.push_str(turn.command.as_deref().expect("every turn after the opening has a command"));
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
    let at = ours.bytes().zip(reference.bytes()).position(|(a, b)| a != b).unwrap_or(ours.len().min(reference.len()));
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
    assert!(session.turns()[0].text.ends_with('>'), "the opening ends with the prompt");
    for command in &commands {
        say(&mut session, command);
    }
    assert_eq!(session.waiting(), Waiting::Ended, "the walkthrough ends by quitting");
    assert_eq!(session.turns().len(), 332);
    assert_same(&shown(&session), &reference_from_first_command(&commands[0]));
}
