//! Inspection changes nothing (docs/introspection.md section 1.1, cited I§1.1;
//! SC-001, FR-036; constitution Principle IV: "Running with introspection on or
//! off MUST produce identical transcripts").
//!
//! Every query takes the machine by shared reference, so the compiler already
//! forbids a write. These tests confirm it on whole games.

mod common;

use common::*;
use zmachine_core::Machine;
use zmachine_introspect::Introspector;

/// Asks all six queries and returns how much they answered, so that the work
/// cannot be optimised away and a silent failure to answer would show.
fn ask_everything(machine: &Machine) -> usize {
    let view = machine.view();
    let i = Introspector::new(&view);
    let mut answered = 0;
    answered += i.object_tree(&view).value.map_or(0, |v| v.len());
    answered += i.dictionary(&view).value.map_or(0, |v| v.len());
    answered += i.location(&view).value.is_ok() as usize;
    answered += i.inventory(&view).value.map_or(0, |v| v.len());
    answered += i.exits(&view).value.map_or(0, |v| v.exits.len());
    answered += i.scope(&view).value.map_or(0, |v| v.objects.len());
    answered
}

fn memory(machine: &Machine) -> Vec<u8> {
    let view = machine.view();
    (0..view.len()).map(|a| view.byte(a).unwrap()).collect()
}

// SC-001, FR-036: each walkthrough, with every query asked at every stop of
// every kind, gives the committed reference transcript byte for byte.
#[test]
fn transcripts_are_unchanged_by_asking_everything_at_every_stop() {
    for game in GAMES {
        let mut answered = 0;
        let mut stops = 0;
        let replay = walk(game, |machine, _| {
            answered += ask_everything(machine);
            stops += 1;
        });
        assert_eq!(
            replay.transcript,
            read_text(game.reference),
            "{}",
            game.name
        );
        assert!(
            stops >= 170 && answered > stops * 500,
            "{}: {answered} answers at {stops} stops",
            game.name
        );
    }
}

// SC-001: the save files written at each committed save point are the same,
// byte for byte, with every query asked at every stop and with none.
#[test]
fn save_files_are_unchanged_by_asking_everything_at_every_stop() {
    let points = [
        (ZORK1, "stories/zork1/fidelity/saves", 5),
        (ADVENT5, "stories/advent/fidelity/saves", 3),
        (ADVENT8, "stories/advent/fidelity/saves", 3),
    ];
    for (game, directory, count) in points {
        for n in 1..=count {
            let script = format!("{directory}/make_{n}.txt");
            let plain = replay(game.story, &script, game.seed, |_, _| {});
            let asked = replay(game.story, &script, game.seed, |machine, _| {
                ask_everything(machine);
            });
            assert_eq!(plain.saves.len(), 1, "{} save point {n}", game.name);
            assert_eq!(asked.saves, plain.saves, "{} save point {n}", game.name);
            assert_eq!(
                asked.transcript, plain.transcript,
                "{} save point {n}",
                game.name
            );
        }
    }
}

// I§1.1: at every stop, the whole of memory is the same before and after the six
// queries, and a copy of the machine made after them plays on exactly as a copy
// made before them: the same text and the same random numbers.
#[test]
fn memory_and_what_follows_are_unchanged_by_the_queries() {
    for game in GAMES {
        let mut checked = 0;
        walk(game, |machine, stop| {
            let before = memory(machine);
            let untouched = machine.clone();
            ask_everything(machine);
            ask_everything(machine);
            assert_eq!(
                memory(machine),
                before,
                "{} stop {}",
                game.name,
                stop.number
            );
            // Every twentieth command prompt, play both copies on by one command
            // that draws on the random generator and prints the room.
            if stop.command_prompt && stop.number % 20 == 0 {
                let after = machine.clone();
                for command in ["look", "wait", "inventory"] {
                    assert_eq!(
                        ask(&after, command),
                        ask(&untouched, command),
                        "{} stop {}: {command}",
                        game.name,
                        stop.number
                    );
                }
                // A control: the comparison above can fail. A copy that is given a
                // command does differ in memory from the state before it.
                let mut moved = machine.clone();
                moved.provide_line("wait").unwrap();
                while matches!(
                    moved.run(1_000_000),
                    Ok(zmachine_core::Event::BudgetExhausted)
                ) {}
                assert_ne!(
                    memory(&moved),
                    before,
                    "{} stop {}: a turn changes memory",
                    game.name,
                    stop.number
                );
                checked += 1;
            }
        });
        assert!(checked >= 8, "{}", game.name);
    }
}

// I§1.7: the same state gives the same answers, from one introspector or from a
// second made later.
#[test]
fn the_same_state_gives_the_same_answers() {
    for game in GAMES {
        let mut first = None;
        walk(game, |machine, stop| {
            if stop.number % 15 != 0 {
                return;
            }
            let view = machine.view();
            let i = first.get_or_insert_with(|| Introspector::new(&view));
            let again = Introspector::new(&view);
            assert_eq!(
                i.location(&view),
                again.location(&view),
                "{} stop {}",
                game.name,
                stop.number
            );
            assert_eq!(i.inventory(&view), again.inventory(&view));
            assert_eq!(i.exits(&view), again.exits(&view));
            assert_eq!(i.scope(&view), again.scope(&view));
            assert_eq!(i.scope(&view), i.scope(&view));
            assert_eq!(i.object_tree(&view), again.object_tree(&view));
        });
        assert!(first.is_some());
    }
}
