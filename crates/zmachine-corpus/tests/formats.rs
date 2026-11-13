//! The corpus's files: what each format accepts and what it refuses, with the
//! line (specs/005-corpus-testing/contracts/formats.md).

use zmachine_corpus::ledger;
use zmachine_corpus::manifest::{self, Tier};
use zmachine_corpus::report::{
    Case, Figures, Marked, Outcomes, Reached, Report, StoryLine, Summary,
};
use zmachine_corpus::trial::Outcome;
use zmachine_corpus::verdict::Reported;

const DIGEST: &str = "37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79";

fn case(room: u16, direction: u8, reported: Reported, outcome: Outcome) -> Case {
    Case {
        room,
        direction,
        reported,
        outcome,
    }
}

/// A report whose figures add up.
fn sample() -> Report {
    Report {
        id: "toy".to_string(),
        sha256: DIGEST.to_string(),
        version: 3,
        family: "zil".to_string(),
        seed: 11,
        places: 5,
        walkthrough: 4,
        crawl: 1,
        rooms_visited: 2,
        named_not_reached: 1,
        budget_rooms: 400,
        budget_trials: 40000,
        reached: Reached::No,
        trials: 30,
        wordless: 2,
        fixed_confirmed: 8,
        fixed_wrong_known: 1,
        fixed_wrong_unexplained: 2,
        none_confirmed: 9,
        none_wrong_known: 0,
        none_wrong_unexplained: 1,
        none_other: 1,
        conditional: Outcomes {
            moved: 2,
            stayed: 1,
            question: 0,
            ended: 1,
            fault: 0,
        },
        unknown: Outcomes {
            moved: 0,
            stayed: 2,
            question: 1,
            ended: 0,
            fault: 0,
        },
        no_verdict: 1,
        pack: zmachine_introspect::PackState::None,
        entries: 0,
        marked: Marked::default(),
        inventory: None,
        inventory_detail: vec![],
        light: None,
        light_detail: vec![],
        lookahead: None,
        lookahead_detail: vec![],
        detail: vec![
            (case(2, 28, Reported::None, Outcome::Moved(3)), 1),
            (case(2, 29, Reported::Fixed(3), Outcome::Moved(4)), 2),
            (case(2, 30, Reported::Conditional, Outcome::Stayed), 1),
            (case(3, 22, Reported::Unknown, Outcome::Question), 1),
            (case(3, 31, Reported::Fixed(2), Outcome::Lost), 1),
        ],
    }
}

const SAMPLE: &str = "\
story toy
sha256 37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79
version 3 family zil seed 11
places 5 walkthrough 4 crawl 1
rooms visited 2 named-not-reached 1
budget rooms 400 trials 40000 reached no
trials 30 wordless 2
fixed confirmed 8 wrong-known 1 wrong-unexplained 2
none confirmed 9 wrong-known 0 wrong-unexplained 1 other 1
conditional 4 moved 2 stayed 1 question 0 ended 1 fault 0
unknown 3 moved 0 stayed 2 question 1 ended 0 fault 0
no-verdict 1
detail
2 28 none moved 3 1
2 29 fixed 3 moved 4 2
2 30 conditional stayed 1
3 22 unknown question 1
3 31 fixed 2 lost 1
";

// F§ "A report": the lines, in order, each of its form.
#[test]
fn a_report_is_written_line_for_line_as_the_format_gives_it() {
    assert_eq!(sample().write(), SAMPLE);
}

#[test]
fn a_report_reads_back_as_it_was_written() {
    assert_eq!(Report::read(SAMPLE), Ok(sample()));
    let mut reached = sample();
    reached.reached = Reached::Trials;
    assert_eq!(Report::read(&reached.write()), Ok(reached.clone()));
    reached.reached = Reached::Rooms;
    assert_eq!(Report::read(&reached.write()), Ok(reached));
}

// F§ "A report": detail lines are sorted by room, then direction, then the rest
// as text, whatever order the cases were counted in.
#[test]
fn detail_lines_are_sorted() {
    let mut report = sample();
    report.detail.reverse();
    assert_eq!(report.write(), SAMPLE);
}

/// The sample with one line replaced, and the line it is on.
fn with(line: usize, replacement: &str) -> String {
    let mut lines: Vec<&str> = SAMPLE.lines().collect();
    lines[line - 1] = replacement;
    lines.join("\n") + "\n"
}

fn refused_at(text: &str) -> usize {
    Report::read(text).expect_err("the report is refused").0
}

// F§ "A report": nothing else may appear. Each refusal names its line.
#[test]
fn a_report_that_departs_from_the_format_is_refused_with_the_line() {
    // A line out of order.
    let mut lines: Vec<&str> = SAMPLE.lines().collect();
    lines.swap(3, 4);
    assert_eq!(refused_at(&(lines.join("\n") + "\n")), 4);
    // An unknown keyword.
    assert_eq!(refused_at(&with(7, "trials 30 wordy 2")), 7);
    // A word where a number belongs.
    assert_eq!(refused_at(&with(4, "places five walkthrough 4 crawl 1")), 4);
    // Words of a game where the format has none.
    assert_eq!(refused_at(&with(14, "2 28 none moved Kitchen 1")), 14);
    assert_eq!(
        refused_at(&with(
            5,
            "rooms visited 2 named-not-reached 1 West of House"
        )),
        5
    );
    // A digest that is not one.
    assert_eq!(refused_at(&with(2, "sha256 abc")), 2);
    // A family that is not one.
    assert_eq!(refused_at(&with(3, "version 3 family frotz seed 11")), 3);
    // A line too many, and a line too few.
    assert_eq!(refused_at(&(SAMPLE.to_string() + "anything\n")), 19);
    let short: String = SAMPLE.lines().take(12).map(|l| format!("{l}\n")).collect();
    assert_eq!(refused_at(&short), 13);
}

// F§ "A report": the outcome counts add up to their total, and the verdict
// lines add up to the trials (FR-007).
#[test]
fn a_report_whose_counts_do_not_add_up_is_refused() {
    let line = "conditional 5 moved 2 stayed 1 question 0 ended 1 fault 0";
    assert_eq!(refused_at(&with(10, line)), 10);
    let line = "unknown 3 moved 1 stayed 2 question 1 ended 0 fault 0";
    assert_eq!(refused_at(&with(11, line)), 11);
    // One trial more than the verdicts account for.
    assert_eq!(refused_at(&with(7, "trials 31 wordless 2")), 7);
}

// ---------------------------------------------------------------------------
// The manifest (F§ "The manifest", the form only)
// ---------------------------------------------------------------------------

const ENTRY: &str = "\
# The corpus.
id: zork1
sha256: 37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79
size: 86838
version: 3
family: zil
source: https://github.com/historicalsource/zork1 COMPILED/zork1.z3
standing: MIT, Copyright (c) 2025 Microsoft; see stories/zork1/SOURCE.md
tier: committed
path: stories/zork1/zork1.z3
seed: 11
walkthrough: stories/zork1/fidelity/walkthrough.txt

id: curses
sha256: 00000000000000000000000000000000000000000000000000000000000000aa
size: 259584
version: 5
family: inform
source: https://ifarchive.org/if-archive/games/zcode/curses.z5
standing: no license attached; personal use only under the archive's terms
tier: fetched
path: stories/local/corpus/curses.z5
seed: 1
opening: no
opening: yes
rooms: 50
trials: 900
";

#[test]
fn a_manifest_reads_as_the_format_gives_it() {
    let stories = manifest::read(ENTRY).expect("the manifest reads");
    assert_eq!(stories.len(), 2);
    let zork = &stories[0];
    assert_eq!(zork.id, "zork1");
    assert_eq!(zork.sha256, DIGEST);
    assert_eq!((zork.size, zork.version, zork.seed), (86838, 3, 11));
    assert_eq!(zork.family, "zil");
    assert_eq!(zork.tier, Tier::Committed);
    assert_eq!(zork.path, "stories/zork1/zork1.z3");
    assert_eq!(
        zork.walkthrough.as_deref(),
        Some("stories/zork1/fidelity/walkthrough.txt")
    );
    assert_eq!(
        zork.source,
        "https://github.com/historicalsource/zork1 COMPILED/zork1.z3"
    );
    assert!(zork.opening.is_empty());
    assert_eq!((zork.rooms, zork.trials), (None, None));
    assert_eq!(zork.line, 2);
    let curses = &stories[1];
    assert_eq!(curses.tier, Tier::Fetched);
    assert_eq!(curses.walkthrough, None);
    // `opening` alone may repeat, in order.
    assert_eq!(curses.opening, ["no", "yes"]);
    assert_eq!((curses.rooms, curses.trials), (Some(50), Some(900)));
    assert_eq!(curses.line, 14);
    // No entries is a manifest too.
    assert_eq!(manifest::read("# nothing yet\n"), Ok(vec![]));
}

/// The manifest with one line replaced (`None` removes it), refused at a line.
fn manifest_refused(line: usize, replacement: Option<&str>) -> usize {
    let mut lines: Vec<&str> = ENTRY.lines().collect();
    match replacement {
        Some(text) => lines[line - 1] = text,
        None => {
            lines.remove(line - 1);
        }
    }
    manifest::read(&(lines.join("\n") + "\n"))
        .expect_err("the manifest is refused")
        .0
}

// F§ "The manifest": each refusal, with the line.
#[test]
fn a_manifest_that_departs_from_the_format_is_refused_with_the_line() {
    // A key that is not known.
    assert_eq!(manifest_refused(5, Some("colour: blue")), 5);
    // A line that is not `key: value`.
    assert_eq!(manifest_refused(5, Some("version 3")), 5);
    // A key repeated, other than `opening`.
    assert_eq!(manifest_refused(6, Some("version: 3")), 6);
    // A required key missing: the entry is refused where it starts.
    assert_eq!(manifest_refused(4, None), 2);
    assert_eq!(manifest_refused(11, None), 2);
    // Values not of their form.
    assert_eq!(manifest_refused(2, Some("id: Zork One")), 2);
    assert_eq!(manifest_refused(3, Some("sha256: 3708")), 3);
    assert_eq!(manifest_refused(4, Some("size: big")), 4);
    assert_eq!(manifest_refused(5, Some("version: 4")), 5);
    assert_eq!(manifest_refused(6, Some("family: frotz")), 6);
    assert_eq!(manifest_refused(9, Some("tier: borrowed")), 9);
    assert_eq!(manifest_refused(11, Some("seed: -1")), 11);
    assert_eq!(manifest_refused(11, Some("seed: 2147483648")), 11);
    // A walkthrough only with a committed story.
    assert_eq!(
        manifest_refused(25, Some("walkthrough: stories/local/corpus/curses.txt")),
        25
    );
    // A fetched story lives under stories/local/corpus/, a committed one does not.
    assert_eq!(
        manifest_refused(22, Some("path: stories/curses/curses.z5")),
        22
    );
    assert_eq!(
        manifest_refused(10, Some("path: stories/local/corpus/zork1.z3")),
        10
    );
}

// ---------------------------------------------------------------------------
// The ledger (F§ "The ledger"; FR-013, FR-014)
// ---------------------------------------------------------------------------

const LEDGER: &str = "\
# Known wrong reports.
story: zork1
room: 82
direction: 28
reported: fixed 83
outcome: moved 84
cause: The game's own code moves the player on at arrival; introspection specification
  section 8.3.

story: zork1
room: 90
direction: 31
reported: none
outcome: moved 91
cause: A routine runs before the table is read.
";

#[test]
fn a_ledger_reads_as_the_format_gives_it() {
    let ledger = ledger::read(LEDGER).expect("the ledger reads");
    assert_eq!(ledger.entries.len(), 2);
    let first = &ledger.entries[0];
    assert_eq!(first.story, "zork1");
    assert_eq!(
        first.case,
        case(82, 28, Reported::Fixed(83), Outcome::Moved(84))
    );
    // A cause may continue on lines that start with two spaces.
    assert_eq!(
        first.cause,
        "The game's own code moves the player on at arrival; introspection specification \
         section 8.3."
    );
    assert_eq!(first.line, 2);
    assert_eq!(ledger.entries[1].line, 10);
    assert!(ledger.knows("zork1", &case(90, 31, Reported::None, Outcome::Moved(91))));
    assert!(!ledger.knows("zork2", &case(90, 31, Reported::None, Outcome::Moved(91))));
    assert!(!ledger.knows("zork1", &case(90, 31, Reported::None, Outcome::Moved(92))));
    assert_eq!(
        ledger::read("# nothing known\n").map(|l| l.entries.len()),
        Ok(0)
    );
}

fn ledger_refused(line: usize, replacement: Option<&str>) -> usize {
    let mut lines: Vec<&str> = LEDGER.lines().collect();
    match replacement {
        Some(text) => lines[line - 1] = text,
        None => {
            lines.remove(line - 1);
        }
    }
    ledger::read(&(lines.join("\n") + "\n"))
        .expect_err("the ledger is refused")
        .0
}

#[test]
fn a_ledger_that_departs_from_the_format_is_refused_with_the_line() {
    // A key missing: the entry is refused where it starts.
    assert_eq!(ledger_refused(3, None), 2);
    assert_eq!(ledger_refused(14, None), 10);
    // A cause that says nothing (FR-014).
    assert_eq!(ledger_refused(15, Some("cause: ")), 15);
    assert_eq!(ledger_refused(15, Some("cause:")), 15);
    // A key that is not known, and one given twice.
    assert_eq!(ledger_refused(4, Some("colour: blue")), 4);
    assert_eq!(ledger_refused(4, Some("room: 82")), 4);
    // Values not of their form.
    assert_eq!(ledger_refused(3, Some("room: the kitchen")), 3);
    assert_eq!(ledger_refused(5, Some("reported: conditional")), 5);
    assert_eq!(ledger_refused(6, Some("outcome: moved")), 6);
    assert_eq!(ledger_refused(6, Some("outcome: lost")), 6);
    // A report and an outcome that are not a wrong verdict.
    assert_eq!(ledger_refused(6, Some("outcome: moved 83")), 2);
    assert_eq!(ledger_refused(14, Some("outcome: stayed")), 10);
    // The same case twice.
    let twice = format!(
        "{LEDGER}\n{}",
        LEDGER.lines().skip(9).collect::<Vec<_>>().join("\n")
    );
    assert_eq!(ledger::read(&(twice + "\n")).expect_err("refused").0, 17);
}

// F§ "The manifest": an opening answer written `(space)` is one space, and
// `(return)` is an empty line; neither can be written safely as it is.
#[test]
fn an_opening_answer_may_be_a_space_or_an_empty_line() {
    let text = ENTRY.replace(
        "opening: no\nopening: yes\n",
        "opening: (space)\nopening: (return)\nopening: n\n",
    );
    assert_ne!(text, ENTRY);
    let stories = manifest::read(&text).expect("the manifest reads");
    assert_eq!(stories[1].opening, [" ", "", "n"]);
}

// ---------------------------------------------------------------------------
// The summary (F§ "The summary"; FR-025)
// ---------------------------------------------------------------------------

// F§ "The summary": a story's line holds its report's figures.
#[test]
fn a_storys_line_of_the_summary_is_its_reports_figures() {
    let line = sample().line();
    assert_eq!((line.id.as_str(), line.family.as_str()), ("toy", "zil"));
    assert_eq!(
        line.figures,
        Figures {
            places: 5,
            rooms: 2,
            trials: 30,
            // 8 confirmed of 8 + 1 + 2 reported as leading to a room.
            fixed: (8, 11),
            // 9 confirmed of 9 + 0 + 1 + 1 reported as not an exit.
            none: (9, 11),
            // 2 moved the player of 4 reported conditional.
            conditional: (2, 4),
            unknown: 3,
            wrong_known: 1,
            wrong_unexplained: 3,
            looked: [0; 4],
        }
    );
}

fn story_line(id: &str, family: &str, scale: usize) -> StoryLine {
    StoryLine {
        id: id.to_string(),
        family: family.to_string(),
        figures: Figures {
            places: scale,
            rooms: 2 * scale,
            trials: 30 * scale,
            fixed: (8 * scale, 11 * scale),
            none: (9 * scale, 11 * scale),
            conditional: (2 * scale, 4 * scale),
            unknown: 3 * scale,
            wrong_known: scale,
            wrong_unexplained: 0,
            looked: [0; 4],
        },
    }
}

const SUMMARY: &str = "\
story one zil places 1 rooms 2 trials 30 fixed 8/11 none 9/11 conditional 2/4 unknown 3 wrong-known 1 wrong-unexplained 0
story two inform places 2 rooms 4 trials 60 fixed 16/22 none 18/22 conditional 4/8 unknown 6 wrong-known 2 wrong-unexplained 0
story three zil places 3 rooms 6 trials 90 fixed 24/33 none 27/33 conditional 6/12 unknown 9 wrong-known 3 wrong-unexplained 0
family zil stories 2 places 4 rooms 8 trials 120 fixed 32/44 none 36/44 conditional 8/16 unknown 12 wrong-known 4 wrong-unexplained 0
family inform stories 1 places 2 rooms 4 trials 60 fixed 16/22 none 18/22 conditional 4/8 unknown 6 wrong-known 2 wrong-unexplained 0
family none stories 0 places 0 rooms 0 trials 0 fixed 0/0 none 0/0 conditional 0/0 unknown 0 wrong-known 0 wrong-unexplained 0
all stories 3 places 6 rooms 12 trials 180 fixed 48/66 none 54/66 conditional 12/24 unknown 18 wrong-known 6 wrong-unexplained 0
skipped four
";

fn summary() -> Summary {
    Summary {
        stories: vec![
            story_line("one", "zil", 1),
            story_line("two", "inform", 2),
            story_line("three", "zil", 3),
        ],
        skipped: vec!["four".to_string()],
    }
}

// F§ "The summary": stories in the manifest's order, families in the order zil,
// inform, none, then all; a skipped story has a line of its own and is in no
// total.
#[test]
fn the_summary_is_written_as_the_format_gives_it_and_reads_back() {
    assert_eq!(summary().write(), SUMMARY);
    assert_eq!(Summary::read(SUMMARY), Ok(summary()));
}

#[test]
fn a_summary_that_departs_from_the_format_is_refused_with_the_line() {
    let refused = |line: usize, replacement: &str| {
        let mut lines: Vec<&str> = SUMMARY.lines().collect();
        lines[line - 1] = replacement;
        Summary::read(&(lines.join("\n") + "\n"))
            .expect_err("the summary is refused")
            .0
    };
    assert_eq!(refused(1, "story one zil places one rooms 2"), 1);
    assert_eq!(refused(2, "story West of House"), 2);
    // A total that is not the sum of its stories.
    let wrong = SUMMARY
        .lines()
        .nth(3)
        .unwrap()
        .replace("places 4", "places 5");
    assert_eq!(refused(4, &wrong), 4);
    let wrong = SUMMARY
        .lines()
        .nth(6)
        .unwrap()
        .replace("trials 180", "trials 181");
    assert_eq!(refused(7, &wrong), 7);
    assert_eq!(refused(8, "skipped Four Rooms"), 8);
}

// ---------------------------------------------------------------------------
// Game packs (specs/006-game-packs/contracts/commands.md, "The corpus's files")
// ---------------------------------------------------------------------------

/// The sample as a run with a pack in force gives it: twelve trials more, on
/// answers that rest on the pack.
fn marked() -> Report {
    let mut report = sample();
    report.pack = zmachine_introspect::PackState::InForce;
    report.entries = 3;
    report.marked = Marked {
        open_confirmed: 4,
        open_wrong_known: 1,
        open_wrong_unexplained: 0,
        closed_confirmed: 2,
        closed_wrong_known: 0,
        closed_wrong_unexplained: 1,
        closed_other: 1,
        distrusted: Outcomes {
            moved: 1,
            stayed: 2,
            question: 0,
            ended: 0,
            fault: 0,
        },
    };
    report.trials += 12;
    report.detail.extend([
        (case(4, 28, Reported::Open(5), Outcome::Stayed), 1),
        (case(4, 29, Reported::Closed(5), Outcome::Ended), 1),
        (case(4, 29, Reported::Closed(5), Outcome::Moved(5)), 1),
        (case(4, 30, Reported::Distrusted, Outcome::Moved(6)), 1),
        (case(4, 30, Reported::Distrusted, Outcome::Stayed), 2),
    ]);
    report
}

const PACK_LINES: &str = "\
no-verdict 1
pack in-force entries 3
open confirmed 4 wrong-known 1 wrong-unexplained 0
closed confirmed 2 wrong-known 0 wrong-unexplained 1 other 1
distrusted 3 moved 1 stayed 2 question 0 ended 0 fault 0
detail
";

// FR-022: a report of a run with a pack says so, and counts the trials on
// answers that rest on it; without a pack a report is what it was.
#[test]
fn a_report_with_a_pack_writes_and_reads_its_marked_trials() {
    let written = marked().write();
    assert!(written.contains(PACK_LINES), "{written}");
    for line in [
        "4 28 open 5 stayed 1",
        "4 29 closed 5 ended 1",
        "4 29 closed 5 moved 5 1",
        "4 30 distrusted moved 6 1",
        "4 30 distrusted stayed 2",
    ] {
        assert!(
            written.contains(&format!("\n{line}\n")),
            "{line}: {written}"
        );
    }
    assert_eq!(Report::read(&written), Ok(marked()));
    assert_eq!(marked().verdicts(), marked().trials);
    assert!(!sample().write().contains("pack"), "no pack, no line");

    // A pack that was given and is not in force: one line, and no marked trial.
    for (state, word) in [
        (zmachine_introspect::PackState::NotMatching, "not-matching"),
        (zmachine_introspect::PackState::Refused, "refused"),
    ] {
        let mut report = sample();
        report.pack = state;
        let written = report.write();
        assert!(
            written.contains(&format!("no-verdict 1\npack {word}\ndetail\n")),
            "{written}"
        );
        assert_eq!(Report::read(&written), Ok(report));
    }
}

#[test]
fn a_report_whose_pack_lines_depart_from_the_format_is_refused() {
    let written = marked().write();
    let with = |from: &str, to: &str| {
        assert!(written.contains(from), "{from}");
        Report::read(&written.replace(from, to))
            .expect_err("refused")
            .0
    };
    assert_eq!(with("pack in-force entries 3", "pack in-force"), 13);
    assert_eq!(
        with("pack in-force entries 3", "pack sometimes entries 3"),
        13
    );
    assert_eq!(with("pack in-force entries 3", "pack none"), 13);
    assert_eq!(with("open confirmed 4", "open confirmed four"), 14);
    assert_eq!(
        with("closed confirmed 2 wrong-known 0", "closed confirmed 2"),
        15
    );
    // The outcomes of the distrusted do not add up.
    assert_eq!(with("distrusted 3 moved 1", "distrusted 4 moved 1"), 16);
    // One marked trial more than the trials account for.
    assert_eq!(with("open confirmed 4", "open confirmed 5"), 7);
    // A marked case in the detail of a report with no pack.
    let plain = sample().write() + "4 28 open 5 stayed 1\n";
    assert_eq!(Report::read(&plain).expect_err("refused").0, 19);
}

// The ledger may explain a wrong report on a door (FR-019); a report that is
// conditional is never wrong, with a pack or without.
#[test]
fn a_ledger_entry_may_be_on_a_door() {
    let entry = |reported: &str, outcome: &str| {
        format!(
            "story: toy\nroom: 4\ndirection: 28\nreported: {reported}\noutcome: {outcome}\ncause: The room is dark.\n"
        )
    };
    let read = ledger::read(&entry("open 5", "ended")).unwrap();
    assert_eq!(
        read.entries[0].case,
        case(4, 28, Reported::Open(5), Outcome::Ended)
    );
    let read = ledger::read(&entry("closed 5", "moved 5")).unwrap();
    assert_eq!(read.entries[0].case.reported, Reported::Closed(5));
    assert_eq!(
        ledger::read(&entry("distrusted", "stayed")).unwrap_err().0,
        4
    );
    // A closed door that kept the player where they were is not wrong.
    assert_eq!(ledger::read(&entry("closed 5", "stayed")).unwrap_err().0, 1);
    assert_eq!(ledger::read(&entry("open 5", "moved 5")).unwrap_err().0, 1);
}

// The summary's form is unchanged: a door reported open counts with the exits
// reported as leading to a room, one reported closed with those reported as not
// an exit, and a distrusted exit with the conditional.
#[test]
fn the_summary_counts_marked_trials_with_their_like() {
    let (plain, with) = (sample().line().figures, marked().line().figures);
    assert_eq!(with.trials, plain.trials + 12);
    assert_eq!(with.fixed, (plain.fixed.0 + 4, plain.fixed.1 + 5));
    assert_eq!(with.none, (plain.none.0 + 2, plain.none.1 + 4));
    assert_eq!(
        with.conditional,
        (plain.conditional.0 + 1, plain.conditional.1 + 3)
    );
    assert_eq!(with.wrong_known, plain.wrong_known + 1);
    assert_eq!(with.wrong_unexplained, plain.wrong_unexplained + 1);
}

// Feature 006 (specs/006-game-packs/data-model.md, "Story"; research R12): the
// key `first` may repeat, in order: commands given at the first command
// prompts. It is not allowed with a walkthrough.
#[test]
fn a_manifest_may_give_first_commands() {
    let with = ENTRY.replace(
        "opening: yes\n",
        "opening: yes\nfirst: north\nfirst: open\n",
    );
    let stories = manifest::read(&with).expect("the manifest reads");
    assert_eq!(stories[1].first, ["north", "open"]);
    assert!(stories[0].first.is_empty());
    let both = ENTRY.replace(
        "walkthrough: stories/zork1/fidelity/walkthrough.txt\n",
        "walkthrough: stories/zork1/fidelity/walkthrough.txt\nfirst: north\n",
    );
    assert_eq!(manifest::read(&both).unwrap_err().0, 13);
}

// ---------------------------------------------------------------------------
// The inventory comparison (specs/007-inventory-panel/contracts/commands.md,
// "The corpus's files"; data-model.md "Story", "Ledger entry")
// ---------------------------------------------------------------------------

// FR-012: the command that asks a game for its inventory is data about the
// story. `aside` may repeat, and needs it.
#[test]
fn a_manifest_may_give_an_inventory_command_and_asides() {
    let with = ENTRY.replace(
        "opening: yes\n",
        "opening: yes\ninventory: i\naside: north / take lamp\naside: east\n",
    );
    let stories = manifest::read(&with).expect("the manifest reads");
    assert_eq!(stories[1].inventory.as_deref(), Some("i"));
    assert_eq!(
        stories[1].asides,
        [
            vec!["north".to_string(), "take lamp".to_string()],
            vec!["east".to_string()]
        ]
    );
    assert_eq!(
        (stories[0].inventory.as_deref(), stories[0].asides.len()),
        (None, 0)
    );
    let twice = ENTRY.replace(
        "opening: yes\n",
        "opening: yes\ninventory: i\ninventory: inv\n",
    );
    assert_eq!(manifest::read(&twice).unwrap_err().0, 27);
    let alone = ENTRY.replace("opening: yes\n", "opening: yes\naside: north\n");
    assert_eq!(manifest::read(&alone).unwrap_err().0, 26);
}

// R9: a ledger entry of the second form, for a difference between what is seen
// and what the game lists.
#[test]
fn a_ledger_entry_may_be_on_the_inventory() {
    use zmachine_corpus::report::Difference;
    let entry = |seen: &str, listed: &str| {
        format!(
            "story: toy\nroom: 4\nobject: 9\nseen: {seen}\nlisted: {listed}\ncause: The game lists it by another name.\n"
        )
    };
    let read = ledger::read(&entry("seen", "unlisted")).unwrap();
    assert!(read.entries.is_empty());
    assert_eq!(
        read.inventory[0].difference,
        Difference {
            room: 4,
            object: 9,
            seen: true,
            listed: false
        }
    );
    assert_eq!(read.inventory[0].line, 1);
    assert!(ledger::read(&entry("kept", "listed")).is_ok());
    // The two must differ; the words are fixed; the forms do not mix.
    assert_eq!(ledger::read(&entry("seen", "listed")).unwrap_err().0, 1);
    assert_eq!(ledger::read(&entry("kept", "unlisted")).unwrap_err().0, 1);
    assert_eq!(ledger::read(&entry("maybe", "listed")).unwrap_err().0, 4);
    let mixed = entry("kept", "listed").replace("object: 9\n", "object: 9\ndirection: 31\n");
    assert_eq!(ledger::read(&mixed).unwrap_err().0, 1);
    let causeless =
        entry("kept", "listed").replace("cause: The game lists it by another name.", "cause:");
    assert_eq!(ledger::read(&causeless).unwrap_err().0, 6);
}

// R9: a report of a story with an inventory command has one more line, and a
// detail line for each difference. Without the command a report is what it was.
#[test]
fn a_report_may_hold_the_inventory_comparison() {
    use zmachine_corpus::report::{Difference, Inventory};
    let mut report = sample();
    report.inventory = Some(Inventory {
        compared: 20,
        agreed: 15,
        differed_known: 2,
        differed_unexplained: 1,
        unjudged: 2,
        no_listing: 1,
        empty: 3,
        unknown: 4,
        through_closed: 5,
    });
    report.inventory_detail = vec![
        (
            Difference {
                room: 3,
                object: 9,
                seen: false,
                listed: true,
            },
            2,
        ),
        (
            Difference {
                room: 3,
                object: 8,
                seen: true,
                listed: false,
            },
            1,
        ),
    ];
    let written = report.write();
    assert!(
        written.contains(
            "no-verdict 1\ninventory compared 20 agreed 15 differed-known 2 differed-unexplained 1 unjudged 2 no-listing 1 empty 3 unknown 4 through-closed 5\ndetail\n"
        ),
        "{written}"
    );
    assert!(
        written.ends_with(
            "3 31 fixed 2 lost 1\ninventory 3 8 seen unlisted 1\ninventory 3 9 kept listed 2\n"
        ),
        "{written}"
    );
    report.inventory_detail.sort();
    assert_eq!(Report::read(&written), Ok(report));
    assert!(!sample().write().contains("inventory"));

    let with = |from: &str, to: &str| {
        assert!(written.contains(from), "{from}");
        Report::read(&written.replace(from, to))
            .expect_err("refused")
            .0
    };
    // The objects compared are those agreed, differed and unjudged.
    assert_eq!(with("compared 20", "compared 21"), 13);
    assert_eq!(with("through-closed 5", "through-closed many"), 13);
    assert_eq!(
        with(
            "inventory 3 8 seen unlisted 1",
            "inventory 3 8 seen listed 1"
        ),
        20
    );
    assert_eq!(
        with(
            "inventory 3 8 seen unlisted 1",
            "inventory 3 sack seen unlisted 1"
        ),
        20
    );
    // An exit's detail after an inventory's.
    let late = written.clone() + "3 31 fixed 2 stayed 1\n";
    assert_eq!(Report::read(&late).expect_err("refused").0, 22);
    // A detail line in a report with no comparison.
    let plain = sample().write() + "inventory 3 8 seen unlisted 1\n";
    assert_eq!(Report::read(&plain).expect_err("refused").0, 19);
}

// ---------------------------------------------------------------------------
// Light (specs/008-darkness/contracts/commands.md, "The corpus")
// ---------------------------------------------------------------------------

// FR-010: the command that asks a game to describe the room is data about the
// story. An aside may now compare light alone.
#[test]
fn a_manifest_may_give_a_look_command() {
    let with = ENTRY.replace(
        "opening: yes\n",
        "opening: yes\nlook: l\naside: north / up\n",
    );
    let stories = manifest::read(&with).expect("the manifest reads");
    assert_eq!(stories[1].look.as_deref(), Some("l"));
    assert_eq!(stories[1].asides.len(), 1);
    assert_eq!(stories[0].look, None);
    let twice = ENTRY.replace("opening: yes\n", "opening: yes\nlook: l\nlook: look\n");
    assert_eq!(manifest::read(&twice).unwrap_err().0, 27);
}

// R7: ledger entries about the light of a place, and about dark ahead.
#[test]
fn a_ledger_entry_may_be_about_light() {
    use zmachine_corpus::report::LightDifference;
    let place = |light: &str, described: &str| {
        format!(
            "story: toy\nroom: 4\nlight: {light}\ndescribed: {described}\ncause: The room is lit by the game's own code.\n"
        )
    };
    let read = ledger::read(&place("dark", "yes")).unwrap();
    assert_eq!(
        read.light[0].difference,
        LightDifference {
            room: 4,
            direction: None,
            dark: true,
            described: true
        }
    );
    assert!(ledger::read(&place("lit", "no")).is_ok());
    assert_eq!(ledger::read(&place("dark", "no")).unwrap_err().0, 1);
    assert_eq!(ledger::read(&place("lit", "yes")).unwrap_err().0, 1);
    assert_eq!(ledger::read(&place("dim", "yes")).unwrap_err().0, 3);
    assert_eq!(ledger::read(&place("dark", "maybe")).unwrap_err().0, 4);
    let ahead = "story: toy\nroom: 4\ndirection: 31\nahead: lit\ndescribed: no\ncause: Someone takes the light on the way.\n";
    let read = ledger::read(ahead).unwrap();
    assert_eq!(
        read.light[0].difference,
        LightDifference {
            room: 4,
            direction: Some(31),
            dark: false,
            described: false
        }
    );
    // The forms do not mix, and a cause must be written.
    assert_eq!(
        ledger::read(&ahead.replace("direction: 31\n", ""))
            .unwrap_err()
            .0,
        1
    );
    assert_eq!(
        ledger::read(&place("dark", "yes").replace("room: 4\n", "room: 4\ndirection: 31\n"))
            .unwrap_err()
            .0,
        1
    );
    assert_eq!(
        ledger::read(&ahead.replace("cause: Someone takes the light on the way.", "cause:"))
            .unwrap_err()
            .0,
        6
    );
}

// R7: a report of a story with a look command has two more lines and their
// detail; the trials counted in the dark are part of what adds up to the trials.
#[test]
fn a_report_may_hold_the_light_comparison() {
    use zmachine_corpus::report::{Light, LightDifference};
    let mut report = sample();
    report.light = Some(Light {
        compared: 12,
        agreed: 8,
        differed_known: 1,
        differed_unexplained: 1,
        unjudged: 2,
        dark: 3,
        rooms_both: 1,
        in_the_dark: 4,
        ahead_compared: 9,
        ahead_agreed: 6,
        ahead_differed_known: 1,
        ahead_differed_unexplained: 0,
        ahead_unjudged: 2,
    });
    report.trials += 4;
    report.light_detail = vec![
        (
            LightDifference {
                room: 3,
                direction: None,
                dark: true,
                described: true,
            },
            1,
        ),
        (
            LightDifference {
                room: 3,
                direction: None,
                dark: false,
                described: false,
            },
            1,
        ),
        (
            LightDifference {
                room: 5,
                direction: Some(31),
                dark: false,
                described: false,
            },
            1,
        ),
    ];
    let written = report.write();
    assert!(
        written.contains(
            "no-verdict 1\nlight compared 12 agreed 8 differed-known 1 differed-unexplained 1 unjudged 2 dark 3 rooms-both 1 in-the-dark 4\nahead compared 9 agreed 6 differed-known 1 differed-unexplained 0 unjudged 2\ndetail\n"
        ),
        "{written}"
    );
    assert!(
        written.ends_with(
            "light 3 lit undescribed 1\nlight 3 dark described 1\nahead 5 31 lit undescribed 1\n"
        ),
        "{written}"
    );
    report.light_detail.sort();
    assert_eq!(Report::read(&written), Ok(report));
    assert!(!sample().write().contains("light"));
    let with = |from: &str, to: &str| {
        assert!(written.contains(from), "{from}");
        Report::read(&written.replace(from, to))
            .expect_err("refused")
            .0
    };
    assert_eq!(with("light compared 12", "light compared 13"), 13);
    assert_eq!(with("ahead compared 9", "ahead compared 8"), 14);
    // One trial in the dark more than the trials account for.
    assert_eq!(with("in-the-dark 4", "in-the-dark 5"), 7);
    assert_eq!(
        with("light 3 dark described 1", "light 3 dark undescribed 1"),
        22
    );
    assert_eq!(
        with(
            "ahead 5 31 lit undescribed 1",
            "ahead 5 north lit undescribed 1"
        ),
        23
    );
    let plain = sample().write() + "light 3 dark described 1\n";
    assert_eq!(Report::read(&plain).expect_err("refused").0, 19);
}

// ---------------------------------------------------------------------------
// Look-ahead (feature 009; specs/009-compass-lookahead/contracts/corpus.md,
// cited C§)
// ---------------------------------------------------------------------------

fn looked() -> Report {
    let mut report = sample();
    report.lookahead = Some(zmachine_corpus::report::Looked {
        walked: 4,
        refused: 6,
        left: 2,
        missed_known: 1,
        missed_unexplained: 2,
    });
    // `fixed N` stands for "found walked to N", `none` for "found refused".
    report.lookahead_detail = vec![
        (case(3, 30, Reported::None, Outcome::Moved(4)), 2),
        (case(2, 28, Reported::Fixed(3), Outcome::Stayed), 1),
    ];
    report
}

// C§5, 6: the line is the last before `detail`, its findings not borne out are
// the last of the detail, and every other line is what it was.
#[test]
fn a_report_may_hold_look_aheads_figures() {
    let (plain, text) = (sample().write(), looked().write());
    let added: Vec<&str> = text
        .lines()
        .filter(|l| !plain.lines().any(|p| p == *l))
        .collect();
    assert_eq!(
        added,
        [
            "lookahead walked 4 refused 6 left 2 not-borne-out-known 1 not-borne-out-unexplained 2",
            "lookahead 2 28 walked 3 stayed 1",
            "lookahead 3 30 refused moved 4 2",
        ]
    );
    let before_detail = text.lines().position(|l| l == "detail").unwrap() - 1;
    assert!(
        text.lines()
            .nth(before_detail)
            .unwrap()
            .starts_with("lookahead walked ")
    );
    assert!(text.ends_with("lookahead 3 30 refused moved 4 2\n"));
    assert_eq!(text.lines().count(), plain.lines().count() + 3);
    // It reads back as written; the detail comes back sorted.
    let read = Report::read(&text).unwrap();
    assert_eq!(read.lookahead, looked().lookahead);
    assert_eq!(read.write(), text);
    // A report with no such ways says nothing of look-ahead.
    assert!(!plain.contains("lookahead"));
    assert_eq!(sample().line().figures.looked, [0; 4]);
    assert_eq!(looked().line().figures.looked, [4, 6, 2, 3]);
}

// C§5: a line that departs from the form is refused.
#[test]
fn a_report_whose_look_ahead_lines_depart_from_the_format_is_refused() {
    let text = looked().write();
    for (from, to) in [
        // More findings not borne out than findings.
        (
            "walked 4 refused 6 left 2 not-borne-out-known 1",
            "walked 1 refused 1 left 2 not-borne-out-known 1",
        ),
        ("lookahead walked 4", "lookahead strolled 4"),
        // A pairing that is borne out is no detail.
        (
            "lookahead 2 28 walked 3 stayed 1",
            "lookahead 2 28 walked 3 moved 3 1",
        ),
        (
            "lookahead 3 30 refused moved 4 2",
            "lookahead 3 30 refused stayed 2",
        ),
        (
            "lookahead 3 30 refused moved 4 2",
            "lookahead 3 30 refused moved 4",
        ),
        (
            "lookahead 3 30 refused moved 4 2",
            "lookahead 3 30 refused lost 2",
        ),
    ] {
        assert!(text.contains(from), "{from}");
        assert!(Report::read(&text.replace(from, to)).is_err(), "{to}");
    }
    // Detail of look-ahead with no line for it.
    let orphan = sample().write() + "lookahead 3 30 refused moved 4 2\n";
    assert!(Report::read(&orphan).is_err());
    // Another kind of detail after look-ahead's.
    assert!(Report::read(&(text + "2 28 none moved 3 1\n")).is_err());
}

// C§6 as built: a line of the summary ends with look-ahead's four numbers when
// it has any, and is what it was when it has none.
#[test]
fn the_summary_totals_look_ahead() {
    let mut summary = summary();
    let before = summary.write();
    assert!(!before.contains("lookahead"));
    summary.stories[0].figures.looked = [4, 6, 2, 1];
    summary.stories[1].figures.looked = [1, 0, 0, 0];
    let text = summary.write();
    assert_eq!(Summary::read(&text).unwrap(), summary);
    let all = text.lines().find(|l| l.starts_with("all ")).unwrap();
    assert!(
        all.ends_with(" lookahead walked 5 refused 6 left 2 not-borne-out 1"),
        "{all}"
    );
    let first = text.lines().next().unwrap();
    assert!(
        first.ends_with(" lookahead walked 4 refused 6 left 2 not-borne-out 1"),
        "{first}"
    );
    // A line that says nothing of look-ahead may not say "none" at length.
    let zero = first.replace(
        "walked 4 refused 6 left 2 not-borne-out 1",
        "walked 0 refused 0 left 0 not-borne-out 0",
    );
    assert!(Summary::read(&text.replace(first, &zero)).is_err());
}

// C§8: an entry for a finding not borne out.
#[test]
fn a_ledger_entry_may_be_on_a_finding_of_look_ahead() {
    let entry = "story: toy\nroom: 204\ndirection: 28\nlookahead: refused\noutcome: moved 203\n\
                 cause: The game decides by a draw.\n";
    let ledger = zmachine_corpus::ledger::read(entry).unwrap();
    assert_eq!(ledger.lookahead.len(), 1);
    assert!(ledger.entries.is_empty());
    assert!(ledger.knows_lookahead("toy", &case(204, 28, Reported::None, Outcome::Moved(203))));
    let walked = entry
        .replace("lookahead: refused", "lookahead: walked 7")
        .replace("moved 203", "stayed");
    let ledger = zmachine_corpus::ledger::read(&walked).unwrap();
    assert!(ledger.knows_lookahead("toy", &case(204, 28, Reported::Fixed(7), Outcome::Stayed)));
    for (from, to) in [
        // Borne out: nothing to explain.
        ("outcome: moved 203", "outcome: stayed"),
        ("lookahead: refused", "lookahead: walked 203"),
        ("lookahead: refused", "lookahead: perhaps"),
        // It has `lookahead` or `reported`, not both.
        (
            "lookahead: refused\n",
            "lookahead: refused\nreported: none\n",
        ),
        ("cause: The game decides by a draw.", "cause:"),
        ("direction: 28\n", ""),
    ] {
        assert!(entry.contains(from));
        assert!(
            zmachine_corpus::ledger::read(&entry.replace(from, to)).is_err(),
            "{to}"
        );
    }
    // The same case twice.
    assert!(zmachine_corpus::ledger::read(&format!("{entry}\n{entry}")).is_err());
}
