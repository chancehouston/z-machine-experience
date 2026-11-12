//! Running a story: verdicts, and the report they add up to
//! (specs/005-corpus-testing/spec.md User Story 1; FR-006 to FR-008).

mod common;

use common::*;
use zmachine_corpus::corpus::{Mode, command};
use zmachine_corpus::ledger::Ledger;
use zmachine_corpus::manifest::{Story, Tier};
use zmachine_corpus::report::{Case, Report};
use zmachine_corpus::trial::Outcome;
use zmachine_corpus::verdict::Reported;
use zmachine_corpus::walk::run_story;

/// A manifest entry for a story built in memory.
pub fn entry(id: &str) -> Story {
    Story {
        id: id.to_string(),
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
        first: vec![],
        rooms: None,
        trials: None,
        line: 1,
    }
}

/// Three rooms. In the first: north is in the table and the code honours it;
/// east is in the table as leading to room 1 but the code goes to room 2; west
/// is not in the table but the code moves anyway; south is conditional (three
/// bytes) and the code stays; up and down are not exits and do nothing.
fn disagreeing() -> Toy {
    let mut toy = Toy::new(3);
    toy.way(0, NORTH, 1)
        .table(0, EAST, vec![Toy::room(1) as u8])
        .does(0, EAST, Toy::room(2) as u8)
        .does(0, WEST, Toy::room(1) as u8)
        .table(0, SOUTH, vec![0x12, 0x34, 0]);
    toy
}

/// An entry whose budget lets the crawl visit nothing: the walkthrough alone.
fn walkthrough_only() -> Story {
    let mut story = entry("toy");
    story.rooms = Some(0);
    story
}

fn count(report: &Report, direction: u8, reported: Reported, outcome: Outcome) -> usize {
    let case = Case {
        room: Toy::room(0),
        direction,
        reported,
        outcome,
    };
    report
        .detail
        .iter()
        .find(|(c, _)| *c == case)
        .map_or(0, |(_, n)| *n)
}

// US1 scenarios 1 to 4; FR-006, FR-007, FR-008. A script of three lines that
// never leave the first room has four command prompts: one before each line and
// one after the last.
#[test]
fn every_direction_at_every_prompt_has_one_verdict() {
    let report = run_story(
        &walkthrough_only(),
        disagreeing().build(),
        Some("look\nlook\nlook\n"),
        &Ledger::default(),
    );
    assert_eq!((report.places, report.walkthrough), (4, 4));
    // Six directions with a word, at four places.
    assert_eq!(report.trials, 24);
    assert_eq!(report.wordless, 0);
    // North: fixed and honoured. Up and down: not exits, and nothing happens.
    assert_eq!(report.fixed_confirmed, 4);
    assert_eq!(report.none_confirmed, 8);
    // East: the table says room 1, the code goes to room 2.
    assert_eq!(report.fixed_wrong_unexplained, 4);
    assert_eq!(
        count(
            &report,
            EAST,
            Reported::Fixed(Toy::room(1)),
            Outcome::Moved(Toy::room(2))
        ),
        4
    );
    // West: not in the table, and the player moves.
    assert_eq!(report.none_wrong_unexplained, 4);
    assert_eq!(
        count(&report, WEST, Reported::None, Outcome::Moved(Toy::room(1))),
        4
    );
    // South: conditional, with what the game did. Never wrong.
    assert_eq!(report.conditional.stayed, 4);
    assert_eq!(report.conditional.total(), 4);
    assert_eq!(
        count(&report, SOUTH, Reported::Conditional, Outcome::Stayed),
        4
    );
    // Nothing is known to a ledger that is empty.
    assert_eq!((report.fixed_wrong_known, report.none_wrong_known), (0, 0));
    // Confirmed cases are not in the detail.
    assert_eq!(report.detail.len(), 3);
    // FR-007: the verdicts add up to the trials.
    let verdicts = report.fixed_confirmed
        + report.fixed_wrong_known
        + report.fixed_wrong_unexplained
        + report.none_confirmed
        + report.none_wrong_known
        + report.none_wrong_unexplained
        + report.none_other
        + report.conditional.total()
        + report.unknown.total()
        + report.no_verdict;
    assert_eq!(verdicts, report.trials);
}

// The heading and the reach of a report (data-model.md "Report"; research R10).
#[test]
fn a_report_says_what_was_run_and_how_far_it_got() {
    let bytes = disagreeing().build();
    let report = run_story(
        &walkthrough_only(),
        bytes,
        Some("look\n"),
        &Ledger::default(),
    );
    assert_eq!(report.id, "toy");
    assert_eq!(report.sha256, "0".repeat(64));
    assert_eq!(
        (report.version, report.family.as_str(), report.seed),
        (3, "zil", 1)
    );
    // The walkthrough stayed in one room; the table names one other.
    assert_eq!((report.rooms_visited, report.named_not_reached), (1, 1));
    // And it writes and reads back.
    assert_eq!(Report::read(&report.write()), Ok(report));
}

// A walkthrough that moves: the room each trial starts in is the room the
// player is in at that prompt.
#[test]
fn trials_start_from_where_the_walkthrough_has_brought_the_player() {
    let mut toy = Toy::new(2);
    toy.way(0, NORTH, 1).way(1, SOUTH, 0);
    let report = run_story(
        &entry("toy"),
        toy.build(),
        Some("north\n"),
        &Ledger::default(),
    );
    assert_eq!(report.places, 2);
    assert_eq!(report.rooms_visited, 2);
    assert_eq!(report.named_not_reached, 0);
    // North from room 0 and south from room 1, each once.
    assert_eq!(report.fixed_confirmed, 2);
    assert_eq!(report.none_confirmed, 10);
    assert!(report.detail.is_empty());
}

// K§ "`corpus`": `write` keeps a report; `run` compares a fresh one with it.
#[test]
fn run_is_ok_after_write_and_differs_when_a_kept_figure_is_changed() {
    let scratch = Scratch::new("run");
    let mut toy = Toy::new(2);
    toy.way(0, NORTH, 1).way(1, SOUTH, 0);
    scratch.write("stories/toy/walk.txt", b"north\nsouth\n");
    scratch.add("toy", &toy.build(), "walkthrough: stories/toy/walk.txt\n");

    // Nothing is kept yet: neither the report nor the summary.
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (
            vec![
                "toy differs".to_string(),
                "stories/corpus/reports/summary.txt differs".to_string()
            ],
            1
        )
    );
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]),
        (vec!["toy written".to_string()], 0)
    );
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    assert!(kept.starts_with("story toy\nsha256 "), "{kept}");
    assert!(
        kept.contains("\nplaces 3 walkthrough 3 crawl 0\n"),
        "{kept}"
    );
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (vec!["toy ok".to_string()], 0)
    );
    // Naming the story does the same.
    let named = ["toy".to_string()];
    assert_eq!(
        command(&scratch.0, Mode::Run, &named),
        (vec!["toy ok".to_string()], 0)
    );

    // One figure of the kept report changed.
    let changed = kept.replace("places 3 walkthrough 3", "places 4 walkthrough 4");
    assert_ne!(changed, kept);
    scratch.write("stories/corpus/reports/toy.txt", changed.as_bytes());
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (vec!["toy differs".to_string()], 1)
    );
}

// K§ "`corpus`": an id that is not in the manifest is an error, and nothing runs.
#[test]
fn an_id_the_manifest_does_not_have_is_an_error() {
    let scratch = Scratch::new("unknown-id");
    scratch.add("toy", &Toy::new(2).build(), "");
    let ids = ["toy".to_string(), "nosuch".to_string()];
    let (lines, status) = command(&scratch.0, Mode::Write, &ids);
    assert_eq!(status, 2);
    assert_eq!(lines, ["nosuch is not in the manifest"]);
    assert!(!scratch.path("stories/corpus/reports/toy.txt").exists());
}

// D§ "Story": a story is present when the file at its path has the manifest's
// digest. One that is absent, or is some other file, is skipped.
#[test]
fn a_story_whose_file_is_absent_or_is_another_file_is_skipped() {
    let scratch = Scratch::new("absent");
    scratch.add("toy", &Toy::new(2).build(), "");
    std::fs::remove_file(scratch.path("stories/toy/toy.z3")).unwrap();
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (vec!["toy skipped".to_string()], 0)
    );
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]),
        (vec!["toy skipped".to_string()], 0)
    );
    scratch.write("stories/toy/toy.z3", &Toy::new(3).build());
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (vec!["toy skipped".to_string()], 0)
    );
}

// SC-001: the report kept for Zork I has a verdict for every direction with a
// word at every command prompt of its walkthrough, and the verdicts add up.
#[test]
fn zork_is_report_covers_every_direction_at_every_prompt() {
    use zmachine_corpus::directions::directions;
    use zmachine_introspect::Introspector;
    let kept = Report::read(&read_text("stories/corpus/reports/zork1.txt")).expect("it reads");
    let machine = load("stories/zork1/zork1.z3", 11);
    let view = machine.view();
    let with_a_word = directions(&Introspector::new(&view), &view)
        .iter()
        .filter(|d| d.word.is_some())
        .count();
    // 330 command prompts (tests/prompts.rs).
    assert_eq!(kept.walkthrough, 330);
    assert!(kept.trials >= kept.walkthrough * with_a_word);
    assert_eq!(kept.verdicts(), kept.trials);
    assert_eq!(kept.wordless, 0);
}
