//! The proof of look-ahead: each finding set against a trial apart from it
//! (specs/009-compass-lookahead/spec.md User Story 5; contracts/corpus.md,
//! cited C§).

mod common;

use common::*;
use zmachine_corpus::corpus::{Mode, command};
use zmachine_corpus::ledger::{self, Ledger};
use zmachine_corpus::manifest::{Story, Tier};
use zmachine_corpus::report::{Looked, Report, missed_text};
use zmachine_corpus::walk::run_story;

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
        first: vec![],
        inventory: None,
        asides: vec![],
        look: None,
        // The walkthrough's places alone.
        rooms: Some(0),
        trials: None,
        line: 1,
    }
}

/// In the first room: north is conditional and allowed, south conditional and
/// refused, west conditional and ends the game, up is a plain way.
fn sure() -> Toy {
    let mut toy = Toy::new(3);
    let there = Toy::room(1) as u8;
    toy.uncertain(0, NORTH, there)
        .uncertain(0, SOUTH, STAY)
        .uncertain(0, WEST, QUIT)
        .way(0, UP, 2);
    toy
}

/// As `sure`, with east decided by an even draw.
fn chancy() -> Toy {
    let mut toy = sure();
    toy.uncertain(0, EAST, COIN);
    toy
}

/// Lines that never leave the first room: `count` + 1 command prompts.
fn staying(count: usize) -> String {
    "look\n".repeat(count)
}

// C§1 to 5; FR-018, FR-019: at each place each way the interface could not
// settle is asked of look-ahead, and counted by its finding; each finding is
// borne out by the trial apart.
#[test]
fn every_unsettled_way_is_counted_by_its_finding() {
    let report = run_story(
        &entry(),
        sure().build(),
        Some(&staying(3)),
        &Ledger::default(),
    );
    assert_eq!(report.places, 4);
    assert_eq!(
        report.lookahead,
        Some(Looked {
            walked: 4,
            refused: 4,
            // The way that ends the game has no finding.
            left: 4,
            missed_known: 0,
            missed_unexplained: 0,
        })
    );
    assert!(report.lookahead_detail.is_empty());
    // The plain way and the directions with no way are not asked.
    assert_eq!(report.lookahead.unwrap().asked(), 12);
}

// C§4, 6; FR-020; SC-008: every other figure is what it is with no look-ahead:
// the report without its look-ahead lines is the report of the same run before
// this feature, which the conditional line already counted.
#[test]
fn every_other_line_of_the_report_is_untouched() {
    let report = run_story(
        &entry(),
        sure().build(),
        Some(&staying(3)),
        &Ledger::default(),
    );
    let text = report.write();
    assert_eq!(report.conditional.total(), 12);
    assert_eq!(
        (
            report.conditional.moved,
            report.conditional.stayed,
            report.conditional.ended
        ),
        (4, 4, 4)
    );
    let mut bare = report.clone();
    bare.lookahead = None;
    let without: Vec<&str> = text
        .lines()
        .filter(|l| !l.starts_with("lookahead "))
        .collect();
    assert_eq!(without.join("\n") + "\n", bare.write());
    // A story with no such way has no line.
    let mut plain = Toy::new(2);
    plain.way(0, NORTH, 1);
    let report = run_story(
        &entry(),
        plain.build(),
        Some(&staying(2)),
        &Ledger::default(),
    );
    assert_eq!(report.lookahead, None);
    assert!(!report.write().contains("lookahead"));
}

// C§3, 8, 9; FR-018; SC-003: a way decided by a draw is mostly left; where three
// copies agree by chance and the trial apart does not, the finding is not
// borne out, and is unexplained until the ledger names it.
#[test]
fn a_finding_not_borne_out_is_unexplained_until_the_ledger_names_it() {
    let image = chancy().build();
    let script = staying(60);
    let report = run_story(&entry(), image.clone(), Some(&script), &Ledger::default());
    let looked = report.lookahead.unwrap();
    // 61 places, 4 ways each; the three sure ways are as before.
    assert_eq!(looked.asked(), 244);
    let (walked, refused, left) = (looked.walked - 61, looked.refused - 61, looked.left - 61);
    assert_eq!(walked + refused + left, 61);
    assert!(left > 30, "an even draw is mostly left: {left}");
    assert!(looked.missed_unexplained > 0, "{looked:?}");
    assert_eq!(looked.missed_known, 0);
    assert!(looked.missed_unexplained <= walked + refused);
    let missed: usize = report.lookahead_detail.iter().map(|(_, n)| n).sum();
    assert_eq!(missed, looked.missed_unexplained);
    assert!(
        report
            .lookahead_detail
            .iter()
            .all(|(case, _)| case.room == Toy::room(0) && case.direction == EAST)
    );

    // The same run with each case in the ledger: all known.
    let entries: String = report
        .lookahead_detail
        .iter()
        .map(|(case, _)| {
            let text = missed_text(case);
            let words: Vec<&str> = text.split(' ').collect();
            let (found, outcome) = match words[3] {
                "walked" => (words[3..5].join(" "), words[5..].join(" ")),
                _ => (words[3].to_string(), words[4..].join(" ")),
            };
            format!(
                "story: toy\nroom: {}\ndirection: {}\nlookahead: {found}\noutcome: {outcome}\n\
                 cause: The toy draws.\n\n",
                words[1], words[2]
            )
        })
        .collect();
    let known = ledger::read(&entries).unwrap();
    let again = run_story(&entry(), image, Some(&script), &known);
    let looked = again.lookahead.unwrap();
    assert_eq!(looked.missed_unexplained, 0);
    assert_eq!(looked.missed_known, missed);
    assert_eq!(Report::read(&again.write()).unwrap().write(), again.write());
}

// C§9: the program fails on a finding not borne out with no cause, naming the
// story, room and direction; passes once the ledger holds it; and reports an
// entry that no finding needs.
#[test]
fn the_run_fails_until_the_ledger_explains_and_reports_an_entry_no_finding_needs() {
    let scratch = Scratch::new("lookahead");
    scratch.write("stories/toy/walk.txt", staying(60).as_bytes());
    scratch.add(
        "toy",
        &chancy().build(),
        "walkthrough: stories/toy/walk.txt\nrooms: 0\n",
    );
    let (lines, status) = command(&scratch.0, Mode::Run, &[]);
    assert_eq!(status, 1);
    let wrong: Vec<&String> = lines
        .iter()
        .filter(|l| l.starts_with("toy lookahead wrong "))
        .collect();
    assert!(!wrong.is_empty(), "{lines:?}");
    let first_room = Toy::room(0);
    assert!(
        wrong
            .iter()
            .all(|l| l.starts_with(&format!("toy lookahead wrong {first_room} {EAST} "))),
        "{lines:?}"
    );

    // Each line names a case: `walked N OUTCOME` or `refused OUTCOME`.
    let mut ledger_text = String::new();
    for line in &wrong {
        let words: Vec<&str> = line.split(' ').collect();
        let (found, outcome) = match words[5] {
            "walked" => (words[5..7].join(" "), words[7..].join(" ")),
            _ => (words[5].to_string(), words[6..].join(" ")),
        };
        ledger_text.push_str(&format!(
            "story: toy\nroom: {}\ndirection: {}\nlookahead: {found}\noutcome: {outcome}\n\
             cause: The toy draws.\n\n",
            words[3], words[4]
        ));
    }
    scratch.write("stories/corpus/ledger.txt", ledger_text.as_bytes());
    assert_eq!(command(&scratch.0, Mode::Write, &[]).1, 0);
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (vec!["toy ok".to_string()], 0)
    );
    // What is kept holds numbers and the format's own words.
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    assert!(kept.contains("\nlookahead walked "), "{kept}");
    assert!(!kept.contains("Room"), "{kept}");

    // An entry for a case that does not happen.
    let extra = format!(
        "{ledger_text}story: toy\nroom: {first_room}\ndirection: {NORTH}\nlookahead: refused\n\
         outcome: moved {}\ncause: Never.\n",
        Toy::room(1)
    );
    scratch.write("stories/corpus/ledger.txt", extra.as_bytes());
    let (lines, status) = command(&scratch.0, Mode::Run, &[]);
    assert_eq!(status, 1);
    assert!(
        lines
            .iter()
            .any(|l| l.ends_with("the entry matches no finding")),
        "{lines:?}"
    );
}
