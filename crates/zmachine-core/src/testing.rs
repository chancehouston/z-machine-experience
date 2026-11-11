//! Helpers shared by this crate's unit tests.

use crate::{Event, LoadOptions, Machine};
use zmachine_testkit::{Story, op};

pub(crate) const BIG: u64 = 1_000_000;
pub(crate) const OPTIONS: LoadOptions = LoadOptions {
    seed: Some(1),
    entropy: 0,
};

pub(crate) fn load(story: &mut Story) -> Machine {
    Machine::load(story.build(), OPTIONS).expect("test story loads")
}

/// Builds the story, runs it and returns the machine with its first event.
pub(crate) fn run_story(story: &mut Story) -> (Machine, Event) {
    let mut machine = load(story);
    let event = machine.run(BIG).expect("first run is in turn");
    (machine, event)
}

/// Runs `code` followed by `quit` as the main code of an otherwise empty story.
pub(crate) fn run(code: &[u8]) -> (Machine, Event) {
    let mut story = Story::new();
    let mut main = code.to_vec();
    main.extend(op::op0(op::QUIT));
    story.main(&main);
    run_story(&mut story)
}

/// The variable number of global `n` (§4.2.2).
pub(crate) const fn g(n: u8) -> u8 {
    16 + n
}

/// Appends `instruction` with branch data that skips `skipped` when the condition
/// equals `on_true`, then `skipped` itself.
pub(crate) fn skip_if(instruction: Vec<u8>, on_true: bool, skipped: &[u8]) -> Vec<u8> {
    let mut out = instruction;
    out.extend(op::branch(on_true, skipped.len() as i16 + 2));
    out.extend_from_slice(skipped);
    out
}
