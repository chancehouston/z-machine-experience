//! A wrong report fails until it is explained (specs/005-corpus-testing/spec.md
//! User Story 2; FR-013, FR-014; quickstart 5).

mod common;

use common::*;
use zmachine_corpus::corpus::{Mode, command};

/// A story whose table says north leads to room 1 and whose code goes to room 2.
fn wrong_north(scratch: &Scratch) {
    let mut toy = Toy::new(3);
    toy.table(0, NORTH, vec![Toy::room(1) as u8])
        .does(0, NORTH, Toy::room(2) as u8);
    scratch.write("stories/toy/walk.txt", b"look\n");
    scratch.add("toy", &toy.build(), "walkthrough: stories/toy/walk.txt\n");
}

const KNOWN: &str = "\
story: toy
room: 2
direction: 31
reported: fixed 3
outcome: moved 4
cause: The code ignores the table here, on purpose, for this test.
";

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|l| l.to_string()).collect()
}

// US2 scenario 1; FR-013: an unexplained wrong case fails the run and is named
// by story, room, direction, what was reported and what happened, in numbers.
#[test]
fn an_unexplained_wrong_case_fails_and_is_named() {
    let scratch = Scratch::new("unexplained");
    wrong_north(&scratch);
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]),
        (lines(&["toy written"]), 0),
        "writing records the case; it does not judge it"
    );
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    assert!(
        kept.contains("\nfixed confirmed 0 wrong-known 0 wrong-unexplained 2\n"),
        "{kept}"
    );
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (lines(&["toy wrong 2 31 fixed 3 moved 4"]), 1)
    );
    // Writing does not touch the ledger.
    assert_eq!(scratch.read("stories/corpus/ledger.txt"), "");
}

// US2 scenario 2: with the case in the ledger it is known, and the run passes.
#[test]
fn a_case_in_the_ledger_is_known_and_the_run_passes() {
    let scratch = Scratch::new("known");
    wrong_north(&scratch);
    scratch.write("stories/corpus/ledger.txt", KNOWN.as_bytes());
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]),
        (lines(&["toy written"]), 0)
    );
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    assert!(
        kept.contains("\nfixed confirmed 0 wrong-known 2 wrong-unexplained 0\n"),
        "{kept}"
    );
    assert_eq!(command(&scratch.0, Mode::Run, &[]), (lines(&["toy ok"]), 0));
}

// US2 scenario 3; FR-014: an entry that no trial matches fails the run and is
// named by its line in the ledger.
#[test]
fn an_entry_that_matches_no_trial_fails_and_is_named() {
    let scratch = Scratch::new("stale");
    wrong_north(&scratch);
    let stale = format!(
        "{KNOWN}\n{}",
        KNOWN.replace("direction: 31", "direction: 30")
    );
    scratch.write("stories/corpus/ledger.txt", stale.as_bytes());
    command(&scratch.0, Mode::Write, &[]);
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (
            lines(&[
                "toy ok",
                "stories/corpus/ledger.txt:8: the entry matches no trial"
            ]),
            1
        )
    );
}

// FR-014: an entry for a story that is absent is not counted as stale; one for
// a story the manifest does not have is refused.
#[test]
fn entries_for_absent_stories_wait_and_entries_for_no_story_are_refused() {
    let scratch = Scratch::new("absent-entry");
    wrong_north(&scratch);
    scratch.write("stories/corpus/ledger.txt", KNOWN.as_bytes());
    command(&scratch.0, Mode::Write, &[]);
    std::fs::remove_file(scratch.path("stories/toy/toy.z3")).unwrap();
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (lines(&["toy skipped"]), 0)
    );

    let other = KNOWN.replace("story: toy", "story: nosuch");
    scratch.write("stories/corpus/ledger.txt", other.as_bytes());
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (
            lines(&["stories/corpus/ledger.txt:1: `nosuch` is not in the manifest"]),
            1
        )
    );
}

// US2 scenario 4; FR-014: an entry with no cause fails.
#[test]
fn an_entry_with_no_cause_fails() {
    let scratch = Scratch::new("causeless");
    wrong_north(&scratch);
    let causeless = KNOWN.replace(
        "cause: The code ignores the table here, on purpose, for this test.",
        "cause: ",
    );
    scratch.write("stories/corpus/ledger.txt", causeless.as_bytes());
    let (said, status) = command(&scratch.0, Mode::Run, &[]);
    assert_eq!(status, 1);
    assert_eq!(said.len(), 1);
    assert!(
        said[0].starts_with("stories/corpus/ledger.txt:6: "),
        "{said:?}"
    );
}
