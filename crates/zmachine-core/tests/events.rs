//! The host-facing event loop (contracts/core-api.md).

mod common;

use common::*;
use zmachine_core::{Event, FaultKind, HostError, LoadOptions, Machine};
use zmachine_testkit::Story;
use zmachine_testkit::op::{self, Arg::*};

/// Reads a line, prints the number of words parsed and the first typed character's
/// code, then quits.
fn read_program(max_chars: u8) -> Story {
    let (mut story, text, parse) = story_with_buffers(max_chars);
    let code = cat(&[
        print("> "),
        sread(text, parse),
        cat(&[op::op2(op::LOADB, L(parse), S(1)), op::store(0)]),
        print_num(V(0)),
        print(" "),
        cat(&[op::op2(op::LOADB, L(text), S(1)), op::store(0)]),
        print_num(V(0)),
        quit(),
    ]);
    story.main(&code);
    story
}

// §15 read, versions 1 to 4: byte 0 of the text buffer holds the maximum number of
// letters plus one for the terminator. contracts/core-api.md: `LineWanted` carries
// "one less than the first byte of its text buffer".
#[test]
fn run_stops_for_a_line_and_reports_the_maximum_length() {
    let mut m = load(&mut read_program(20));
    assert_eq!(m.run(BIG), Ok(Event::LineWanted { max_len: 20 }));
    assert_eq!(m.take_output(), "> ");
}

// §15 read: the line is tokenised and execution continues after the instruction.
#[test]
fn provide_line_resumes_the_game() {
    let mut m = load(&mut read_program(20));
    m.run(BIG).unwrap();
    m.take_output();
    assert_eq!(m.provide_line("look north"), Ok(()));
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), format!("2 {}\n", b'l'));
}

// §15 read: "the interpreter should not accept more than this". Letters are
// reduced to lower case (§15 read).
#[test]
fn provide_line_truncates_and_lowercases() {
    let mut m = load(&mut read_program(4));
    assert_eq!(m.run(BIG), Ok(Event::LineWanted { max_len: 4 }));
    m.take_output();
    // "LOOK north" is cut to "look", one word.
    m.provide_line("LOOK north").unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), format!("1 {}\n", b'l'));
}

// An empty line is passed to the game, which sees zero words.
#[test]
fn an_empty_line_gives_zero_words() {
    let mut m = load(&mut read_program(20));
    m.run(BIG).unwrap();
    m.take_output();
    m.provide_line("").unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "0 0\n");
}

// A character with no ZSCII code reaches the game as a question mark (§3.8).
#[test]
fn unrepresentable_input_becomes_a_question_mark() {
    let mut m = load(&mut read_program(20));
    m.run(BIG).unwrap();
    m.take_output();
    m.provide_line("\u{4e2d}").unwrap();
    m.run(BIG).unwrap();
    assert_eq!(m.take_output(), format!("1 {}\n", b'?'));
}

// §15 quit.
#[test]
fn quit_finishes_the_game() {
    let mut story = Story::new();
    story.main(&cat(&[print("bye"), quit(), print("never")]));
    let mut m = load(&mut story);
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "bye\n", "quit ends the unfinished line");
}

/// Counts from 1 to 50 printing each number, reads a line, counts 51 to 60, quits.
fn counting_program() -> Story {
    let (mut story, text, parse) = story_with_buffers(20);
    let count_to = |limit: u8| {
        let body = cat(&[
            op::op1(op::INC, S(g(5))),
            print_num(V(g(5))),
            print(" "),
            op::op2(op::JL, V(g(5)), S(limit)),
        ]);
        let back = -(body.len() as i16 + 2) + 2;
        cat(&[body, op::branch_long(true, back)])
    };
    story.main(&cat(&[
        count_to(50),
        sread(text, parse),
        count_to(60),
        quit(),
    ]));
    story
}

/// Drives the counting program with a given budget, returning the events seen
/// (other than budget exhaustion) and all the output.
fn drive(budget: u64) -> (Vec<Event>, String, u64) {
    let mut m = load(&mut counting_program());
    let mut events = Vec::new();
    let mut output = String::new();
    let mut exhausted = 0;
    loop {
        let event = m.run(budget).unwrap();
        output.push_str(&m.take_output());
        match event {
            Event::BudgetExhausted => exhausted += 1,
            Event::LineWanted { .. } => {
                events.push(event);
                m.provide_line("look").unwrap();
            }
            Event::Finished => {
                events.push(event);
                return (events, output, exhausted);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

// contracts/core-api.md guarantee 4: "Budget size does not affect" events or output.
#[test]
fn the_budget_size_does_not_change_what_happens() {
    let (events, output, exhausted) = drive(BIG);
    assert_eq!(exhausted, 0);
    assert_eq!(
        events,
        vec![Event::LineWanted { max_len: 20 }, Event::Finished]
    );
    let expected: String = (1..=60).map(|n| format!("{n} ")).collect::<String>() + "\n";
    assert_eq!(output, expected);

    for budget in [1, 2, 7, 100] {
        let (e, o, x) = drive(budget);
        assert_eq!((e, o), (events.clone(), output.clone()), "budget {budget}");
        assert!(x > 0, "budget {budget} must have run out at least once");
    }
}

// FR-010: the host can stop a game that never asks for input. A budget of n runs
// exactly n instructions.
#[test]
fn an_endless_loop_only_ever_exhausts_the_budget() {
    let mut story = Story::new();
    // inc g0; jump back to it.
    let inc = op::op1(op::INC, S(g(0)));
    let back = -(inc.len() as i16 + 3) + 2;
    story.main(&cat(&[inc, op::op1(op::JUMP, L(back as u16)), quit()]));
    let mut m = load(&mut story);
    for _ in 0..5 {
        assert_eq!(m.run(1000), Ok(Event::BudgetExhausted));
    }
    assert_eq!(
        m.run(0),
        Ok(Event::BudgetExhausted),
        "a zero budget runs nothing"
    );
}

// contracts/core-api.md: `HostError` is returned, "and nothing changes", when the
// host calls out of turn.
#[test]
fn calling_out_of_turn_is_refused_and_changes_nothing() {
    let mut m = load(&mut read_program(20));
    // Answers before anything was asked.
    assert_eq!(m.provide_line("look"), Err(HostError::NotWaitingForThis));
    assert_eq!(m.save_result(true), Err(HostError::NotWaitingForThis));
    assert_eq!(m.provide_restore(None), Err(HostError::NotWaitingForThis));
    assert_eq!(m.transcript_result(true), Err(HostError::NotWaitingForThis));

    assert_eq!(m.run(BIG), Ok(Event::LineWanted { max_len: 20 }));
    // Running, or giving the wrong answer, while a line is owed.
    assert_eq!(m.run(BIG), Err(HostError::AnswerOwed));
    assert_eq!(m.save_result(true), Err(HostError::NotWaitingForThis));
    assert_eq!(m.provide_restore(None), Err(HostError::NotWaitingForThis));
    assert_eq!(
        m.transcript_result(false),
        Err(HostError::NotWaitingForThis)
    );
    assert_eq!(m.take_output(), "> ");

    // The game is unharmed: the right answer still works.
    m.provide_line("north look").unwrap();
    assert_eq!(m.provide_line("again"), Err(HostError::NotWaitingForThis));
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), format!("2 {}\n", b'n'));

    // After the game has finished, everything is refused.
    assert_eq!(m.run(BIG), Err(HostError::Stopped));
    assert_eq!(m.provide_line("x"), Err(HostError::Stopped));
    assert_eq!(m.save_result(true), Err(HostError::Stopped));
    assert_eq!(m.provide_restore(None), Err(HostError::Stopped));
    assert_eq!(m.transcript_result(true), Err(HostError::Stopped));
}

// contracts/core-api.md guarantee 5: the `take_` calls do not affect execution.
#[test]
fn draining_is_passive() {
    let run = |drain: bool| {
        let mut m = load(&mut counting_program());
        let mut output = String::new();
        loop {
            let event = m.run(3).unwrap();
            if drain {
                output.push_str(&m.take_output());
                m.take_status();
                m.take_diagnostics();
                m.take_transcript();
            }
            match event {
                Event::LineWanted { .. } => m.provide_line("look").unwrap(),
                Event::Finished => break,
                _ => {}
            }
        }
        output.push_str(&m.take_output());
        output
    };
    assert_eq!(run(true), run(false));
    // Nothing is lost by draining late, and a second drain is empty.
    let mut m = load(&mut counting_program());
    run_until_event(&mut m, BIG);
    assert!(!m.take_output().is_empty());
    assert_eq!(m.take_output(), "");
}

// FR-018: a fault stops that game with its kind and address; the host carries on
// and can load another.
#[test]
fn after_a_fault_the_machine_is_stopped_and_another_can_run() {
    let mut story = Story::new();
    let at = story.next_code_address();
    story.main(&cat(&[
        print("before"),
        cat(&[op::op2(op::DIV, S(1), S(0)), op::store(0)]),
        quit(),
    ]));
    let mut m = load(&mut story);
    let Ok(Event::Fault(fault)) = m.run(BIG) else {
        panic!("expected a fault")
    };
    assert_eq!(fault.kind, FaultKind::DivisionByZero);
    assert_eq!(fault.address, at + print("before").len() as u32);
    assert_eq!(
        m.take_output(),
        "before",
        "text printed before the fault is kept"
    );
    assert_eq!(m.run(BIG), Err(HostError::Stopped));

    let mut other = load(&mut read_program(20));
    assert_eq!(other.run(BIG), Ok(Event::LineWanted { max_len: 20 }));
}

// FR-002: files that are not version 3 story files are refused with a reason.
#[test]
fn load_refuses_what_is_not_a_version_3_story() {
    use zmachine_core::LoadError;
    let options = LoadOptions {
        seed: None,
        entropy: 7,
    };
    // Version 4 is not run; versions 5 and 8 are, since feature 002.
    let mut v4 = read_program(20).build();
    v4[0] = 4;
    assert!(matches!(
        Machine::load(v4, options),
        Err(LoadError::UnsupportedVersion(4))
    ));
    assert!(matches!(
        Machine::load(vec![3; 10], options),
        Err(LoadError::TooShort)
    ));
    assert!(matches!(
        Machine::load(Vec::new(), options),
        Err(LoadError::TooShort)
    ));
    let text = b"This is not a story file at all, just some text that is long enough.".to_vec();
    assert!(matches!(
        Machine::load(text, options),
        Err(LoadError::UnsupportedVersion(b'T'))
    ));
}

// A text or parse buffer outside dynamic memory is the game's error, reported as a
// fault when the line is delivered, never a panic.
#[test]
fn a_read_into_static_memory_is_a_fault() {
    let (mut story, _text, parse) = story_with_buffers(20);
    let static_base = story.static_base();
    story.main(&cat(&[sread(static_base, parse), quit()]));
    let mut m = load(&mut story);
    match m.run(BIG).unwrap() {
        Event::Fault(f) => assert_eq!(f.kind, FaultKind::WriteOutsideDynamicMemory),
        Event::LineWanted { .. } => {
            m.provide_line("look").unwrap();
            let Ok(Event::Fault(f)) = m.run(BIG) else {
                panic!("expected a fault")
            };
            assert_eq!(f.kind, FaultKind::WriteOutsideDynamicMemory);
        }
        other => panic!("unexpected {other:?}"),
    }
}
