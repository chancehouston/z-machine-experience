//! Save and restore through the event interface (§15 save, restore; §6.1.2;
//! contracts/core-api.md).

mod common;

use common::*;
use zmachine_core::{Event, HostError, Machine};
use zmachine_testkit::Story;
use zmachine_testkit::op::{self, Arg::*};

/// How a test's host answers save and restore requests.
#[derive(Default)]
struct Host {
    /// Whether each save, in turn, is reported stored (default: yes).
    save_answers: Vec<bool>,
    /// What each restore, in turn, is given: `Some(None)` uses the last save,
    /// `Some(Some(bytes))` those bytes, `None` declines. Past the end: decline.
    restore_answers: Vec<Option<Option<Vec<u8>>>>,
}

struct Outcome {
    output: String,
    saves: Vec<Vec<u8>>,
    restores_requested: usize,
    /// The machine's count of jumps when the game finished.
    jumps: u32,
}

fn drive(image: Vec<u8>, host: Host) -> Outcome {
    let mut m = Machine::load(image, OPTIONS).unwrap();
    let mut outcome = Outcome {
        output: String::new(),
        saves: Vec::new(),
        restores_requested: 0,
        jumps: 0,
    };
    loop {
        let event = m.run(BIG).unwrap();
        outcome.output.push_str(&m.take_output());
        match event {
            Event::Finished => {
                outcome.jumps = m.jumps();
                return outcome;
            }
            Event::SaveRequested(bytes) => {
                let stored = host
                    .save_answers
                    .get(outcome.saves.len())
                    .copied()
                    .unwrap_or(true);
                outcome.saves.push(bytes);
                m.save_result(stored).unwrap();
            }
            Event::RestoreRequested => {
                let answer = host
                    .restore_answers
                    .get(outcome.restores_requested)
                    .cloned()
                    .flatten();
                outcome.restores_requested += 1;
                match answer {
                    None => m.provide_restore(None).unwrap(),
                    Some(None) => m
                        .provide_restore(Some(outcome.saves.last().unwrap()))
                        .unwrap(),
                    Some(Some(bytes)) => m.provide_restore(Some(&bytes)).unwrap(),
                }
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

/// `save`, printing "S" on the success branch and "F" on the failure one.
fn save_and_report() -> Vec<u8> {
    cat(&[skip_if(op::op0(op::SAVE), true, &print("F")), print("S")])
}

fn program(code: Vec<Vec<u8>>) -> Vec<u8> {
    let mut story = Story::new();
    story.main(&cat(&[cat(&code), quit()]));
    story.build()
}

// §15 save (version 3): "branch if successful". contracts/core-api.md:
// `SaveRequested` carries complete Quetzal bytes and `save_result` decides the
// branch.
#[test]
fn save_asks_the_host_and_branches_on_its_answer() {
    let image = program(vec![print("a "), save_and_report()]);

    let stored = drive(image.clone(), Host::default());
    assert_eq!(stored.output, "a S\n");
    assert_eq!(stored.saves.len(), 1);
    assert_eq!(&stored.saves[0][..4], b"FORM");
    assert_eq!(&stored.saves[0][8..12], b"IFZS");

    let not_stored = drive(
        image,
        Host {
            save_answers: vec![false],
            ..Host::default()
        },
    );
    assert_eq!(not_stored.output, "a FS\n");
}

// The same position saved twice gives the same bytes: a save is a pure function of
// the game's state.
#[test]
fn saving_is_deterministic() {
    let image = program(vec![op::var(op::PUSH, &[S(7)]), save_and_report()]);
    assert_eq!(
        drive(image.clone(), Host::default()).saves,
        drive(image, Host::default()).saves
    );
}

/// A program that saves inside a routine, changes a global, a local and a header
/// byte, and restores. The host supplies the save once and then declines, so the
/// program runs: save, restore (succeeds), restore again (declined), finish.
///
/// It prints the routine's local and global 0 each time it comes out of `save`,
/// then, after the declined restore, global 0, the Standard revision byte, Flags 2,
/// the routine's result, and a value pushed before the call.
fn save_restore_program() -> Vec<u8> {
    let mut story = Story::new();
    let failed_save = cat(&[print("savefail "), op::op0(op::RFALSE)]);
    let routine = story.routine(
        &[5],
        &cat(&[
            skip_if(op::op0(op::SAVE), true, &failed_save),
            print("out:"),
            print_num(V(1)),
            print(","),
            print_num(V(g(0))),
            print(" "),
            // Change everything a restore should put back...
            op::op2(op::STORE, S(g(0)), S(2)),
            op::op2(op::STORE, S(1), S(9)),
            op::var(op::PUSH, &[S(99)]),
            // ...and two things it should not: the interpreter's header byte and
            // Flags 2 bit 1.
            op::var(op::STOREB, &[S(0), S(0x32), S(9)]),
            op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
            // restore ?(next instruction): on failure, carry on.
            cat(&[op::op0(op::RESTORE), op::branch(true, 2)]),
            print("declined "),
            op::op1(op::RET, S(33)),
        ]),
    );
    story.main(&cat(&[
        op::op2(op::STORE, S(g(0)), S(1)),
        op::var(op::STOREB, &[S(0), S(0x32), S(7)]),
        op::var(op::PUSH, &[S(42)]),
        cat(&[op::var(op::CALL, &[L(routine), S(6)]), op::store(g(1))]),
        print_num(V(g(0))),
        print(" rev:"),
        cat(&[op::op2(op::LOADB, S(0), S(0x32)), op::store(0)]),
        print_num(V(0)),
        print(" flags:"),
        cat(&[op::op2(op::LOADB, S(0), S(0x11)), op::store(0)]),
        print_num(V(0)),
        print(" result:"),
        print_num(V(g(1))),
        print(" stack:"),
        print_num(V(0)),
        quit(),
    ]));
    story.build()
}

// §15 restore (version 3) and Quetzal §5.8.1: a successful restore resumes inside
// the saved `save` instruction with its branch taken, with memory and the call
// stack as they were. §6.1.2: the interpreter's header fields are re-applied and
// Flags 2 keeps its current bits.
#[test]
fn restore_resumes_in_the_saved_save_with_state_as_it_was() {
    let host = Host {
        restore_answers: vec![Some(None)],
        ..Host::default()
    };
    let outcome = drive(save_restore_program(), host);
    assert_eq!(outcome.saves.len(), 1, "the game saved once");
    assert_eq!(outcome.restores_requested, 2);
    // One restore was carried out and one declined: one jump
    // (specs/012-map-first-slice/research.md R3).
    assert_eq!(outcome.jumps, 1);
    assert_eq!(
        outcome.output,
        // First time out of save: the argument 6 in the local, global 0 is 1.
        "out:6,1 \
         out:6,1 \
         declined 2 rev:9 flags:2 result:33 stack:42\n",
        "\nsecond 'out': the local and the global are back to their saved values\n\
         rev: after the second pass the game wrote 9 again, so 9; see the next test\n"
    );
}

// §6.1.2: after a restore the interpreter's own header fields are what the
// interpreter says, not what the save held. The game wrote 7 into the Standard
// revision byte before saving; straight after the restore it must read 1.
#[test]
fn restore_reapplies_the_interpreter_header_fields_and_keeps_flags_2() {
    let mut story = Story::new();
    let failed_save = cat(&[print("savefail"), quit()]);
    let after_restore = cat(&[
        // Reached with g0 = 0 the first time and again after the restore.
        print("rev:"),
        cat(&[op::op2(op::LOADB, S(0), S(0x32)), op::store(0)]),
        print_num(V(0)),
        print(" flags:"),
        cat(&[op::op2(op::LOADB, S(0), S(0x11)), op::store(0)]),
        print_num(V(0)),
        print(" "),
    ]);
    story.main(&cat(&[
        op::var(op::STOREB, &[S(0), S(0x32), S(7)]),
        skip_if(op::op0(op::SAVE), true, &failed_save),
        after_restore,
        // Set Flags 2 bit 1, which the save did not have, then restore.
        op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
        cat(&[op::op0(op::RESTORE), op::branch(true, 2)]),
        quit(),
    ]));
    let host = Host {
        restore_answers: vec![Some(None)],
        ..Host::default()
    };
    let outcome = drive(story.build(), host);
    assert_eq!(outcome.output, "rev:7 flags:0 rev:1 flags:2 \n");
}

// contracts/core-api.md, FR-017: a declined restore, or one given bytes that are
// malformed or belong to another story, takes the failure branch and leaves the
// game in progress untouched.
#[test]
fn a_refused_restore_leaves_the_game_untouched() {
    let other_story = {
        let mut story = Story::new();
        story.serial("999999");
        story.main(&cat(&[save_and_report(), quit()]));
        drive(story.build(), Host::default()).saves.remove(0)
    };
    let declined = drive(save_restore_program(), Host::default());
    assert_eq!(
        declined.output,
        "out:6,1 declined 2 rev:9 flags:2 result:33 stack:42\n"
    );
    // A restore that was declined is no jump.
    assert_eq!(declined.jumps, 0);

    let mut truncated = declined.saves[0].clone();
    truncated.truncate(truncated.len() - 3);
    let mut corrupt = declined.saves[0].clone();
    corrupt[1] = b'X';
    for (what, bytes) in [
        ("another story's save", other_story),
        ("a truncated save", truncated),
        ("a corrupted save", corrupt),
        ("an empty file", Vec::new()),
        (
            "not a save at all",
            b"Dear diary, today I got lost in a maze.".to_vec(),
        ),
    ] {
        let host = Host {
            restore_answers: vec![Some(Some(bytes))],
            ..Host::default()
        };
        let outcome = drive(save_restore_program(), host);
        assert_eq!(outcome.output, declined.output, "{what}");
        assert_eq!(outcome.restores_requested, 1, "{what}");
    }
}

// A save can be restored into a fresh machine running the same story, which is how
// a player comes back to a game later.
#[test]
fn a_save_restores_into_a_fresh_machine() {
    // First pass (g0 = 0): set g1, save. A fresh machine that restores comes out of
    // the save with g1 set although it never ran the code that set it.
    let mut story = Story::new();
    let failed_save = cat(&[print("savefail"), quit()]);
    story.main(&cat(&[
        // restore ?(next): in a fresh machine with a save offered, this never returns.
        cat(&[op::op0(op::RESTORE), op::branch(true, 2)]),
        op::op2(op::STORE, S(g(1)), L(1234)),
        op::var(op::PUSH, &[S(77)]),
        skip_if(op::op0(op::SAVE), true, &failed_save),
        print("g1="),
        print_num(V(g(1))),
        print(" top="),
        print_num(V(0)),
        quit(),
    ]));
    let image = story.build();
    let first = drive(image.clone(), Host::default());
    assert_eq!(first.output, "g1=1234 top=77\n");

    let host = Host {
        restore_answers: vec![Some(Some(first.saves[0].clone()))],
        ..Host::default()
    };
    let second = drive(image, host);
    assert_eq!(second.output, "g1=1234 top=77\n");
    assert!(
        second.saves.is_empty(),
        "the fresh machine never executed the save itself"
    );
}

// research.md R2 and R7: "Restore does not touch the generator." Six numbers drawn
// around a save and a successful restore are the same six a plain run draws.
#[test]
fn restore_does_not_touch_the_random_generator() {
    let draw = || {
        cat(&[
            cat(&[op::var(op::RANDOM, &[L(1000)]), op::store(0)]),
            print_num(V(0)),
            print(" "),
        ])
    };
    let plain = program(vec![draw(), draw(), draw(), draw(), draw(), draw()]);
    let expected = drive(plain, Host::default()).output;

    let failed_save = cat(&[print("savefail"), quit()]);
    let with_restore = program(vec![
        draw(),
        draw(),
        skip_if(op::op0(op::SAVE), true, &failed_save),
        draw(),
        draw(),
        // Succeeds once (back to just after the save), then is declined.
        cat(&[op::op0(op::RESTORE), op::branch(true, 2)]),
    ]);
    let host = Host {
        restore_answers: vec![Some(None)],
        ..Host::default()
    };
    assert_eq!(drive(with_restore, host).output, expected);
}

// contracts/core-api.md: each answer is valid only immediately after its event.
#[test]
fn save_and_restore_answers_out_of_turn_are_refused() {
    let image = program(vec![
        save_and_report(),
        cat(&[op::op0(op::RESTORE), op::branch(true, 2)]),
    ]);
    let mut m = Machine::load(image, OPTIONS).unwrap();
    assert_eq!(m.save_result(true), Err(HostError::NotWaitingForThis));
    assert_eq!(m.provide_restore(None), Err(HostError::NotWaitingForThis));

    assert!(matches!(m.run(BIG), Ok(Event::SaveRequested(_))));
    assert_eq!(m.run(BIG), Err(HostError::AnswerOwed));
    assert_eq!(m.provide_restore(None), Err(HostError::NotWaitingForThis));
    assert_eq!(m.provide_line("x"), Err(HostError::NotWaitingForThis));
    m.save_result(true).unwrap();
    assert_eq!(m.save_result(true), Err(HostError::NotWaitingForThis));

    assert_eq!(m.run(BIG), Ok(Event::RestoreRequested));
    assert_eq!(m.run(BIG), Err(HostError::AnswerOwed));
    assert_eq!(m.save_result(true), Err(HostError::NotWaitingForThis));
    m.provide_restore(None).unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output(), "S\n");
}

// A save taken while a transcript is on can be restored while one is on: the
// current Flags 2 bit is kept (§6.1.2), so the transcript carries on.
#[test]
fn a_transcript_survives_a_restore() {
    let failed_save = cat(&[print("savefail"), quit()]);
    let image = program(vec![
        // The save is taken with transcription off.
        skip_if(op::op0(op::SAVE), true, &failed_save),
        print("after save\n"),
        // Turn it on, then restore to the state that had it off.
        op::var(op::STOREW, &[S(0), S(8), S(1)]),
        print("recorded\n"),
        cat(&[op::op0(op::RESTORE), op::branch(true, 2)]),
        print("end\n"),
    ]);
    let mut m = Machine::load(image, OPTIONS).unwrap();
    let mut transcript = String::new();
    let mut saved = Vec::new();
    let mut restores = 0;
    loop {
        let event = m.run(BIG).unwrap();
        transcript.push_str(&m.take_transcript());
        match event {
            Event::Finished => break,
            Event::SaveRequested(bytes) => {
                saved = bytes;
                m.save_result(true).unwrap();
            }
            Event::TranscriptRequested => m.transcript_result(true).unwrap(),
            Event::RestoreRequested => {
                restores += 1;
                m.provide_restore(if restores == 1 { Some(&saved) } else { None })
                    .unwrap();
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(transcript, "recorded\nafter save\nrecorded\nend\n");
}
