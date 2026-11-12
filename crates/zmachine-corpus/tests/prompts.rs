//! The driver, and the rule that tells a command prompt from a question
//! (specs/005-corpus-testing/research.md R3, R6; tasks.md T006 to T008).

mod common;

use common::*;
use zmachine_corpus::walk::{Driver, Wants, walkthrough};

// R3: a prompt is a command prompt when the game waits where it waited at its
// first line prompt; a line wanted anywhere else is a question.
#[test]
fn a_question_at_another_address_is_not_a_command_prompt() {
    let mut toy = Toy::new(2);
    toy.does(0, EAST, ASK);
    let mut driver = Driver::new(toy.build(), 1).unwrap();
    assert_eq!(driver.stop(), Wants::Command);
    let address = driver.command_address;
    assert!(address.is_some());
    driver.answer("east");
    assert_eq!(driver.stop(), Wants::Answer);
    assert_eq!(
        driver.command_address, address,
        "taken once, at the first prompt"
    );
    driver.answer("anything");
    assert_eq!(driver.stop(), Wants::Command);
    // And when the game quits there is nothing to answer.
    let mut toy = Toy::new(2);
    toy.does(0, WEST, QUIT);
    let mut driver = Driver::new(toy.build(), 1).unwrap();
    assert_eq!(driver.stop(), Wants::Command);
    driver.answer("west");
    assert_eq!(driver.stop(), Wants::Nothing);
}

// T007: the driver plays each walkthrough as the earlier features' driver does:
// the transcript is the committed reference, byte for byte.
#[test]
fn the_driver_reproduces_the_reference_transcripts() {
    for game in GAMES {
        let replay = walkthrough(
            read(game.story),
            &read_text(game.walkthrough),
            game.seed,
            |_, _| {},
        );
        assert!(
            replay.transcript == read_text(game.reference),
            "{}: the transcript differs from the reference",
            game.name
        );
    }
}

// T008: the earlier features counted command prompts by the game's text (a line
// wanted after text ending in `>`): 332 of 334 stops for Zork I, 172 of 174 for
// each build of Advent (`crates/zmachine-introspect/tests/driver.rs`,
// `the_walkthroughs_have_the_expected_number_of_command_prompts`). By where the
// game is waiting there are fewer, and every difference is a stop at which the
// game is not asking for a command, though it prints `>`:
//
// - Zork I, line 87 of the walkthrough: one room reads its line itself, at
//   another address. Feature 004 met the same stop. No trials are run there.
// - Zork I, line 333, and Advent, line 173: the game is over and asks whether to
//   restart, restore or quit.
#[test]
fn the_rule_counts_the_command_prompts_of_each_walkthrough() {
    let counts: Vec<(usize, usize)> = GAMES
        .iter()
        .map(|game| {
            let mut places = 0;
            let replay = walkthrough(
                read(game.story),
                &read_text(game.walkthrough),
                game.seed,
                |_, _| places += 1,
            );
            assert_eq!(places, replay.command_prompts, "{}", game.name);
            (replay.command_prompts, replay.stops)
        })
        .collect();
    assert_eq!(counts, [(330, 334), (171, 174), (171, 174)]);
}
