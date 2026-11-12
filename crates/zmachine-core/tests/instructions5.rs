//! The instructions new in version 5 that compute, call and move data (feature 002,
//! T022 to T024). Programs are built for version 5 and report through `print_num`.

mod common;

use common::*;
use zmachine_core::{Event, FaultKind};
use zmachine_testkit::op::{self, Arg, Arg::*};
use zmachine_testkit::{SCRATCH, Story5, zencode_dict9};

/// A routine with `locals` locals that returns local 1 + 10 * local 2 + 100 * local 3.
fn weigh(story: &mut Story5, locals: usize) -> u16 {
    let code = cat(&[
        to(op::op2(op::MUL, V(2), S(10)), 0),
        to(op::op2(op::ADD, V(1), V(0)), 0),
        to(op::op2(op::MUL, V(3), S(100)), 5),
        to(op::op2(op::ADD, V(0), V(5)), 0),
        op::op1(op::RET, V(0)),
    ]);
    story.routine(locals, &code)
}

// §5.2.1: from version 5 a routine's locals start at zero. §6.4, §15: the storing
// call forms. §1.2.3: a version 5 packed address is the byte address over 4.
#[test]
fn storing_calls_pass_arguments_into_zeroed_locals() {
    let mut story = Story5::new(5);
    let r = weigh(&mut story, 5);
    story.main(&cat(&[
        to(op::op1(op::CALL_1S, L(r)), g(0)),
        to(op::op2(op::CALL_2S, L(r), S(7)), g(1)),
        to(op::var(op::CALL_VS, &[L(r), S(1), S(2), S(3)]), g(2)),
        to(
            op::var2(op::CALL_VS2, &[L(r), S(4), S(5), S(6), S(0), S(0)]),
            g(3),
        ),
        show(0),
        show(1),
        show(2),
        show(3),
        quit(),
    ]));
    assert_eq!(output_of_story5(&mut story), "0 7 321 654 \n");
}

// §4.4.3.1, §6.4.4: `call_vs2` and `call_vn2` carry up to seven arguments.
#[test]
fn seven_arguments() {
    let mut story = Story5::new(5);
    // Returns the sum of its seven locals.
    let mut body = vec![];
    body.extend(to(op::op2(op::ADD, V(1), V(2)), 0));
    for local in 3..=7 {
        body.extend(to(op::op2(op::ADD, V(0), V(local)), 0));
    }
    body.extend(op::op1(op::RET, V(0)));
    let sum = story.routine(7, &body);
    let store_sum = story.routine(
        7,
        &cat(&[to(op::op2(op::ADD, V(1), V(7)), g(9)), op::op0(op::RTRUE)]),
    );
    let args: Vec<Arg> = (1..=7).map(S).collect();
    let with = |r: u16| [vec![L(r)], args.clone()].concat();
    story.main(&cat(&[
        to(op::var2(op::CALL_VS2, &with(sum)), g(0)),
        op::var2(op::CALL_VN2, &with(store_sum)),
        show(0),
        show(9),
        quit(),
    ]));
    assert_eq!(output_of_story5(&mut story), "28 8 \n");
}

// §6.4.1, §15 call_1n, call_2n, call_vn, call_vn2: the result is thrown away, and
// nothing is left on the caller's stack.
#[test]
fn discarding_calls_store_nothing() {
    let mut story = Story5::new(5);
    let bump = story.routine(
        1,
        &cat(&[op::op1(op::INC, S(g(9))), op::op1(op::RET, S(99))]),
    );
    story.main(&cat(&[
        op::var(op::PUSH, &[S(77)]),
        op::op1(op::CALL_1N, L(bump)),
        op::op2(op::CALL_2N, L(bump), S(1)),
        op::var(op::CALL_VN, &[L(bump), S(1), S(2)]),
        op::var2(op::CALL_VN2, &[L(bump), S(1), S(2), S(3), S(4), S(5)]),
        show(9),
        op::var(op::PULL, &[S(g(0))]),
        show(0),
        quit(),
    ]));
    assert_eq!(output_of_story5(&mut story), "4 77 \n");
}

// §6.4.3: calling packed address 0 returns false, in every call form.
#[test]
fn calling_address_zero_returns_false() {
    let code = cat(&[
        op::op2(op::STORE, S(g(0)), S(5)),
        to(op::op1(op::CALL_1S, S(0)), g(0)),
        op::op1(op::CALL_1N, S(0)),
        show(0),
    ]);
    assert_eq!(out5(&code), "0 \n");
}

// §15 check_arg_count: "Branches if the given argument-number (counting from 1) has
// been provided by the routine call to the current routine." Observed with dfrotz
// (data-model.md): arguments beyond the routine's locals still count.
#[test]
fn check_arg_count_counts_every_argument_passed() {
    let mut story = Story5::new(5);
    let mut body = vec![];
    for n in 1..=5 {
        body.extend(skip_if(
            op::var(op::CHECK_ARG_COUNT, &[S(n)]),
            false,
            &print("y"),
        ));
    }
    body.extend(print("/"));
    body.extend(op::op0(op::RTRUE));
    let count = story.routine(3, &body);
    story.main(&cat(&[
        op::op1(op::CALL_1N, L(count)),
        op::op2(op::CALL_2N, L(count), S(1)),
        op::var(op::CALL_VN, &[L(count), S(1), S(2)]),
        op::var(op::CALL_VN, &[L(count), S(1), S(2), S(3)]),
        op::var2(op::CALL_VN2, &[L(count), S(1), S(2), S(3), S(4)]),
        op::var2(op::CALL_VN2, &[L(count), S(1), S(2), S(3), S(4), S(5)]),
        quit(),
    ]));
    assert_eq!(output_of_story5(&mut story), "/y/yy/yyy/yyyy/yyyyy/\n");
}

// §15 catch, throw; Quetzal §6.2: `catch` stores the number of routine calls in
// progress (1 in a routine called from the main code, observed with dfrotz), and
// `throw` returns a value from the frame so numbered, abandoning those above it.
#[test]
fn catch_and_throw() {
    let mut story = Story5::new(5);
    // c(frame): throw 55 frame
    let c = story.routine(
        1,
        &cat(&[op::op2(op::THROW, S(55), V(1)), op::op0(op::RTRUE)]),
    );
    // b(frame): c(frame), discarding; then return 2 (never reached)
    let b = story.routine(
        1,
        &cat(&[op::op2(op::CALL_2N, L(c), V(1)), op::op1(op::RET, S(2))]),
    );
    // a(): catch -> L1; g1 = L1; b(L1); return 1 (never reached)
    let a = story.routine(
        1,
        &cat(&[
            to(op::op0(op::CATCH), 1),
            op::op2(op::STORE, S(g(1)), V(1)),
            to(op::op2(op::CALL_2S, L(b), V(1)), g(2)),
            op::op1(op::RET, S(1)),
        ]),
    );
    // deeper(): return a(), so that a's catch is at depth 2.
    let deeper = story.routine(
        0,
        &cat(&[to(op::op1(op::CALL_1S, L(a)), 0), op::op0(op::RET_POPPED)]),
    );
    story.main(&cat(&[
        to(op::op0(op::CATCH), g(5)),
        to(op::op1(op::CALL_1S, L(a)), g(0)),
        show(5),
        show(1),
        show(0),
        show(2),
        to(op::op1(op::CALL_1S, L(deeper)), g(0)),
        show(1),
        show(0),
        quit(),
    ]));
    // catch in the main code is 0; in `a` it is 1; `a` returns the thrown 55; b's
    // storing call never completed, so global 2 is untouched. One call deeper the
    // catch is 2.
    assert_eq!(output_of_story5(&mut story), "0 1 55 0 2 55 \n");
}

// data-model.md: a throw to a frame that does not exist is a fault.
#[test]
fn throw_to_a_missing_frame_is_a_fault() {
    let mut story = Story5::new(5);
    let r = story.routine(
        0,
        &cat(&[op::op2(op::THROW, S(1), S(9)), op::op0(op::RTRUE)]),
    );
    story.main(&cat(&[op::op1(op::CALL_1N, L(r)), quit()]));
    let mut m = load5(&mut story);
    assert!(matches!(m.run(BIG), Ok(Event::Fault(f)) if f.kind == FaultKind::InvalidFrame));
}

// §15 log_shift: a positive count shifts left and a negative one right, filling
// with zeros. §15 art_shift: the same, but a right shift copies the sign bit.
#[test]
fn shifts() {
    let shift = |opcode: u8, value: u16, places: i16| {
        to(op::ext(opcode, &[L(value), L(places as u16)]), g(0))
    };
    let cases: [(u8, u16, i16, i16); 10] = [
        (op::LOG_SHIFT, 0x0001, 3, 8),
        (op::LOG_SHIFT, 0x8001, 1, 2),
        (op::LOG_SHIFT, 0x8001, -1, 0x4000),
        (op::LOG_SHIFT, 0xFFFF, -15, 1),
        (op::LOG_SHIFT, 0x1234, 0, 0x1234),
        (op::ART_SHIFT, 0x0001, 3, 8),
        (op::ART_SHIFT, 0x8001, -1, -16384), // 0xC000
        (op::ART_SHIFT, 0xFFF0, -4, -1),
        (op::ART_SHIFT, 0x7FF0, -4, 0x07FF),
        (op::ART_SHIFT, 0x4001, 1, -32766), // 0x8002
    ];
    for (opcode, value, places, expected) in cases {
        let got = out5(&cat(&[shift(opcode, value, places), print_num(V(g(0)))]));
        assert_eq!(
            got,
            format!("{expected}\n"),
            "opcode {opcode} {value:#06x} by {places}"
        );
    }
}

// §14: `not` is VAR:24 from version 5 (1OP:15 has become `call_1n`).
#[test]
fn not_is_a_var_instruction_in_version_5() {
    let code = cat(&[
        to(op::var(op::NOT_VAR, &[L(0x00FF)]), g(0)),
        print_num(V(g(0))),
    ]);
    assert_eq!(out5(&code), "-256\n");
}

fn bytes_at(story: &mut Story5, bytes: &[u8]) -> u16 {
    story.scratch(bytes)
}

/// Prints `count` bytes from `address` as numbers.
fn dump(address: u16, count: u8) -> Vec<u8> {
    let mut code = vec![];
    for i in 0..count {
        code.extend(to(op::op2(op::LOADB, L(address), S(i)), 0));
        code.extend(print_num(V(0)));
        code.extend(print(" "));
    }
    code
}

// §15 copy_table: second table 0 zeroes the first; a positive size copies so that
// overlapping tables come out right; a negative size copies forwards regardless.
#[test]
fn copy_table() {
    let run = |first_offset: u16, second: Option<u16>, size: i16| {
        let mut story = Story5::new(5);
        let table = bytes_at(&mut story, &[1, 2, 3, 4, 5, 6, 7, 8]);
        let second = second.map_or(0, |o| table + o);
        story.main(&cat(&[
            op::var(
                op::COPY_TABLE,
                &[L(table + first_offset), L(second), L(size as u16)],
            ),
            dump(table, 8),
            quit(),
        ]));
        output_of_story5(&mut story)
    };
    assert_eq!(run(2, None, 3), "1 2 0 0 0 6 7 8 \n", "zeroing");
    assert_eq!(run(0, Some(4), 4), "1 2 3 4 1 2 3 4 \n", "separate tables");
    assert_eq!(
        run(0, Some(2), 4),
        "1 2 1 2 3 4 7 8 \n",
        "overlap, copied safely"
    );
    assert_eq!(
        run(2, Some(0), 4),
        "3 4 5 6 5 6 7 8 \n",
        "overlap the other way"
    );
    assert_eq!(
        run(0, Some(2), -4),
        "1 2 1 2 1 2 7 8 \n",
        "negative size: forwards, corrupting"
    );
    assert_eq!(
        run(0, Some(4), 0),
        "1 2 3 4 5 6 7 8 \n",
        "size 0 copies nothing"
    );
}

// §15 scan_table: finds a word (or, with form bit 7 clear, a byte) in a table of
// entries of the given length, storing its address or 0 and branching if found.
#[test]
fn scan_table() {
    let run = |x: u16, len: u8, form: Option<u8>| {
        let mut story = Story5::new(5);
        // Words: 0x0102 0x0304 0x0506 0x0708
        let table = bytes_at(&mut story, &[1, 2, 3, 4, 5, 6, 7, 8]);
        let mut operands = vec![L(x), L(table), S(len)];
        operands.extend(form.map(S));
        let scan = to(op::var(op::SCAN_TABLE, &operands), g(0));
        story.main(&cat(&[
            skip_if(scan, true, &print("no ")),
            // Print the offset of the match, or -1.
            skip_if(
                op::op1(op::JZ, V(g(0))),
                true,
                &cat(&[
                    to(op::op2(op::SUB, V(g(0)), L(table)), g(0)),
                    print_num(V(g(0))),
                    quit(),
                ]),
            ),
            print("-1"),
            quit(),
        ]));
        output_of_story5(&mut story)
    };
    assert_eq!(run(0x0506, 4, None), "4\n", "words by default");
    assert_eq!(run(0x0605, 4, None), "no -1\n");
    assert_eq!(
        run(0x0304, 4, Some(0x82)),
        "2\n",
        "form 0x82 is words of 2 bytes"
    );
    assert_eq!(
        run(0x0708, 3, None),
        "no -1\n",
        "only the first three entries are searched"
    );
    assert_eq!(run(6, 8, Some(0x01)), "5\n", "bytes, one per entry");
    assert_eq!(
        run(0x0506, 2, Some(0x84)),
        "4\n",
        "words in entries of four bytes"
    );
    assert_eq!(
        run(0x0304, 2, Some(0x84)),
        "no -1\n",
        "0x0304 is not at the start of a field"
    );
}

// §15 print_table: prints a rectangle of ZSCII text, `width` characters to a row,
// `height` rows (default 1), skipping `skip` bytes between rows.
#[test]
fn print_table() {
    let run = |operands: &[Arg]| {
        let mut story = Story5::new(5);
        let table = bytes_at(&mut story, b"abcXdefXghi");
        let mut all = vec![L(table)];
        all.extend_from_slice(operands);
        story.main(&cat(&[op::var(op::PRINT_TABLE, &all), quit()]));
        output_of_story5(&mut story)
    };
    assert_eq!(run(&[S(3)]), "abc\n", "one row by default");
    assert_eq!(run(&[S(4), S(2)]), "abcX\ndefX\n");
    assert_eq!(
        run(&[S(3), S(3), S(1)]),
        "abc\ndef\nghi\n",
        "skipping one byte between rows"
    );
}

// §15 encode_text: encodes `length` characters of ZSCII text starting `from` bytes
// into a buffer, as a dictionary word, into six bytes at the given address.
#[test]
fn encode_text() {
    let mut story = Story5::new(5);
    let text = bytes_at(&mut story, b"..lantern!!");
    let coded = story.reserve(8);
    story.main(&cat(&[
        op::var(op::ENCODE_TEXT, &[L(text), S(7), S(2), L(coded)]),
        dump(coded, 7),
        quit(),
    ]));
    let expected: String = zencode_dict9("lantern")
        .iter()
        .map(|b| format!("{b} "))
        .collect::<String>()
        + "0 \n";
    assert_eq!(output_of_story5(&mut story), expected);
}

// §15 tokenise: lexical analysis of a text buffer on request, against the main
// dictionary or one of the game's own; the flag leaves unknown words untouched.
#[test]
fn tokenise() {
    let mut story = Story5::new(5);
    story.word("take");
    let text = story.text_buffer(30, "take zebra");
    let parse = story.parse_buffer(4);
    let own = story.scratch(&Story5::dictionary_bytes(".", &["zebra"], false));
    // Print whether each of the two words was found (non-zero address).
    let found = |word: u8| {
        cat(&[
            to(op::op2(op::LOADW, L(parse), S(1 + 2 * word)), 0),
            skip_if(op::op1(op::JZ, V(0)), true, &cat(&[print("y")])),
            print("."),
        ])
    };
    story.main(&cat(&[
        op::var(op::TOKENISE, &[L(text), L(parse)]),
        found(0),
        found(1),
        print(" "),
        op::var(op::TOKENISE, &[L(text), L(parse), L(own), S(1)]),
        found(0),
        found(1),
        print(" "),
        op::var(op::TOKENISE, &[L(text), L(parse), L(own)]),
        found(0),
        found(1),
        quit(),
    ]));
    // Main dictionary: take found, zebra not. Own dictionary with the flag: zebra
    // found and take's entry left alone. Own dictionary without it: take is zeroed.
    assert_eq!(output_of_story5(&mut story), "y.. y.y. .y.\n");
    let _ = SCRATCH;
}
