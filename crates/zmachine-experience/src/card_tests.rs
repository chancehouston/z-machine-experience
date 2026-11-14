//! The peek, on stories built in memory
//! (specs/014-inventory-card/contracts/card.md, L§4).

use crate::card::peek;
use crate::lookahead;
use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Story, zencode};

fn print(text: &str) -> Vec<u8> {
    [op::op0(op::PRINT), zencode(text)].concat()
}

/// A story that prints `>`, reads a line, then runs `after`, which is given a
/// second read.
fn story(after: impl FnOnce(Vec<u8>) -> Vec<u8>) -> Vec<u8> {
    let mut story = Story::new();
    story.word("look");
    let mut text = vec![0u8; 64];
    text[0] = 60;
    let text = story.scratch(&text);
    let mut parse = vec![0u8; 22];
    parse[0] = 5;
    let parse = story.scratch(&parse);
    let read = op::var(op::SREAD, &[L(text), L(parse)]);
    // Round again to the first read, whatever follows.
    let body = [print(">"), read.clone(), after(read)].concat();
    let back = -(body.len() as i16 + 3) + 2;
    story.main(&[body, op::op1(op::JUMP, L(back as u16))].concat());
    story.build()
}

/// The story at its first prompt, and that prompt's address.
fn at_prompt(image: Vec<u8>) -> (Machine, u32) {
    let options = LoadOptions {
        seed: Some(1),
        entropy: 0,
    };
    let mut machine = Machine::load(image, options).unwrap();
    assert!(matches!(
        machine.run(1_000_000).unwrap(),
        Event::LineWanted { .. }
    ));
    machine.take_output();
    let address = machine.waiting_at().unwrap();
    (machine, address)
}

const LIMIT: u64 = lookahead::LIMITS.copy;

// L§4: the copy's reply, without the prompt and the white space round it.
#[test]
fn a_peek_gives_what_the_copy_printed() {
    let (machine, address) = at_prompt(story(|_| print("A brass lamp.\n\n")));
    assert_eq!(
        peek(&machine, "examine lamp", address, LIMIT).as_deref(),
        Some("A brass lamp.")
    );
    // The machine itself has run nothing: it still wants its line.
    assert_eq!(machine.waiting_at(), Some(address));
    assert_eq!(
        peek(&machine, "examine lamp", address, LIMIT).as_deref(),
        Some("A brass lamp.")
    );
}

// L§4: a copy that stops anywhere but the command prompt gives nothing.
#[test]
fn a_peek_that_does_not_come_back_gives_nothing() {
    // A question, from another read.
    let (machine, address) = at_prompt(story(|read| [print("Which? >"), read].concat()));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // The end of the game.
    let (machine, address) = at_prompt(story(|_| [print("Bye."), op::op0(op::QUIT)].concat()));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // A fault.
    let (machine, address) = at_prompt(story(|_| {
        [print("Oh. "), op::op2(op::DIV, S(1), S(0)), op::store(16)].concat()
    }));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // A copy that never stops: the limit.
    let (machine, address) = at_prompt(story(|_| {
        let nop = op::op0(op::NOP);
        let back = -(nop.len() as i16 + 3) + 2;
        [nop, op::op1(op::JUMP, L(back as u16))].concat()
    }));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
    // A reply of nothing at all describes nothing.
    let (machine, address) = at_prompt(story(|_| Vec::new()));
    assert_eq!(peek(&machine, "examine lamp", address, LIMIT), None);
}
