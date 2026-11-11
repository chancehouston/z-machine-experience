//! Helpers shared by the integration tests. Everything here goes through the
//! public API only.
#![allow(dead_code)]

use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_testkit::op::{self, Arg, Arg::*};
use zmachine_testkit::{Object, Story, zencode};

pub const BIG: u64 = 1_000_000;
pub const OPTIONS: LoadOptions = LoadOptions {
    seed: Some(1),
    entropy: 0,
};

pub fn load(story: &mut Story) -> Machine {
    Machine::load(story.build(), OPTIONS).expect("test story loads")
}

pub fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

/// Variable number of global `n`.
pub const fn g(n: u8) -> u8 {
    16 + n
}

/// `print "text"`
pub fn print(text: &str) -> Vec<u8> {
    cat(&[op::op0(op::PRINT), zencode(text)])
}

/// `print_num value`
pub fn print_num(value: Arg) -> Vec<u8> {
    op::var(op::PRINT_NUM, &[value])
}

pub fn quit() -> Vec<u8> {
    op::op0(op::QUIT)
}

/// `instruction` with branch data that skips `skipped` when the condition equals
/// `on_true`, then `skipped`.
pub fn skip_if(instruction: Vec<u8>, on_true: bool, skipped: &[u8]) -> Vec<u8> {
    cat(&[
        instruction,
        op::branch(on_true, skipped.len() as i16 + 2),
        skipped.to_vec(),
    ])
}

/// A story with a location object named "Room" in the first global, a text buffer
/// accepting `max_chars` characters and a parse buffer for five words. Returns the
/// story and the addresses of the two buffers. The dictionary holds "look" and
/// "north".
pub fn story_with_buffers(max_chars: u8) -> (Story, u16, u16) {
    let mut story = Story::new();
    let room = story.object(Object::new("Room"));
    story.global(0, room as u16);
    story.word("look").word("north");
    let mut text = vec![0u8; 256];
    text[0] = max_chars + 1;
    let text = story.scratch(&text);
    let mut parse = vec![0u8; 2 + 4 * 5];
    parse[0] = 5;
    let parse = story.scratch(&parse);
    (story, text, parse)
}

/// `sread text parse`
pub fn sread(text: u16, parse: u16) -> Vec<u8> {
    op::var(op::SREAD, &[L(text), L(parse)])
}

/// Runs to completion, answering nothing: collects every event until the machine
/// stops or needs an answer. `BudgetExhausted` events are skipped.
pub fn run_until_event(machine: &mut Machine, budget: u64) -> Event {
    loop {
        match machine.run(budget).expect("run in turn") {
            Event::BudgetExhausted => continue,
            other => return other,
        }
    }
}

/// Runs `code` followed by `quit` as main code and returns everything printed.
pub fn output_of(code: &[u8]) -> String {
    let mut story = Story::new();
    story.main(&cat(&[code.to_vec(), quit()]));
    output_of_story(&mut story)
}

pub fn output_of_story(story: &mut Story) -> String {
    let mut machine = load(story);
    assert_eq!(run_until_event(&mut machine, BIG), Event::Finished);
    machine.take_output()
}
