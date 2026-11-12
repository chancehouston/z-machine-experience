//! The crawl: reaching places by walking, within a budget, the same every time
//! (specs/005-corpus-testing/research.md R5, R6; spec.md User Story 3).

mod common;

use common::*;
use zmachine_corpus::ledger::Ledger;
use zmachine_corpus::manifest::{Story, Tier};
use zmachine_corpus::report::Reached;
use zmachine_corpus::walk::{ROOMS, TRIALS, run_story, run_story_traced};

fn entry() -> Story {
    Story {
        id: "toy".to_string(),
        sha256: "0".repeat(64),
        size: 0,
        version: 3,
        family: "zil".to_string(),
        source: "built in memory".to_string(),
        standing: "a test's own".to_string(),
        tier: Tier::Committed,
        path: String::new(),
        seed: 1,
        walkthrough: None,
        opening: vec![],
        rooms: None,
        trials: None,
        line: 1,
    }
}

/// Six rooms: 0 north to 1 and east to 2; 1 north to 3; 2 east to 4; 3 east to
/// 5; 4 north to 5. In room 0 the game quits on west, asks a question on up and
/// faults on down; none of the three is in the table.
fn six_rooms() -> Toy {
    let mut toy = Toy::new(6);
    toy.way(0, NORTH, 1)
        .way(0, EAST, 2)
        .way(1, NORTH, 3)
        .way(2, EAST, 4)
        .way(3, EAST, 5)
        .way(4, NORTH, 5)
        .does(0, WEST, QUIT)
        .does(0, UP, ASK)
        .does(0, DOWN, BOOM);
    toy
}

fn rooms(indexes: &[usize]) -> Vec<u16> {
    indexes.iter().map(|&i| Toy::room(i)).collect()
}

// R5; FR-009, FR-012; US3 scenarios 1 and 3. Directions are tried in ascending
// order of number (down, up, west, east, south, north) and places are taken
// first in, first out.
#[test]
fn the_crawl_visits_every_room_once_nearest_first() {
    let (report, order) = run_story_traced(&entry(), six_rooms().build(), None, &Ledger::default());
    assert_eq!(order, rooms(&[0, 2, 1, 4, 3, 5]));
    assert_eq!((report.places, report.walkthrough, report.crawl), (6, 0, 6));
    assert_eq!((report.rooms_visited, report.named_not_reached), (6, 0));
    assert_eq!(report.trials, 36);
    // Six ways honoured, one of them to a room already visited.
    assert_eq!(report.fixed_confirmed, 6);
    // The three branches that do not come back to a command prompt are counted
    // under what happened, and nothing lies beyond them.
    assert_eq!(report.none_other, 3);
    assert_eq!(report.none_confirmed, 27);
    assert_eq!(report.verdicts(), report.trials);
    assert_eq!(report.reached, Reached::No);
    assert_eq!((report.budget_rooms, report.budget_trials), (ROOMS, TRIALS));
}

// FR-010; US3 scenario 5: the budget ends the crawl, and the report says which
// limit did.
#[test]
fn the_budget_ends_the_crawl_and_the_report_says_so() {
    let mut story = entry();
    story.rooms = Some(3);
    let (report, order) = run_story_traced(&story, six_rooms().build(), None, &Ledger::default());
    assert_eq!(order, rooms(&[0, 2, 1]));
    assert_eq!(report.crawl, 3);
    assert_eq!(report.reached, Reached::Rooms);
    assert_eq!(report.budget_rooms, 3);
    assert_eq!(report.rooms_visited, 3);
    // Rooms 3 and 4 are named by exits of rooms visited, and were not reached.
    assert_eq!(report.named_not_reached, 2);

    let mut story = entry();
    story.trials = Some(10);
    let report = run_story(&story, six_rooms().build(), None, &Ledger::default());
    assert_eq!(report.trials, 10);
    assert_eq!(report.reached, Reached::Trials);
    assert_eq!(report.verdicts(), 10);

    // A budget that is exactly enough is not reached.
    let mut story = entry();
    story.rooms = Some(6);
    story.trials = Some(36);
    let report = run_story(&story, six_rooms().build(), None, &Ledger::default());
    assert_eq!((report.crawl, report.trials), (6, 36));
    assert_eq!(report.reached, Reached::No);
}

// FR-011; US3 scenario 2; SC-004. The toy draws a random number on every move.
#[test]
fn the_same_story_seed_and_budget_give_the_same_report() {
    let first = run_story(&entry(), six_rooms().build(), None, &Ledger::default());
    let second = run_story(&entry(), six_rooms().build(), None, &Ledger::default());
    assert_eq!(first.write(), second.write());
    assert!(first.trials > 0);
}

// R5, item 2; US3 scenario 4: the crawl goes outward from the places along a
// walkthrough, once the walkthrough has finished. A room the walkthrough
// reaches itself is not crawled a second time.
#[test]
fn the_crawl_goes_on_from_a_walkthrough() {
    // 0 north to 1, 1 north to 2, and a side room 3 east of 1, with 4 beyond it.
    let mut toy = Toy::new(5);
    toy.way(0, NORTH, 1)
        .way(1, NORTH, 2)
        .way(1, EAST, 3)
        .way(3, EAST, 4);
    let (report, order) = run_story_traced(
        &entry(),
        toy.build(),
        Some("north\nnorth\n"),
        &Ledger::default(),
    );
    // Three prompts of the walkthrough: rooms 0, 1 and 2. Room 2 was first seen
    // by a trial from room 1, and then the walkthrough went there itself.
    assert_eq!(report.walkthrough, 3);
    assert_eq!(order, rooms(&[3, 4]));
    assert_eq!(report.crawl, 2);
    assert_eq!(report.places, 5);
    assert_eq!(report.rooms_visited, 5);
    assert_eq!(report.trials, 30);
}

// R6: a story that asks something before its first command prompt is started
// with its opening lines, and the command prompt is the first one after them.
#[test]
fn opening_lines_answer_what_a_game_asks_first() {
    // The toy asks nothing first, so an opening line is its first command: with
    // `north` given as one, the first place is the room north of the start.
    let mut toy = Toy::new(2);
    toy.way(0, NORTH, 1);
    let mut story = entry();
    let plain = run_story(&story, toy.build(), None, &Ledger::default());
    story.opening = vec!["north".to_string()];
    let (opened, order) = run_story_traced(&story, toy.build(), None, &Ledger::default());
    // The opening line was given before the first place: the crawl starts in room 1.
    assert_eq!(order, rooms(&[1]));
    assert_eq!(plain.crawl, 2);
    assert_eq!(opened.crawl, 1);
}
