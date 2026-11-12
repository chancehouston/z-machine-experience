//! Output: printing instructions, the status line, windows and stream selection.

mod common;

use common::*;
use zmachine_core::{Event, LoadOptions, Machine, StatusKind, StatusLine};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Object, Story, zencode};

// §15 print, new_line, print_char, print_num.
#[test]
fn literal_printing() {
    let code = cat(&[
        print("Hello, World!"),
        op::op0(op::NEW_LINE),
        op::var(op::PRINT_CHAR, &[S(b'A')]),
        op::var(op::PRINT_CHAR, &[S(13)]),
        print_num(S(42)),
        print(" "),
        print_num(L(-5i16 as u16)),
        print(" "),
        print_num(L(0x8000)),
        print(" "),
        print_num(S(0)),
    ]);
    assert_eq!(output_of(&code), "Hello, World!\nA\n42 -5 -32768 0\n");
}

// §3.8.2.1: ZSCII 0 prints nothing.
#[test]
fn print_char_zero_prints_nothing() {
    assert_eq!(
        output_of(&cat(&[op::var(op::PRINT_CHAR, &[S(0)]), print("x")])),
        "x\n"
    );
}

// §15 print_ret: "Print the quoted (literal) Z-encoded string, then print a
// new-line and then return true".
#[test]
fn print_ret_prints_a_newline_and_returns_true() {
    let mut story = Story::new();
    let routine = story.routine(
        &[],
        &cat(&[op::op0(op::PRINT_RET), zencode("done"), print("no")]),
    );
    story.main(&cat(&[
        cat(&[op::var(op::CALL, &[L(routine)]), op::store(g(0))]),
        print_num(V(g(0))),
        quit(),
    ]));
    assert_eq!(output_of_story(&mut story), "done\n1\n");
}

// §15 print_addr takes a byte address; print_paddr a packed address (§1.2.3:
// twice the packed address in version 3).
#[test]
fn print_addr_and_print_paddr() {
    let mut story = Story::new();
    let packed = story.string("by packed");
    let other = story.string("by byte");
    story.main(&cat(&[
        op::op1(op::PRINT_PADDR, L(packed)),
        print("/"),
        op::op1(op::PRINT_ADDR, L(other * 2)),
        quit(),
    ]));
    assert_eq!(output_of_story(&mut story), "by packed/by byte\n");
}

// §15 print_obj: the short name of the object.
#[test]
fn print_obj_prints_the_short_name() {
    let mut story = Story::new();
    story.object(Object::new("brass lantern"));
    story.object(Object::new(""));
    story.main(&cat(&[
        op::op1(op::PRINT_OBJ, S(1)),
        print("|"),
        op::op1(op::PRINT_OBJ, S(2)),
        print("|"),
        quit(),
    ]));
    assert_eq!(output_of_story(&mut story), "brass lantern||\n");
}

// §3.3: abbreviations are expanded wherever text is printed.
#[test]
fn printed_text_expands_abbreviations() {
    let mut story = Story::new();
    story.abbreviation(0, "the ");
    // Z-characters: 1,0 (abbreviation 0) then "end".
    let z = [1u16, 0, 10, 19, 9, 5];
    let words = [
        z[0] << 10 | z[1] << 5 | z[2],
        0x8000 | z[3] << 10 | z[4] << 5 | z[5],
    ];
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_be_bytes()).collect();
    story.main(&cat(&[op::op0(op::PRINT), bytes, quit()]));
    assert_eq!(output_of_story(&mut story), "the end\n");
}

/// A story that reads a line and then prints "ok".
fn reading_story(flags1: u8, score: u16, turns: u16) -> Story {
    let (mut story, text, parse) = story_with_buffers(20);
    story.flags1(flags1).global(1, score).global(2, turns);
    story.main(&cat(&[print(">"), sread(text, parse), print("ok"), quit()]));
    story
}

// §7.1.1.1 is about the screen echoing what the player types; the host has already
// shown it, so the core's screen text must not repeat it (research.md R6).
#[test]
fn player_input_is_not_echoed_into_the_screen_text() {
    let mut m = load(&mut reading_story(0, 0, 0));
    m.run(BIG).unwrap();
    assert_eq!(m.take_output(), ">");
    m.provide_line("look").unwrap();
    m.run(BIG).unwrap();
    assert_eq!(m.take_output(), "ok\n");
}

// §8.2.2: the status line shows the short name of the object in the first global,
// then score and turns from the second and third. §8.2.1: it is redrawn before
// keyboard input in version 3.
#[test]
fn status_line_for_a_score_game_is_produced_before_reading() {
    let mut m = load(&mut reading_story(0, -3i16 as u16, 17));
    assert_eq!(
        m.take_status(),
        None,
        "nothing until the game asks for input"
    );
    m.run(BIG).unwrap();
    assert_eq!(
        m.take_status(),
        Some(StatusLine {
            location: "Room".to_string(),
            kind: StatusKind::Score {
                score: -3,
                turns: 17
            },
        })
    );
    assert_eq!(
        m.take_status(),
        None,
        "only when it changed since the last call"
    );
}

// §8.2.3.2, §11.1 Flags 1 bit 1: a time game shows hours and minutes instead.
#[test]
fn status_line_for_a_time_game() {
    let mut m = load(&mut reading_story(0b0000_0010, 14, 5));
    m.run(BIG).unwrap();
    assert_eq!(
        m.take_status(),
        Some(StatusLine {
            location: "Room".to_string(),
            kind: StatusKind::Time {
                hours: 14,
                minutes: 5
            },
        })
    );
}

// §15 show_status: redraws the status line on demand. The host is told again only
// when the content differs.
#[test]
fn show_status_updates_when_the_content_changes() {
    let mut story = Story::new();
    story.object(Object::new("Cave"));
    story.object(Object::new("Lake"));
    story.global(0, 1).global(1, 10).global(2, 1);
    let (text, parse) = {
        let mut t = vec![0u8; 32];
        t[0] = 21;
        let mut p = vec![0u8; 22];
        p[0] = 5;
        (story.scratch(&t), story.scratch(&p))
    };
    story.main(&cat(&[
        op::op0(op::SHOW_STATUS),
        sread(text, parse),
        op::op0(op::SHOW_STATUS),
        sread(text, parse),
        op::op2(op::STORE, S(g(0)), S(2)),
        op::op0(op::SHOW_STATUS),
        quit(),
    ]));
    let mut m = load(&mut story);
    let score = StatusKind::Score {
        score: 10,
        turns: 1,
    };
    m.run(BIG).unwrap();
    assert_eq!(
        m.take_status(),
        Some(StatusLine {
            location: "Cave".into(),
            kind: score
        })
    );
    m.provide_line("").unwrap();
    m.run(BIG).unwrap();
    assert_eq!(m.take_status(), None, "same content, shown twice");
    m.provide_line("").unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(
        m.take_status(),
        Some(StatusLine {
            location: "Lake".into(),
            kind: score
        })
    );
}

// research.md R12: a location of object 0 gives an empty name and a diagnostic.
#[test]
fn status_line_with_no_location_object_is_tolerated() {
    let (mut story, text, parse) = story_with_buffers(20);
    story.global(0, 0);
    story.main(&cat(&[sread(text, parse), quit()]));
    let mut m = load(&mut story);
    assert!(matches!(m.run(BIG), Ok(Event::LineWanted { .. })));
    assert_eq!(m.take_status().unwrap().location, "");
    assert_eq!(m.take_diagnostics().len(), 1);
}

// §7.1, §7.4, §15 output_stream: a positive number selects a stream and a negative
// one deselects it. With stream 1 deselected nothing reaches the screen.
#[test]
fn deselecting_stream_one_suppresses_screen_text() {
    let code = cat(&[
        print("a"),
        op::var(op::OUTPUT_STREAM, &[L(-1i16 as u16)]),
        print("hidden"),
        op::op0(op::NEW_LINE),
        print_num(S(5)),
        op::var(op::OUTPUT_STREAM, &[S(1)]),
        print("b"),
    ]);
    assert_eq!(output_of(&code), "ab\n");
}

// §15 output_stream: "If stream is 0, nothing happens."
#[test]
fn output_stream_zero_does_nothing() {
    let code = cat(&[op::var(op::OUTPUT_STREAM, &[S(0)]), print("x")]);
    assert_eq!(output_of(&code), "x\n");
}

// §8.5.2, §15 split_window, set_window: version 3 has a lower window (0) for the
// game's text and an upper window (1). Only lower-window text is game text.
#[test]
fn upper_window_text_stays_out_of_the_screen_text() {
    let code = cat(&[
        print("one "),
        op::var(op::SPLIT_WINDOW, &[S(2)]),
        op::var(op::SET_WINDOW, &[S(1)]),
        print("UPPER"),
        op::var(op::SET_WINDOW, &[S(0)]),
        print("two"),
        op::var(op::SPLIT_WINDOW, &[S(0)]),
    ]);
    assert_eq!(output_of(&code), "one two\n");
}

// §15 input_stream, sound_effect: accepted. Version 3 games use them for command
// files and bleeps, which this host does not offer.
#[test]
fn input_stream_and_sound_effect_are_accepted() {
    let code = cat(&[
        op::var(op::INPUT_STREAM, &[S(0)]),
        op::var(op::INPUT_STREAM, &[S(1)]),
        op::var(op::SOUND_EFFECT, &[S(1)]),
        op::var(op::SOUND_EFFECT, &[S(2)]),
        op::var(op::SOUND_EFFECT, &[S(3), S(2), S(8)]),
        print("still here"),
    ]);
    assert_eq!(output_of(&code), "still here\n");
}

/// Prints "good" or "bad" according to `verify`.
fn verify_story() -> Story {
    let mut story = Story::new();
    story.string("padding so that there is something to corrupt");
    let bad = cat(&[print("bad"), quit()]);
    story.main(&cat(&[
        skip_if(op::op0(op::VERIFY), true, &bad),
        print("good"),
        quit(),
    ]));
    story
}

fn run_image(image: Vec<u8>) -> String {
    let mut m = Machine::load(image, OPTIONS).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
    m.take_output()
}

// §15 verify: the sum of the bytes from 0x40 to the stated length of the file,
// modulo 0x10000, compared with the header checksum.
#[test]
fn verify_passes_on_an_intact_file() {
    assert_eq!(run_image(verify_story().build()), "good\n");
}

#[test]
fn verify_fails_on_a_corrupted_file() {
    // Change one byte of the padding string, which sits at the start of high memory
    // and is never executed.
    let mut image = verify_story().build();
    let high = u16::from_be_bytes([image[0x04], image[0x05]]) as usize;
    image[high] = image[high].wrapping_add(1);
    assert_eq!(run_image(image), "bad\n");

    // A stated length longer than the file cannot be summed.
    let mut image = verify_story().build();
    image[0x1A] = 0xFF;
    image[0x1B] = 0xFF;
    assert_eq!(run_image(image), "bad\n");
}

// §15 verify sums the file as shipped: what the game or the interpreter has since
// written into dynamic memory does not count.
#[test]
fn verify_is_not_disturbed_by_changes_to_dynamic_memory() {
    let mut story = Story::new();
    let scratch = story.reserve(4);
    let bad = cat(&[print("bad"), quit()]);
    story.main(&cat(&[
        op::var(op::STOREW, &[L(scratch), S(0), L(0xFFFF)]),
        skip_if(op::op0(op::VERIFY), true, &bad),
        print("good"),
        quit(),
    ]));
    assert_eq!(output_of_story(&mut story), "good\n");
}

// §6.1.3, §15 restart: the game returns to its original state, except that Flags 2
// survives (the transcribing and fixed-pitch bits). The interpreter re-applies the
// header fields it owns.
#[test]
fn restart_returns_to_the_opening_state() {
    let mut story = Story::new();
    // First pass: Flags 2 bit 1 is clear. Set it, dirty a global and the Standard
    // revision byte, push a value, and restart. Second pass: the bit is still set,
    // so print what survived and quit.
    let first_pass = cat(&[
        op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
        op::op2(op::STORE, S(g(5)), S(77)),
        op::var(op::STOREB, &[S(0), S(0x32), S(9)]),
        op::var(op::PUSH, &[S(1)]),
        op::op0(op::RESTART),
    ]);
    let code = cat(&[
        print("start "),
        cat(&[op::op2(op::LOADB, S(0), S(0x11)), op::store(0)]),
        skip_if(op::op2(op::TEST, V(0), S(2)), true, &first_pass),
        print_num(V(g(5))),
        print(" "),
        cat(&[op::op2(op::LOADB, S(0), S(0x32)), op::store(0)]),
        print_num(V(0)),
        quit(),
    ]);
    story.main(&code);
    assert_eq!(output_of_story(&mut story), "start start 0 1\n");
}

// The seed in LoadOptions is optional; without one the entropy value is required
// and the machine still runs.
#[test]
fn a_machine_runs_without_a_fixed_seed() {
    let mut story = Story::new();
    story.main(&cat(&[print("ok"), quit()]));
    let mut m = Machine::load(
        story.build(),
        LoadOptions {
            seed: None,
            entropy: 99,
        },
    )
    .unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "ok\n");
}
