//! Output in versions 5 and 8: styles, the memory stream, and the instructions
//! that are accepted without effect (feature 002, T026; §7.1.2, §8.7.1, §15).

mod common;

use common::*;
use zmachine_core::{Event, FaultKind, Machine, Style};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::Story5;

fn style(n: u8) -> Vec<u8> {
    op::var(op::SET_TEXT_STYLE, &[S(n)])
}

fn stream(n: i16) -> Vec<u8> {
    op::var(op::OUTPUT_STREAM, &[L(n as u16)])
}

fn stream3(table: u16) -> Vec<u8> {
    op::var(op::OUTPUT_STREAM, &[S(3), L(table)])
}

const PLAIN: Style = Style { reverse: false, bold: false, italic: false, fixed_pitch: false };
const BOLD: Style = Style { bold: true, ..PLAIN };

fn runs_of(code: Vec<u8>) -> Vec<(String, Style)> {
    let mut story = Story5::new(5);
    story.main(&cat(&[code, quit()]));
    let mut m = load5(&mut story);
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    m.take_styled_output().into_iter().map(|r| (r.text, r.style)).collect()
}

// §8.7.1, §15 set_text_style: 1 reverse, 2 bold, 4 italic, 8 fixed pitch; 0 turns
// them all off. contracts/core-api.md: each run carries the style it was printed in
// and adjacent text in one style is one run.
#[test]
fn styled_runs_carry_the_style_in_force() {
    let runs = runs_of(cat(&[
        print("plain "),
        print("still plain "),
        style(2),
        print("bold"),
        style(0),
        print(" "),
        style(4),
        print("italic"),
        style(0),
        style(1),
        print("reverse"),
        style(0),
        style(8),
        print("fixed"),
        style(0),
    ]));
    assert_eq!(
        runs,
        vec![
            ("plain still plain ".to_string(), PLAIN),
            ("bold".to_string(), BOLD),
            (" ".to_string(), PLAIN),
            ("italic".to_string(), Style { italic: true, ..PLAIN }),
            ("reverse".to_string(), Style { reverse: true, ..PLAIN }),
            ("fixed".to_string(), Style { fixed_pitch: true, ..PLAIN }),
            ("\n".to_string(), PLAIN),
        ]
    );
}

// §15 set_text_style, as revised in Standard 1.1: styles accumulate until 0.
#[test]
fn styles_accumulate() {
    let runs = runs_of(cat(&[style(2), style(4), print("both"), style(1), print("three"), style(0)]));
    assert_eq!(runs[0], ("both".to_string(), Style { bold: true, italic: true, ..PLAIN }));
    assert_eq!(
        runs[1],
        ("three".to_string(), Style { bold: true, italic: true, reverse: true, fixed_pitch: false })
    );
    // One operand may name several styles at once.
    let runs = runs_of(cat(&[style(6), print("x"), style(0)]));
    assert_eq!(runs[0].1, Style { bold: true, italic: true, ..PLAIN });
}

// contracts/core-api.md: `take_output` and `take_styled_output` drain one buffer;
// the plain text is the same text.
#[test]
fn plain_output_is_the_same_text_without_styles() {
    let code = cat(&[print("a "), style(2), print("b"), style(0), print(" c")]);
    assert_eq!(out5(&code), "a b c\n");
    let mut story = Story5::new(5);
    story.main(&cat(&[code, quit()]));
    let mut m = load5(&mut story);
    m.run(BIG).unwrap();
    // "a ", "b" in bold, and " c" with the line feed that ends the last line.
    assert_eq!(m.take_styled_output().len(), 3);
    assert_eq!(m.take_output(), "", "the buffer was drained by the styled call");
}

// §8.1: while Flags 2 bit 1 is set, text is fixed pitch whatever the style.
#[test]
fn flags_2_bit_1_forces_fixed_pitch() {
    let runs = runs_of(cat(&[
        print("a"),
        op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
        print("b"),
        op::var(op::STOREB, &[S(0), S(0x11), S(0)]),
        print("c"),
    ]));
    assert_eq!(
        runs[..3],
        [
            ("a".to_string(), PLAIN),
            ("b".to_string(), Style { fixed_pitch: true, ..PLAIN }),
            ("c\n".to_string(), PLAIN)
        ]
    );
}

/// Prints the word at `table`, then `count` bytes after it as numbers.
fn dump_table(table: u16, count: u8) -> Vec<u8> {
    let mut code = cat(&[to(op::op2(op::LOADW, L(table), S(0)), 0), print_num(V(0)), print(":")]);
    for i in 0..count {
        code.extend(to(op::op2(op::LOADB, L(table), S(2 + i)), 0));
        code.extend(print_num(V(0)));
        code.extend(print(" "));
    }
    code
}

// §7.1.2.1: output stream 3 writes text into a table: a word holding the number of
// characters, then the characters as ZSCII, a new-line as 13. §7.1.2.2: while it is
// selected nothing goes to the other streams.
#[test]
fn the_memory_stream_captures_text_into_a_table() {
    let mut story = Story5::new(5);
    let table = story.reserve(20);
    story.main(&cat(&[
        print("a"),
        stream3(table),
        print("Hi\n"),
        print_num(S(7)),
        stream(-3),
        print("b "),
        dump_table(table, 4),
        quit(),
    ]));
    let hi = format!("{} {} 13 {} ", b'H', b'i', b'7');
    assert_eq!(output_of_story5(&mut story), format!("ab 4:{hi}\n"));
}

// §7.1.2.1.1: memory streams nest, the innermost receiving the text, to a depth of
// sixteen; a seventeenth is an error.
#[test]
fn memory_streams_nest_to_sixteen() {
    let mut story = Story5::new(5);
    let outer = story.reserve(12);
    let inner = story.reserve(12);
    story.main(&cat(&[
        stream3(outer),
        print("ab"),
        stream3(inner),
        print("xyz"),
        stream(-3),
        print("c"),
        stream(-3),
        dump_table(outer, 3),
        dump_table(inner, 3),
        quit(),
    ]));
    assert_eq!(
        output_of_story5(&mut story),
        format!("3:{} {} {} 3:{} {} {} \n", b'a', b'b', b'c', b'x', b'y', b'z')
    );

    let nest = |depth: usize| {
        let mut story = Story5::new(5);
        let table = story.reserve(8);
        let mut code = vec![];
        for _ in 0..depth {
            code.extend(stream3(table));
        }
        for _ in 0..depth {
            code.extend(stream(-3));
        }
        code.extend(print("ok"));
        story.main(&cat(&[code, quit()]));
        let mut m = load5(&mut story);
        m.run(BIG).unwrap()
    };
    assert_eq!(nest(16), Event::Finished);
    assert!(matches!(nest(17), Event::Fault(f) if f.kind == FaultKind::StreamNesting));
}

// data-model.md: closing a memory stream when none is open does nothing; a table
// outside dynamic memory is a fault when the stream is closed and written.
#[test]
fn memory_stream_edge_cases() {
    assert_eq!(out5(&cat(&[stream(-3), stream(-3), print("fine")])), "fine\n");

    let mut story = Story5::new(5);
    let base = story.static_base();
    story.main(&cat(&[stream3(base), print("x"), stream(-3), quit()]));
    let mut m = load5(&mut story);
    assert!(matches!(m.run(BIG), Ok(Event::Fault(f)) if f.kind == FaultKind::WriteOutsideDynamicMemory));
}

// §7.1.2.1: a character with no ZSCII code is stored as a question mark; the
// default extra characters keep their codes (praxix's `streamtrip` test).
#[test]
fn the_memory_stream_stores_zscii() {
    let mut story = Story5::new(5);
    let table = story.reserve(12);
    story.main(&cat(&[
        stream3(table),
        op::var(op::PRINT_CHAR, &[S(155)]),
        op::var(op::PRINT_CHAR, &[S(223)]),
        op::ext(op::PRINT_UNICODE, &[L(0x4E2D)]),
        op::var(op::PRINT_CHAR, &[S(0)]),
        stream(-3),
        dump_table(table, 3),
        quit(),
    ]));
    assert_eq!(output_of_story5(&mut story), format!("3:155 223 {} \n", b'?'));
}

// Guarantee 6: version 3 has no memory stream; `output_stream 3` stays ignored.
#[test]
fn version_3_still_ignores_stream_3() {
    let mut story = zmachine_testkit::Story::new();
    story.main(&cat(&[stream(3), print("shown"), stream(-3), quit()]));
    assert_eq!(output_of_story(&mut story), "shown\n");
}

// §3.8.5.4, §15 print_unicode: prints the character with that Unicode value.
// §15 check_unicode: bit 0 set if it can be printed, bit 1 if it can be read
// (data-model.md: any character can be printed; one with a ZSCII code can be read).
#[test]
fn print_unicode_and_check_unicode() {
    let check = |c: u16| cat(&[to(op::ext(op::CHECK_UNICODE, &[L(c)]), g(0)), show(0)]);
    let code = cat(&[
        op::ext(op::PRINT_UNICODE, &[L(0x00E9)]),
        op::ext(op::PRINT_UNICODE, &[L(0x03B1)]),
        print(" "),
        check(b'a' as u16),
        check(0x00E4),
        check(0x03B1),
    ]);
    assert_eq!(out5(&code), "\u{e9}\u{3b1} 3 3 1 \n");
}

// §15, data-model.md: instructions that are accepted with any operands and do
// nothing. praxix issues `set_colour` with colour numbers version 5 does not define
// and `set_true_colour` with -1.
#[test]
fn instructions_accepted_without_effect() {
    let mut code = vec![];
    for colour in [0u8, 1, 2, 9, 10, 12, 200] {
        code.extend(op::op2(op::SET_COLOUR, S(colour), S(colour)));
    }
    code.extend(op::ext(op::SET_TRUE_COLOUR, &[L(0xFFFF), L(0x7FFF)]));
    code.extend(op::var(op::SOUND_EFFECT, &[S(1)]));
    code.extend(op::var(op::SOUND_EFFECT, &[S(3), S(2), L(0x0108), L(0x1234)]));
    code.extend(op::var(op::INPUT_STREAM, &[S(1)]));
    code.extend(op::var(op::BUFFER_MODE, &[S(0)]));
    code.extend(op::var(op::BUFFER_MODE, &[S(1)]));
    // §15: show_status is illegal from version 4 but should be treated as nop.
    code.extend(op::op0(12));
    // §14.2.1: unspecified extended opcodes are ignored.
    code.extend(op::ext(200, &[S(1), S(2)]));
    code.extend(print("still here"));
    assert_eq!(out5(&code), "still here\n");
}

// §15 piracy: "Branches if the game disc is believed to be genuine": always.
#[test]
fn piracy_always_branches() {
    let code = cat(&[skip_if(op::op0(op::PIRACY), true, &print("pirated ")), print("genuine")]);
    assert_eq!(out5(&code), "genuine\n");
}

// §15 set_font: stores the previous font, or 0 if the font asked for is not
// available; font 0 reports the current font without changing it. Only font 1 is
// offered (contracts/screen-model.md).
#[test]
fn set_font() {
    let font = |n: u8| cat(&[to(op::ext(op::SET_FONT, &[S(n)]), g(0)), show(0)]);
    assert_eq!(out5(&cat(&[font(1), font(0), font(4), font(3), font(2), font(1)])), "1 1 0 0 0 1 \n");
}

// §11.1: the header tells the game what is offered: bold, italic and fixed pitch
// yes; colour and timed input no.
#[test]
fn the_header_reports_what_is_available() {
    let code = cat(&[to(op::op2(op::LOADB, S(0), S(1)), 0), print_num(V(0))]);
    assert_eq!(out5(&code), format!("{}\n", 0b0001_1100));
}

// The line feed that ends an unfinished last line (rule 11) is printed like any
// other text, in the style in force.
#[test]
fn the_exit_line_feed_takes_the_current_style() {
    let runs = runs_of(cat(&[style(2), print("bold")]));
    assert_eq!(runs, vec![("bold\n".to_string(), BOLD)]);
    let _ = Machine::load;
}
