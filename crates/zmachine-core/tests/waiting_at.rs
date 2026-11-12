//! Where a game is waiting (feature 004, T003; specs/004-first-demo/research.md
//! R6; §15 sread, read, read_char). Each request for input is made by one
//! instruction; `waiting_at` gives its address.

mod common;

use common::*;
use zmachine_core::{Event, Key, LoadOptions, Machine};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Story, Story5};

const ZORK1: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");

fn zork() -> Machine {
    let story = std::fs::read(ZORK1).unwrap();
    Machine::load(story, LoadOptions { seed: Some(11), entropy: 0 }).unwrap()
}

fn to_prompt(m: &mut Machine) -> Event {
    let event = run_until_event(m, BIG);
    m.take_output();
    event
}

// R6: nothing is waiting before the game has run, after it has finished, and
// after a fault.
#[test]
fn nothing_is_waiting_when_the_game_is_not_stopped_for_input() {
    let mut story = Story::new();
    story.main(&cat(&[print("done"), quit()]));
    let mut m = load(&mut story);
    assert_eq!(m.waiting_at(), None, "before the first run");
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    assert_eq!(m.waiting_at(), None, "after the game has finished");

    let mut bad = Story5::new(5);
    bad.main(&cat(&[to(op::op2(op::DIV, S(1), S(0)), g(0)), quit()]));
    let mut m = load5(&mut bad);
    assert!(matches!(run_until_event(&mut m, BIG), Event::Fault(_)));
    assert_eq!(m.waiting_at(), None, "after a fault");
}

// §15 sread: while a line is wanted, the address is that of the read instruction.
// Two reads at different places give different addresses, and the same read
// reached again gives the same one.
#[test]
fn a_line_request_reports_the_read_that_made_it() {
    let (mut story, text, parse) = story_with_buffers(20);
    // read A; read B; then jump back to read A.
    let first = sread(text, parse);
    let second = sread(text, parse);
    let back = -((first.len() + second.len() + 3) as i16) + 2;
    story.main(&cat(&[first.clone(), second, op::op1(op::JUMP, L(back as u16))]));
    let image = story.build();
    let start = u16::from_be_bytes([image[6], image[7]]) as u32;
    let mut m = Machine::load(image.clone(), OPTIONS).unwrap();

    assert!(matches!(run_until_event(&mut m, BIG), Event::LineWanted { .. }));
    let a = m.waiting_at().expect("a line is wanted");
    assert_eq!(a, start, "the first read is the first instruction");
    // VAR form of opcode 4 (sread): 0xE4.
    assert_eq!(image[a as usize], 0xE4);
    m.provide_line("look").unwrap();
    assert_eq!(m.waiting_at(), None, "answered: no longer waiting");

    assert!(matches!(run_until_event(&mut m, BIG), Event::LineWanted { .. }));
    let b = m.waiting_at().unwrap();
    assert_eq!(b, start + first.len() as u32);
    assert_ne!(a, b);
    m.provide_line("look").unwrap();

    assert!(matches!(run_until_event(&mut m, BIG), Event::LineWanted { .. }));
    assert_eq!(m.waiting_at(), Some(a), "the first read, reached again");
}

// §15 read_char: a key request reports its instruction too; and nothing is
// reported while the host owes some other answer.
#[test]
fn a_key_request_reports_its_instruction_and_other_stops_report_nothing() {
    let mut story = Story5::new(5);
    let key = read_char(g(0));
    story.main(&cat(&[key.clone(), to(op::ext(op::EXT_SAVE, &[]), g(1)), quit()]));
    let image = story.build();
    let start = u16::from_be_bytes([image[6], image[7]]) as u32;
    let mut m = Machine::load(image.clone(), OPTIONS).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::KeyWanted);
    assert_eq!(m.waiting_at(), Some(start));
    // VAR form of opcode 22 (read_char): 0xF6.
    assert_eq!(image[start as usize], 0xF6);
    m.provide_key(Key::Return).unwrap();
    assert!(matches!(run_until_event(&mut m, BIG), Event::SaveRequested(_)));
    assert_eq!(m.waiting_at(), None, "a save is not a request for input");
    m.save_result(false).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
}

// R6, on the real game: every ordinary command prompt of Zork I waits at one
// instruction, and its yes-or-no question waits at another.
#[test]
fn zork_asks_for_commands_and_for_answers_from_different_places() {
    let mut m = zork();
    assert!(matches!(to_prompt(&mut m), Event::LineWanted { .. }));
    let command = m.waiting_at().expect("the opening prompt");
    for line in ["north", "look", "inventory"] {
        m.provide_line(line).unwrap();
        assert!(matches!(to_prompt(&mut m), Event::LineWanted { .. }));
        assert_eq!(m.waiting_at(), Some(command), "after {line}");
    }
    m.provide_line("quit").unwrap();
    assert!(matches!(to_prompt(&mut m), Event::LineWanted { .. }));
    let question = m.waiting_at().expect("the question");
    assert_ne!(question, command, "the question is asked from elsewhere");
    m.provide_line("n").unwrap();
    assert!(matches!(to_prompt(&mut m), Event::LineWanted { .. }));
    assert_eq!(m.waiting_at(), Some(command), "back at the command prompt");
}

// The accessor reads and does not disturb: a clone reports the same address, and
// neither a view nor taking output changes it.
#[test]
fn asking_where_the_game_waits_changes_nothing() {
    let mut m = zork();
    run_until_event(&mut m, BIG);
    let at = m.waiting_at();
    assert!(at.is_some());
    assert_eq!(m.clone().waiting_at(), at);
    let _ = m.view().len();
    m.take_output();
    m.take_status();
    assert_eq!(m.waiting_at(), at);
    assert_eq!(m.waiting_at(), at);
    // The transcript request is not a request for input.
    m.provide_line("script").unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::TranscriptRequested);
    assert_eq!(m.waiting_at(), None);
}
