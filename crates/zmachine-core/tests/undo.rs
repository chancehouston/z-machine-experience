//! Undo (feature 002, T027; §15 save_undo, restore_undo; §6.1.2;
//! specs/002-v5-v8/data-model.md "Undo position").

mod common;

use common::*;
use zmachine_core::{Event, LoadOptions, Machine, Settings};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::Story5;

fn save_undo(variable: u8) -> Vec<u8> {
    to(op::ext(op::SAVE_UNDO, &[]), variable)
}

fn restore_undo(variable: u8) -> Vec<u8> {
    to(op::ext(op::RESTORE_UNDO, &[]), variable)
}

/// `save_undo -> g0`. First time through (g0 = 1): change state and `restore_undo`.
/// The game then comes out of the same `save_undo` with 2.
fn undo_program() -> Vec<u8> {
    let mut story = Story5::new(5);
    let after_undo = cat(&[print("back:"), show(1), print_num(V(0)), quit()]);
    let routine = story.routine(
        1,
        &cat(&[
            op::op2(op::STORE, S(1), S(6)),
            op::var(op::PUSH, &[S(42)]),
            save_undo(g(0)),
            show(0),
            // If the result is 2 we have been undone: report and stop.
            skip_if(op::op2(op::JE, V(g(0)), S(2)), false, &after_undo),
            // Otherwise change a global, the local and the stack, then undo.
            op::op2(op::STORE, S(g(1)), S(9)),
            op::op2(op::STORE, S(1), S(9)),
            op::var(op::PUSH, &[S(99)]),
            restore_undo(g(2)),
            print("undo failed"),
            quit(),
        ]),
    );
    story.main(&cat(&[
        op::op2(op::STORE, S(g(1)), S(5)),
        op::op1(op::CALL_1N, L(routine)),
        quit(),
    ]));
    story.build()
}

// §15 save_undo: stores 1. §15 restore_undo: the game resumes at the save_undo
// with 2 stored, memory and the call stack as they were.
#[test]
fn undo_returns_to_the_saved_position_with_2() {
    let mut m = Machine::load(undo_program(), OPTIONS).unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    // "1 " from the first pass; then back with 2, global 1 restored to 5, and the
    // stack top the 42 pushed before the save.
    assert_eq!(m.take_output(), "1 2 back:5 42\n");
}

// The result may go to the stack or a local as well as a global.
#[test]
fn the_result_goes_wherever_the_instruction_stores() {
    for target in [0u8, 1] {
        let mut story = Story5::new(5);
        let done = cat(&[print("undone"), quit()]);
        let routine = story.routine(
            2,
            &cat(&[
                save_undo(target),
                // Read the result back into global 0 (pops it if it was on the stack).
                op::op2(op::STORE, S(g(0)), V(target)),
                show(0),
                skip_if(op::op2(op::JE, V(g(0)), S(2)), false, &done),
                restore_undo(g(3)),
                quit(),
            ]),
        );
        story.main(&cat(&[op::op1(op::CALL_1N, L(routine)), quit()]));
        assert_eq!(output_of_story5(&mut story), "1 2 undone\n", "target variable {target}");
    }
}

// §15 restore_undo: with nothing to undo it stores 0 and the game continues.
#[test]
fn restore_undo_with_nothing_saved_stores_0() {
    let code = cat(&[op::op2(op::STORE, S(g(0)), S(7)), restore_undo(g(0)), show(0)]);
    assert_eq!(out5(&code), "0 \n");
}

/// Saves `saves` undo positions, each with a different value in global 1, then
/// undoes until it fails, printing the value restored each time.
fn depth_program(saves: u8) -> Vec<u8> {
    let mut story = Story5::new(5);
    // Loop: g1 += 1; save_undo -> g0; if g0 == 2 goto undone; if g1 < saves loop.
    let bump = op::op1(op::INC, S(g(1)));
    let save = save_undo(g(0));
    let test_undone = op::op2(op::JE, V(g(0)), S(2));
    let loop_test = op::op2(op::JL, V(g(1)), S(saves));
    // Layout: [bump][save][test_undone ?undone][loop_test ?loop][restore] undone: [print][restore]...
    let undo_block = cat(&[restore_undo(g(2)), print("end"), quit()]);
    let mut code = vec![];
    let loop_start = 0usize;
    code.extend(&bump);
    code.extend(&save);
    code.extend(&test_undone);
    // Branch on true to `undone`, which follows the loop test and the first undo block.
    let to_undone = loop_test.len() + 2 + undo_block.len() + 2;
    code.extend(op::branch(true, to_undone as i16));
    code.extend(&loop_test);
    let after = code.len() + 2;
    code.extend(op::branch_long(true, loop_start as i16 - after as i16 + 2));
    code.extend(&undo_block);
    // undone:
    code.extend(show(1));
    code.extend(&undo_block);
    story.main(&code);
    story.build()
}

// data-model.md, research R5: up to 25 positions are kept, the reference
// interpreter's number; the oldest is dropped when another is saved.
#[test]
fn the_25_most_recent_positions_are_kept() {
    let run = |saves: u8, settings: Settings| {
        let mut m = Machine::load_with(depth_program(saves), OPTIONS, settings).unwrap();
        assert_eq!(m.run(BIG), Ok(Event::Finished));
        m.take_output()
    };
    // Three saves, three undos: back to 3, 2, 1, then nothing left.
    assert_eq!(run(3, Settings::default()), "3 2 1 end\n");
    // Forty saves: only the last 25 can be undone, 40 down to 16.
    let expected: String = (16..=40).rev().map(|n| format!("{n} ")).collect::<String>() + "end\n";
    assert_eq!(run(40, Settings::default()), expected);
    // The depth is the host's setting.
    assert_eq!(run(5, Settings { undo_slots: 2, ..Settings::default() }), "5 4 end\n");
}

// §15 save_undo: "-1 if the interpreter does not provide undo". §6.1.4: and Flags 2
// bit 4 is cleared so the game can tell beforehand.
#[test]
fn with_no_undo_slots_save_undo_stores_minus_1() {
    let mut story = Story5::new(5);
    story.main(&cat(&[
        save_undo(g(0)),
        show(0),
        restore_undo(g(1)),
        show(1),
        cat(&[to(op::op2(op::LOADB, S(0), S(0x11)), 0), to(op::op2(op::AND, V(0), S(0x10)), 0), print_num(V(0))]),
        quit(),
    ]));
    let mut image = story.build();
    image[0x11] |= 0x10; // the game asks for undo
    let sum = image[0x40..].iter().fold(0u32, |a, &b| a + b as u32) & 0xFFFF;
    image[0x1C..0x1E].copy_from_slice(&(sum as u16).to_be_bytes());
    let settings = Settings { undo_slots: 0, ..Settings::default() };
    let mut m = Machine::load_with(image, OPTIONS, settings).unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "-1 -1 0\n");
}

// §6.1.2: Flags 2 is preserved across an undo, and §6.1.2.2 the interpreter writes
// its own header fields again. The game sets Flags 2 bit 1 and dirties the Standard
// revision byte after saving; after the undo the bit is still set and the byte is 1.
#[test]
fn undo_keeps_flags_2_and_reapplies_the_interpreter_header() {
    let report = cat(&[
        cat(&[to(op::op2(op::LOADB, S(0), S(0x11)), 0), to(op::op2(op::AND, V(0), S(2)), 0), print_num(V(0))]),
        print(" "),
        cat(&[to(op::op2(op::LOADB, S(0), S(0x32)), 0), print_num(V(0))]),
        quit(),
    ]);
    let code = cat(&[
        save_undo(g(0)),
        skip_if(op::op2(op::JE, V(g(0)), S(2)), false, &report),
        op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
        op::var(op::STOREB, &[S(0), S(0x32), S(9)]),
        restore_undo(g(1)),
    ]);
    assert_eq!(out5(&code), "2 1\n");
}

// data-model.md: "The random state, the screen and the output streams are not part
// of a position." Six numbers drawn around a save and an undo are the six a plain
// run draws.
#[test]
fn undo_does_not_touch_the_random_generator() {
    let draw = || cat(&[to(op::var(op::RANDOM, &[L(1000)]), 0), print_num(V(0)), print(" ")]);
    let options = LoadOptions { seed: Some(77), entropy: 0 };
    let run = |code: Vec<u8>| {
        let mut story = Story5::new(5);
        story.main(&cat(&[code, quit()]));
        let mut m = Machine::load(story.build(), options).unwrap();
        assert_eq!(m.run(BIG), Ok(Event::Finished));
        m.take_output()
    };
    let plain = run(cat(&[draw(), draw(), draw(), draw(), draw(), draw()]));
    // draw 2; save_undo; draw 2; if this is the second time through, draw 2 more and
    // stop; otherwise undo.
    let finish = cat(&[draw(), draw(), quit()]);
    let with_undo = run(cat(&[
        draw(),
        draw(),
        save_undo(g(0)),
        skip_if(op::op2(op::JE, V(g(0)), S(2)), false, &finish),
        draw(),
        draw(),
        restore_undo(g(1)),
    ]));
    assert_eq!(with_undo, plain);
}

// A version 3 story has no undo instructions; the extended form does not exist
// there (§14).
#[test]
fn version_3_has_no_undo() {
    let mut story = zmachine_testkit::Story::new();
    story.main(&cat(&[op::ext(op::SAVE_UNDO, &[]), vec![0], quit()]));
    let mut m = load(&mut story);
    assert!(matches!(m.run(BIG), Ok(Event::Fault(f)) if f.kind == zmachine_core::FaultKind::UnknownInstruction));
}
