//! Save and restore in versions 5 and 8 (feature 002, T028; §15 save, restore;
//! Quetzal §5.8.2).

mod common;

use common::*;
use zmachine_core::{Event, Machine};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::Story5;

fn save(variable: u8) -> Vec<u8> {
    to(op::ext(op::EXT_SAVE, &[]), variable)
}

fn restore(variable: u8) -> Vec<u8> {
    to(op::ext(op::EXT_RESTORE, &[]), variable)
}

struct Outcome {
    output: String,
    saves: Vec<Vec<u8>>,
    restores: usize,
}

/// Runs an image; saves are stored unless `store` is false; the first `offers`
/// restore requests are given `offered` (or the last save), later ones declined.
fn drive(image: Vec<u8>, store: bool, offers: usize, offered: Option<Vec<u8>>) -> Outcome {
    let mut m = Machine::load(image, OPTIONS).unwrap();
    let mut out = Outcome { output: String::new(), saves: Vec::new(), restores: 0 };
    loop {
        let event = m.run(BIG).unwrap();
        out.output.push_str(&m.take_output());
        match event {
            Event::Finished => return out,
            Event::SaveRequested(bytes) => {
                out.saves.push(bytes);
                m.save_result(store).unwrap();
            }
            Event::RestoreRequested => {
                out.restores += 1;
                if out.restores <= offers {
                    let bytes = offered.clone().or_else(|| out.saves.last().cloned());
                    m.provide_restore(bytes.as_deref()).unwrap();
                } else {
                    m.provide_restore(None).unwrap();
                }
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

/// Inside a routine: save -> g0 and print it. If it is 1 (just saved), change a
/// global and a local and restore. A successful restore comes back out of the save
/// with 2.
fn program(version: u8, serial: &str) -> Vec<u8> {
    let mut story = Story5::new(version);
    story.serial(serial);
    let report = cat(&[print("back:"), show(1), print_num(V(1)), quit()]);
    let routine = story.routine(
        1,
        &cat(&[
            op::op2(op::STORE, S(1), S(6)),
            save(g(0)),
            show(0),
            skip_if(op::op2(op::JE, V(g(0)), S(2)), false, &report),
            op::op2(op::STORE, S(g(1)), S(9)),
            op::op2(op::STORE, S(1), S(9)),
            restore(g(2)),
            print("restore:"),
            show(2),
            quit(),
        ]),
    );
    story.main(&cat(&[op::op2(op::STORE, S(g(1)), S(5)), op::op1(op::CALL_1N, L(routine)), quit()]));
    story.build()
}

// §15 save, version 5: stores 1 on success and 0 on failure. §15 restore: a
// restored game resumes at the save with 2 stored, state as it was.
#[test]
fn save_stores_its_result_and_restore_resumes_with_2() {
    let ok = drive(program(5, "111111"), true, 1, None);
    assert_eq!(ok.output, "1 2 back:5 6\n");
    assert_eq!(ok.saves.len(), 1);
    assert_eq!(&ok.saves[0][..4], b"FORM");

    // The host fails to store: save stores 0. The restore that follows has nothing
    // to restore and is declined: it stores 0 and the game carries on.
    let failed = drive(program(5, "111111"), false, 0, None);
    assert_eq!(failed.output, "0 restore:0 \n");
}

// Quetzal §5.8.2: "the saved PC points to the single byte describing where to
// store the result". The byte before it is the last byte of the `save` instruction,
// its operand types byte (0xFF: no operands), after 0xBE and opcode 0.
#[test]
fn the_saved_program_counter_points_at_the_store_byte() {
    let image = program(5, "111111");
    let outcome = drive(image.clone(), true, 0, None);
    let save = &outcome.saves[0];
    // IFhd is the first chunk: FORM(4) len(4) IFZS(4) IFhd(4) len(4), then 13 bytes.
    assert_eq!(&save[12..16], b"IFhd");
    let pc = u32::from_be_bytes([0, save[30], save[31], save[32]]) as usize;
    assert_eq!(&image[pc - 3..pc], &[0xBE, 0x00, 0xFF]);
    assert_eq!(image[pc], 16, "the store byte names global 0");
}

// FR-017 for version 5: a refused restore stores 0 and changes nothing.
#[test]
fn a_refused_restore_stores_0_and_changes_nothing() {
    let other_story = drive(program(5, "222222"), true, 0, None).saves.remove(0);
    for (what, bytes) in [
        ("another story's save", other_story),
        ("not a save", b"Dear diary".to_vec()),
        ("an empty file", Vec::new()),
    ] {
        let outcome = drive(program(5, "111111"), true, 1, Some(bytes));
        assert_eq!(outcome.output, "1 restore:0 \n", "{what}");
    }
}

// data-model.md: a version 5 build and a version 8 build of the same game are
// different stories; a save from one is refused by the other.
#[test]
fn a_save_from_the_other_version_is_refused() {
    let from_8 = drive(program(8, "111111"), true, 0, None).saves.remove(0);
    let outcome = drive(program(5, "111111"), true, 1, Some(from_8));
    assert_eq!(outcome.output, "1 restore:0 \n");
    // And version 8 saves and restores as version 5 does.
    assert_eq!(drive(program(8, "111111"), true, 1, None).output, "1 2 back:5 6\n");
}

// §7.6.5.1, 002 FR-021: `save` and `restore` with a table to write or read are
// answered as failures, and the host is not asked.
#[test]
fn save_and_restore_of_a_memory_region_fail_without_asking() {
    let mut story = Story5::new(5);
    let table = story.reserve(8);
    story.main(&cat(&[
        to(op::ext(op::EXT_SAVE, &[L(table), S(8)]), g(0)),
        to(op::ext(op::EXT_RESTORE, &[L(table), S(8)]), g(1)),
        show(0),
        show(1),
        quit(),
    ]));
    let outcome = drive(story.build(), true, 5, None);
    assert_eq!(outcome.output, "0 0 \n");
    assert_eq!((outcome.saves.len(), outcome.restores), (0, 0));
}

// Quetzal §4.3: a frame made by a discarding call has the p flag set; the argument
// mask has a bit for each argument passed. A save taken inside nested calls of
// both kinds restores correctly into a fresh machine.
#[test]
fn a_save_inside_nested_calls_restores_into_a_fresh_machine() {
    let mut story = Story5::new(5);
    let failed = cat(&[print("savefail"), quit()]);
    // inner(a, b): save; on 1 or 2 print the locals and return a + b.
    let inner = story.routine(
        2,
        &cat(&[
            save(g(0)),
            skip_if(op::op1(op::JZ, V(g(0))), false, &failed),
            show(0),
            to(op::op2(op::ADD, V(1), V(2)), 0),
            op::op0(op::RET_POPPED),
        ]),
    );
    // outer(): g1 = inner(30, 12); print it.
    let outer = story.routine(
        0,
        &cat(&[to(op::var(op::CALL_VS, &[L(inner), S(30), S(12)]), g(1)), show(1), op::op0(op::RTRUE)]),
    );
    story.main(&cat(&[
        // In a fresh machine offered a save, this restore does not return.
        restore(g(5)),
        op::var(op::PUSH, &[S(77)]),
        op::op1(op::CALL_1N, L(outer)),
        print_num(V(0)),
        quit(),
    ]));
    let image = story.build();
    let first = drive(image.clone(), true, 0, None);
    assert_eq!(first.output, "1 42 77\n");
    let second = drive(image, true, 1, Some(first.saves[0].clone()));
    assert_eq!(second.output, "2 42 77\n");
    assert!(second.saves.is_empty());
}
