//! Directions and trials (specs/005-corpus-testing/research.md R1, R2, R4;
//! spec.md FR-001, FR-003, FR-004).

mod common;

use common::*;
use zmachine_core::{Event, Machine};
use zmachine_corpus::directions::{Direction, directions};
use zmachine_corpus::trial::{LIMIT, Outcome, trial};
use zmachine_introspect::Introspector;

/// Runs a machine to its first line prompt and returns where it is waiting.
fn to_prompt(machine: &mut Machine) -> u32 {
    loop {
        match machine.run(1_000_000).unwrap() {
            Event::BudgetExhausted => continue,
            Event::LineWanted { .. } => break,
            other => panic!("the game stopped for {other:?}"),
        }
    }
    machine.take_output();
    machine.waiting_at().unwrap()
}

fn introspector(machine: &Machine) -> Introspector {
    Introspector::new(&machine.view())
}

fn word(number: u8, word: &str) -> Direction {
    Direction {
        number,
        word: Some(word.to_string()),
    }
}

// R4: the distinct direction numbers of the dictionary, ascending, each with its
// longest word.
#[test]
fn a_storys_directions_are_its_dictionarys_in_ascending_order() {
    let machine = Toy::new(2).machine();
    let found = directions(&introspector(&machine), &machine.view());
    assert_eq!(
        found,
        [
            word(DOWN, "down"),
            word(UP, "up"),
            word(WEST, "west"),
            word(EAST, "east"),
            word(SOUTH, "south"),
            // `north` and `n` name one direction: the longest is sent.
            word(NORTH, "north"),
        ]
    );
}

// R4: among words of equal length, the first in alphabetical order.
#[test]
fn zork_i_has_twelve_directions_each_with_one_word() {
    let machine = load("stories/zork1/zork1.z3", 11);
    let found = directions(&introspector(&machine), &machine.view());
    let words: Vec<&str> = found.iter().filter_map(|d| d.word.as_deref()).collect();
    for expected in [
        "north", "northe", "northw", "south", "down", "up", "east", "west",
    ] {
        assert!(words.contains(&expected), "{expected} in {words:?}");
    }
    let mut numbers: Vec<u8> = found.iter().map(|d| d.number).collect();
    assert!(
        numbers.windows(2).all(|pair| pair[0] < pair[1]),
        "{numbers:?}"
    );
    numbers.dedup();
    assert_eq!(numbers.len(), found.len());
    assert!(found.len() >= 12, "{found:?}");
}

// R4: a story of no family has no directions, so no trials are run on it.
#[test]
fn a_story_of_no_family_has_no_directions() {
    let mut toy = Toy::new(2);
    toy.direction_words = false;
    let machine = toy.machine();
    assert_eq!(directions(&introspector(&machine), &machine.view()), []);
}

/// The outcome of one word tried at the toy's first prompt.
fn outcome(toy: &Toy, word: &str, limit: u64) -> Outcome {
    let mut machine = toy.machine();
    let address = to_prompt(&mut machine);
    trial(&machine, &introspector(&machine), address, word, limit).0
}

// R2, row by row.
#[test]
fn a_trial_that_changes_room_moved_to_that_room() {
    let mut toy = Toy::new(3);
    toy.way(0, NORTH, 2);
    assert_eq!(outcome(&toy, "north", LIMIT), Outcome::Moved(Toy::room(2)));
}

#[test]
fn a_trial_that_leaves_the_room_as_it_was_stayed() {
    let toy = Toy::new(2);
    assert_eq!(outcome(&toy, "north", LIMIT), Outcome::Stayed);
    // A word that is not a direction at all.
    assert_eq!(outcome(&toy, "look", LIMIT), Outcome::Stayed);
}

#[test]
fn a_trial_that_meets_a_question_at_another_address_is_a_question() {
    let mut toy = Toy::new(2);
    toy.does(0, EAST, ASK);
    assert_eq!(outcome(&toy, "east", LIMIT), Outcome::Question);
}

#[test]
fn a_trial_on_which_the_game_quits_ended() {
    let mut toy = Toy::new(2);
    toy.does(0, WEST, QUIT);
    assert_eq!(outcome(&toy, "west", LIMIT), Outcome::Ended);
}

#[test]
fn a_trial_on_which_the_machine_faults_is_a_fault() {
    let mut toy = Toy::new(2);
    toy.does(0, UP, BOOM);
    assert_eq!(outcome(&toy, "up", LIMIT), Outcome::Fault);
}

#[test]
fn a_trial_that_never_stops_is_a_fault_at_the_limit() {
    let mut toy = Toy::new(2);
    toy.does(0, DOWN, SPIN);
    assert_eq!(outcome(&toy, "down", 100_000), Outcome::Fault);
}

#[test]
fn a_trial_where_the_location_is_unknown_is_lost() {
    let mut toy = Toy::new(2);
    toy.direction_words = false;
    toy.does(0, NORTH, Toy::room(1) as u8);
    assert_eq!(outcome(&toy, "north", LIMIT), Outcome::Lost);
}

// R2: the copy a trial returns is at the place the trial reached, with nothing
// left to read from it.
#[test]
fn the_copy_returned_is_where_the_trial_ended() {
    let mut toy = Toy::new(3);
    toy.way(0, NORTH, 1).way(1, NORTH, 2);
    let mut machine = toy.machine();
    let address = to_prompt(&mut machine);
    let i = introspector(&machine);
    let (first, mut copy) = trial(&machine, &i, address, "north", LIMIT);
    assert_eq!(first, Outcome::Moved(Toy::room(1)));
    assert_eq!(copy.take_output(), "");
    assert_eq!(copy.waiting_at(), Some(address));
    let (second, _) = trial(&copy, &i, address, "north", LIMIT);
    assert_eq!(second, Outcome::Moved(Toy::room(2)));
}

/// What a host can see of a machine over its next `prompts` commands.
fn seen(mut machine: Machine, lines: &[&str]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in lines {
        machine.provide_line(line).unwrap();
        let event = loop {
            match machine.run(1_000_000).unwrap() {
                Event::BudgetExhausted => continue,
                other => break other,
            }
        };
        out.push((format!("{event:?}"), machine.take_output()));
    }
    out
}

// FR-001: after any number of trials the game copied from is unchanged: the
// same text, the same stops and the same random numbers as a twin that had no
// trials run from it.
#[test]
fn trials_leave_the_machine_they_were_copied_from_alone() {
    let mut machine = load("stories/zork1/zork1.z3", 11);
    let address = to_prompt(&mut machine);
    let twin = machine.clone();
    let i = introspector(&machine);
    let found = directions(&i, &machine.view());
    assert!(!found.is_empty());
    let mut moved = 0;
    for direction in &found {
        let word = direction.word.as_deref().unwrap();
        let (outcome, _) = trial(&machine, &i, address, word, LIMIT);
        moved += matches!(outcome, Outcome::Moved(_)) as usize;
    }
    assert!(moved >= 3, "some trials moved the copy: {moved}");
    // Twenty commands, several of which draw random numbers (the thief, combat).
    let lines = [
        "open mailbox",
        "read leaflet",
        "north",
        "east",
        "open window",
        "in",
        "west",
        "take lamp",
        "take sword",
        "move rug",
        "open trap door",
        "turn on lamp",
        "down",
        "north",
        "kill troll with sword",
        "kill troll with sword",
        "kill troll with sword",
        "look",
        "inventory",
        "diagnose",
    ];
    assert_eq!(seen(machine, &lines), seen(twin, &lines));
}
