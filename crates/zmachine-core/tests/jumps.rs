//! The count of jumps (specs/012-map-first-slice/research.md R3): how many times
//! a game has restarted itself, restored a position or undone a turn. A host
//! that keeps a record of the player's moves reads it to know that a change of
//! place was not a move. The count of restores is tested in `save.rs`, beside
//! the programs that restore.

mod common;

use common::*;
use zmachine_core::{Event, LoadOptions, Machine, Settings};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Story, Story5};

fn save_undo(variable: u8) -> Vec<u8> {
    to(op::ext(op::SAVE_UNDO, &[]), variable)
}

fn restore_undo(variable: u8) -> Vec<u8> {
    to(op::ext(op::RESTORE_UNDO, &[]), variable)
}

// A game that does neither has jumped no times, at the start and at the end.
#[test]
fn a_game_that_only_runs_has_not_jumped() {
    let mut story = Story::new();
    story.main(&cat(&[print("hello"), quit()]));
    let mut m = Machine::load(story.build(), OPTIONS).unwrap();
    assert_eq!(m.jumps(), 0);
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.jumps(), 0);
}

// §15 restart: one jump each time it is carried out.
#[test]
fn a_restart_is_a_jump() {
    // On the first pass (Flags 2 bit 1 clear) set the bit and restart; on the
    // second, quit. The bit survives a restart (§6.1.3).
    let first_pass = cat(&[
        op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
        op::op0(op::RESTART),
    ]);
    let mut story = Story::new();
    story.main(&cat(&[
        cat(&[op::op2(op::LOADB, S(0), S(0x11)), op::store(0)]),
        skip_if(op::op2(op::TEST, V(0), S(2)), true, &first_pass),
        quit(),
    ]));
    let mut m = Machine::load(story.build(), OPTIONS).unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.jumps(), 1);
}

// §15 restore_undo: a jump when it returns to a position, and none when there
// is none to return to or undo is not offered.
#[test]
fn an_undo_is_a_jump_only_when_it_happens() {
    let undoing = |story: &mut Story5| {
        let done = cat(&[print("undone"), quit()]);
        let routine = story.routine(
            1,
            &cat(&[
                save_undo(g(0)),
                skip_if(op::op2(op::JE, V(g(0)), S(2)), false, &done),
                restore_undo(g(2)),
                print("not undone"),
                quit(),
            ]),
        );
        story.main(&cat(&[op::op1(op::CALL_1N, L(routine)), quit()]));
    };
    let mut story = Story5::new(5);
    undoing(&mut story);
    let mut m = Machine::load(story.build(), OPTIONS).unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.take_output().trim_end(), "undone");
    assert_eq!(m.jumps(), 1);

    // With no position saved.
    let mut story = Story5::new(5);
    story.main(&cat(&[restore_undo(g(0)), quit()]));
    let mut m = Machine::load(story.build(), OPTIONS).unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.jumps(), 0);

    // With undo not offered: the game is told so, and nothing is undone.
    let mut story = Story5::new(5);
    undoing(&mut story);
    let settings = Settings {
        undo_slots: 0,
        ..Settings::default()
    };
    let options = LoadOptions {
        seed: Some(1),
        entropy: 0,
    };
    let mut m = Machine::load_with(story.build(), options, settings).unwrap();
    assert_eq!(m.run(BIG), Ok(Event::Finished));
    assert_eq!(m.jumps(), 0);
}

// A copy of a game carries the count with it, and counts on by itself.
#[test]
fn a_copy_keeps_the_count() {
    let mut story = Story::new();
    story.main(&cat(&[print("hello"), quit()]));
    let m = Machine::load(story.build(), OPTIONS).unwrap();
    assert_eq!(m.clone().jumps(), m.jumps());
}
