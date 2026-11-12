//! Tests of the walkthrough driver itself, before anything is measured with it
//! (specs/003-introspection/tasks.md T015).

mod common;

use common::*;

// The driver plays each walkthrough exactly as the command-line host does: the
// transcript is the committed reference, byte for byte.
#[test]
fn the_driver_reproduces_the_reference_transcripts() {
    for game in GAMES {
        let replay = walk(game, |_, _| {});
        assert_eq!(replay.transcript, read_text(game.reference), "{}", game.name);
    }
}

// The number of command prompts in each walkthrough, recorded here so that tests
// which say "at every prompt" are known to cover them all.
#[test]
fn the_walkthroughs_have_the_expected_number_of_command_prompts() {
    let counts: Vec<(usize, usize)> = GAMES
        .iter()
        .map(|&game| {
            let replay = walk(game, |_, _| {});
            (replay.command_prompts, replay.stops)
        })
        .collect();
    assert!(counts[0].0 >= 320, "Zork I: {:?}", counts[0]);
    assert!(counts[1].0 >= 165, "Advent: {:?}", counts[1]);
    assert_eq!(counts[1], counts[2], "the two builds of Advent stop at the same places");
    // Recorded: (command prompts, stops of every kind).
    assert_eq!(counts, [(332, 334), (172, 174), (172, 174)]);
}

// Each stop tells the hook which script line answers it; the first command
// prompt of each game is answered by line 1, `script`.
#[test]
fn stops_carry_their_script_line_and_the_games_text() {
    for game in GAMES {
        let mut first = None;
        let mut headings = 0;
        walk(game, |_, stop| {
            if first.is_none() {
                first = Some(stop.clone());
            }
            if stop.command_prompt && !first_line(&stop.text).is_empty() {
                headings += 1;
            }
        });
        let first = first.unwrap();
        assert_eq!(first.number, 1);
        assert_eq!(first.wants, Wants::Line);
        assert!(first.command_prompt, "{}: {:?}", game.name, first.text);
        assert_eq!(first.line, Some((1, "script".to_string())));
        assert!(headings > 100, "{}", game.name);
        if game.name != "zork1" {
            assert!(first.status_row.contains("At End Of Road"), "{:?}", first.status_row);
        }
    }
}

// The copy helper: the game's reply to `inventory` at the very first prompt is
// its empty reply, and asking leaves the machine as it was.
#[test]
fn asking_a_copy_leaves_the_machine_alone() {
    for game in GAMES {
        let mut asked = false;
        let replay = walk(game, |machine, stop| {
            if stop.number == 1 {
                assert_eq!(ask(machine, game.inventory), game.empty_handed, "{}", game.name);
                asked = true;
            } else if stop.command_prompt && stop.number % 40 == 0 {
                let reply = ask(machine, game.inventory);
                assert!(!reply.is_empty());
            }
        });
        assert!(asked);
        assert_eq!(replay.transcript, read_text(game.reference), "{}: asking copies changed nothing", game.name);
    }
}

// Save scripts: the driver collects the save files the game writes.
#[test]
fn the_driver_collects_save_files() {
    let replay = replay(ZORK1.story, "stories/zork1/fidelity/saves/make_1.txt", ZORK1.seed, |_, _| {});
    assert_eq!(replay.saves.len(), 1);
    assert_eq!(replay.saves[0].0, "save_1.qzl");
    assert_eq!(&replay.saves[0].1[..4], b"FORM");
}
