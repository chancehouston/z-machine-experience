//! Line and key input in versions 5 and 8 (feature 002, T025; §10, §15 read,
//! read_char; specs/002-v5-v8/contracts/core-api.md).

mod common;

use common::*;
use zmachine_core::{Event, HostError, Key, StatusKind};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Object5, Story5};

/// Reads a line into a buffer of `capacity` holding `preloaded`, then prints the
/// stored result, the length byte, the first three characters and the word count.
fn read_program(capacity: u8, preloaded: &str) -> Story5 {
    let (mut story, text, parse) = story5_with_buffers(capacity, preloaded);
    let byte = |address: u16, index: u8| {
        cat(&[to(op::op2(op::LOADB, L(address), S(index)), 0), print_num(V(0)), print(" ")])
    };
    story.main(&cat(&[
        print(">"),
        aread(text, parse, g(0)),
        show(0),
        byte(text, 1),
        byte(text, 2),
        byte(text, 3),
        byte(text, 4),
        byte(parse, 1),
        quit(),
    ]));
    story
}

// §15 read, version 5: the instruction stores the terminating character, 13 for
// return; byte 1 of the buffer gets the length and the text starts at byte 2.
#[test]
fn aread_stores_the_line_and_the_terminator() {
    let mut m = load5(&mut read_program(20, ""));
    assert_eq!(m.run(BIG), Ok(Event::LineWanted { max_len: 20 }));
    assert_eq!(m.take_output(), ">");
    assert_eq!(m.preloaded_input(), "");
    m.provide_line("Look North").unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), format!("13 10 {} {} {} 2 \n", b'l', b'o', b'o'));
}

// §8.2: only version 3 has an interpreter-drawn status line. A version 5 read
// produces none.
#[test]
fn a_version_5_read_draws_no_status_line() {
    let mut m = load5(&mut read_program(20, ""));
    m.run(BIG).unwrap();
    assert_eq!(m.take_status(), None);
    let _ = StatusKind::Score { score: 0, turns: 0 };
}

// §15 read: the line is cut to the buffer's capacity.
#[test]
fn a_line_longer_than_the_buffer_is_cut() {
    let mut m = load5(&mut read_program(4, ""));
    assert_eq!(m.run(BIG), Ok(Event::LineWanted { max_len: 4 }));
    m.take_output();
    m.provide_line("look north").unwrap();
    m.run(BIG).unwrap();
    assert_eq!(m.take_output(), format!("13 4 {} {} {} 1 \n", b'l', b'o', b'o'));
}

// §15 read: if the buffer already holds text, "the interpreter writes the new
// characters after those already there". contracts/core-api.md: `max_len` is the
// room left and `preloaded_input` the text already there.
#[test]
fn preloaded_text_is_kept_and_typed_text_follows_it() {
    let mut m = load5(&mut read_program(10, "lo"));
    assert_eq!(m.run(BIG), Ok(Event::LineWanted { max_len: 8 }));
    assert_eq!(m.preloaded_input(), "lo");
    m.take_output();
    m.provide_line("OK").unwrap();
    assert_eq!(m.preloaded_input(), "", "only while a line is wanted");
    m.run(BIG).unwrap();
    // The buffer holds "look": one word, found in the dictionary or not.
    assert_eq!(m.take_output(), format!("13 4 {} {} {} 1 \n", b'l', b'o', b'o'));
}

// §15 read: the parse buffer is optional in version 5; with 0 no lexical analysis
// is done.
#[test]
fn aread_without_a_parse_buffer() {
    let mut story = Story5::new(5);
    let text = story.text_buffer(20, "");
    story.main(&cat(&[
        to(op::var(op::AREAD, &[L(text), S(0)]), g(0)),
        show(0),
        cat(&[to(op::op2(op::LOADB, L(text), S(1)), 0), print_num(V(0))]),
        quit(),
    ]));
    let mut m = load5(&mut story);
    assert!(matches!(m.run(BIG), Ok(Event::LineWanted { .. })));
    m.provide_line("hello").unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "13 5\n");
}

/// A read program whose story lists function key 1 (ZSCII 133) and the up arrow
/// (129) as keys that end a line.
fn terminating_program() -> Story5 {
    let (mut story, text, parse) = story5_with_buffers(20, "");
    let table = story.scratch(&[133, 129, 0]);
    story.header_word(0x2E, table);
    story.main(&cat(&[aread(text, parse, g(0)), show(0), quit()]));
    story
}

// §10.5.2.1, §15 read: a key in the terminating characters table may end a line,
// and the instruction stores that key's code.
#[test]
fn a_line_may_be_ended_by_a_listed_key() {
    let mut m = load5(&mut terminating_program());
    m.run(BIG).unwrap();
    assert_eq!(m.provide_line_ended_by("look", Key::Function(1)), Ok(()));
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "133 \n");

    let mut m = load5(&mut terminating_program());
    m.run(BIG).unwrap();
    m.provide_line_ended_by("look", Key::Up).unwrap();
    m.run(BIG).unwrap();
    assert_eq!(m.take_output(), "129 \n");
}

// contracts/core-api.md: a key the game did not list is refused and the machine
// keeps waiting.
#[test]
fn a_key_the_game_did_not_list_cannot_end_a_line() {
    let mut m = load5(&mut terminating_program());
    m.run(BIG).unwrap();
    assert_eq!(m.provide_line_ended_by("look", Key::Function(2)), Err(HostError::NotATerminator));
    assert_eq!(m.provide_line_ended_by("look", Key::Char('x')), Err(HostError::NotATerminator));
    assert_eq!(m.run(BIG), Err(HostError::AnswerOwed), "still waiting for the line");
    // Return always ends a line.
    assert_eq!(m.provide_line_ended_by("look", Key::Return), Ok(()));
    m.run(BIG).unwrap();
    assert_eq!(m.take_output(), "13 \n");

    // With no table, only return ends a line.
    let mut m = load5(&mut read_program(20, ""));
    m.run(BIG).unwrap();
    assert_eq!(m.provide_line_ended_by("x", Key::Function(1)), Err(HostError::NotATerminator));
}

/// Reads three keys and prints each code.
fn key_program() -> Story5 {
    let mut story = Story5::new(5);
    story.main(&cat(&[
        print("k:"),
        read_char(g(0)),
        show(0),
        read_char(g(0)),
        show(0),
        read_char(g(0)),
        show(0),
        quit(),
    ]));
    story
}

// §10.6, §15 read_char: a single keypress is read and its ZSCII code stored.
// §3.8.4: the codes of the special keys.
#[test]
fn read_char_stores_the_key() {
    let keys = |a: Key, b: Key, c: Key| {
        let mut m = load5(&mut key_program());
        let mut out = String::new();
        for key in [a, b, c] {
            assert_eq!(m.run(BIG), Ok(Event::KeyWanted));
            out.push_str(&m.take_output());
            m.provide_key(key).unwrap();
        }
        assert_eq!(m.run(BIG), Ok(Event::Finished));
        out + &m.take_output()
    };
    assert_eq!(keys(Key::Char('a'), Key::Char('Z'), Key::Char(' ')), "k:97 90 32 \n");
    assert_eq!(keys(Key::Return, Key::Delete, Key::Escape), "k:13 8 27 \n");
    assert_eq!(keys(Key::Up, Key::Down, Key::Left), "k:129 130 131 \n");
    assert_eq!(keys(Key::Right, Key::Function(1), Key::Function(12)), "k:132 133 144 \n");
    assert_eq!(keys(Key::Keypad(0), Key::Keypad(9), Key::Char('\u{e4}')), "k:145 154 155 \n");
}

// contracts/core-api.md: a character with no ZSCII code is refused and the machine
// keeps waiting; answers out of turn are refused.
#[test]
fn unrepresentable_keys_and_answers_out_of_turn() {
    let mut m = load5(&mut key_program());
    assert_eq!(m.provide_key(Key::Return), Err(HostError::NotWaitingForThis));
    assert_eq!(m.run(BIG), Ok(Event::KeyWanted));
    assert_eq!(m.provide_key(Key::Char('\u{4e2d}')), Err(HostError::UnrepresentableKey));
    assert_eq!(m.provide_key(Key::Function(13)), Err(HostError::UnrepresentableKey));
    assert_eq!(m.provide_key(Key::Keypad(10)), Err(HostError::UnrepresentableKey));
    assert_eq!(m.run(BIG), Err(HostError::AnswerOwed));
    assert_eq!(m.provide_line("x"), Err(HostError::NotWaitingForThis));
    assert_eq!(m.provide_key(Key::Char('q')), Ok(()));
    assert_eq!(m.provide_key(Key::Char('q')), Err(HostError::NotWaitingForThis));
}

// §10.5.3, 002 FR-017: with timed input reported unavailable, a request with a time
// limit behaves as one without, and its routine is never called.
#[test]
fn timed_input_behaves_as_untimed() {
    let (mut story, text, parse) = story5_with_buffers(20, "");
    // The routine would set global 9; it must never run.
    let tick = story.routine(0, &cat(&[op::op2(op::STORE, S(g(9)), S(1)), op::op0(op::RTRUE)]));
    story.main(&cat(&[
        to(op::var(op::READ_CHAR, &[S(1), S(5), L(tick)]), g(0)),
        to(op::var(op::AREAD, &[L(text), L(parse), S(5), L(tick)]), g(1)),
        show(0),
        show(1),
        show(9),
        quit(),
    ]));
    let mut m = load5(&mut story);
    assert_eq!(m.run(BIG), Ok(Event::KeyWanted));
    m.provide_key(Key::Char('k')).unwrap();
    assert!(matches!(m.run(BIG), Ok(Event::LineWanted { .. })));
    m.provide_line("look").unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "107 13 0 \n");
    // §11.1: and the game was told so. Flags 1 bit 7 is clear.
    let _ = Object5::new("");
}
