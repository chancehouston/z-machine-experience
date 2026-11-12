//! Every entry of a pack is proven by trials, or the run fails
//! (specs/006-game-packs/spec.md User Story 2, FR-016 to FR-021; research R11).
//! The published rules these trials test are docs/introspection.md I§12.3 and
//! I§12.4.

mod common;

use common::*;
use zmachine_corpus::corpus::{Mode, command, corpus_lines, load};
use zmachine_corpus::packs::{Proof, prove_open};
use zmachine_corpus::walk::run_story_with;

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|l| l.to_string()).collect()
}

/// Two rooms with a door north from the first, and a way back.
fn door_toy() -> Toy {
    let mut toy = Toy::new(2);
    toy.way(1, SOUTH, 0);
    toy.door = Some((0, NORTH, 1));
    toy
}

/// A scratch repository holding a toy, its walkthrough and its pack.
fn scratch(name: &str, toy: &Toy, walk: &str, entries: &str) -> Scratch {
    let scratch = Scratch::new(name);
    let image = toy.build();
    scratch.write("stories/toy/walk.txt", walk.as_bytes());
    scratch.add("toy", &image, "walkthrough: stories/toy/walk.txt\n");
    scratch.pack("toy", &image, entries);
    scratch
}

fn written(scratch: &Scratch) {
    let (said, status) = command(&scratch.0, Mode::Write, &[]);
    assert_eq!(status, 0, "{said:?}");
    assert_eq!(
        said.last().map(String::as_str),
        Some("toy written"),
        "{said:?}"
    );
}

// US2 scenario 2; FR-017: with the right attribute, and the door seen both
// closed and open, the entry is proven and the run passes.
#[test]
fn the_right_attribute_is_proven() {
    let entries = format!("open-attribute: {OPEN_ATTR}\n");
    let scratch = scratch("proven", &door_toy(), "open\nlook\n", &entries);
    written(&scratch);
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    assert!(kept.contains("\npack in-force entries 1\n"), "{kept}");
    assert!(
        kept.contains("\nopen confirmed 2 wrong-known 0 wrong-unexplained 0\n"),
        "{kept}"
    );
    assert!(
        kept.contains("\nclosed confirmed 1 wrong-known 0 wrong-unexplained 0 other 0\n"),
        "{kept}"
    );
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (lines(&["toy entry open-attribute 9: proven", "toy ok"]), 0)
    );
}

// US2 scenario 1: a pack that names another attribute is contradicted, the run
// fails, and the case is named: room, direction, what was reported, what the
// copy did.
#[test]
fn a_wrong_attribute_is_contradicted_and_the_case_is_named() {
    let entries = format!("open-attribute: {}\n", OPEN_ATTR + 1);
    let scratch = scratch("contradicted", &door_toy(), "open\nlook\n", &entries);
    written(&scratch);
    let (said, status) = command(&scratch.0, Mode::Run, &[]);
    assert_eq!(status, 1, "{said:?}");
    assert_eq!(
        said,
        lines(&[
            "toy entry open-attribute 10: contradicted",
            "toy wrong 2 31 closed 3 moved 3",
        ])
    );
}

// US2 scenario 3; FR-017: a door seen in one state only proves nothing, since
// many attributes are clear on a closed door; the run fails.
#[test]
fn a_door_seen_in_one_state_is_not_proven() {
    let entries = format!("open-attribute: {OPEN_ATTR}\n");
    let scratch = scratch("one-state", &door_toy(), "look\n", &entries);
    written(&scratch);
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (lines(&["toy entry open-attribute 9: not-proven"]), 1)
    );
}

// FR-019: a wrong report on a door, with its cause in the ledger, is known, and
// the entry is still proven.
#[test]
fn a_wrong_report_on_a_door_may_be_explained() {
    let mut toy = door_toy();
    // A second way through the same door, which the code never honours.
    let door = toy.door_object() as u8;
    toy.table(1, EAST, vec![Toy::room(0) as u8, door, 0, 0, 0]);
    let entries = format!("open-attribute: {OPEN_ATTR}\n");
    let scratch = scratch("explained", &toy, "open\nnorth\nlook\n", &entries);
    written(&scratch);
    let (said, status) = command(&scratch.0, Mode::Run, &[]);
    assert_eq!(status, 1);
    assert!(
        said.contains(&"toy wrong 3 29 open 2 stayed".to_string()),
        "{said:?}"
    );
    assert!(
        said.contains(&"toy entry open-attribute 9: contradicted".to_string()),
        "{said:?}"
    );
    scratch.write(
        "stories/corpus/ledger.txt",
        b"story: toy\nroom: 3\ndirection: 29\nreported: open 2\noutcome: stayed\n\
          cause: Something other than the door stands in the way, for this test.\n",
    );
    written(&scratch);
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (lines(&["toy entry open-attribute 9: proven", "toy ok"]), 0)
    );
}

/// Two rooms, and a way east on a variable that holds 1: the table says the way
/// is open, and the code goes only if `honoured`.
fn flag_toy(honoured: bool) -> Toy {
    let mut toy = Toy::new(2);
    toy.way(1, SOUTH, 0).flag_way(0, EAST, 1, 30, honoured);
    toy
}

// US2 scenario 4; FR-018: a variable is proven untrusted when a report that
// hangs on it is wrong without the entry and is not with it.
#[test]
fn an_untrusted_variable_is_proven_by_a_wrong_report_without_it() {
    let scratch = scratch(
        "variable",
        &flag_toy(false),
        "look\n",
        "untrusted-variable: 46\n",
    );
    written(&scratch);
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    assert!(kept.contains("\ndistrusted 2 moved 0 stayed 2"), "{kept}");
    assert!(
        kept.contains("\nfixed confirmed 0 wrong-known 0 wrong-unexplained 0\n"),
        "{kept}"
    );
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (
            lines(&["toy entry untrusted-variable 46: proven", "toy ok"]),
            0
        )
    );
}

// FR-018: an entry that changes no verdict fails as unneeded: a variable whose
// exit is never wrong, and a variable no exit hangs on.
#[test]
fn an_untrusted_variable_that_changes_no_verdict_is_unneeded() {
    for (toy, variable) in [(flag_toy(true), 46), (flag_toy(false), 50)] {
        let entries = format!("untrusted-variable: {variable}\n");
        let scratch = scratch("unneeded", &toy, "look\n", &entries);
        let (said, _) = command(&scratch.0, Mode::Write, &[]);
        assert!(said.contains(&"toy written".to_string()), "{said:?}");
        let (said, status) = command(&scratch.0, Mode::Run, &[]);
        assert_eq!(status, 1, "{said:?}");
        assert!(
            said.contains(&format!(
                "toy entry untrusted-variable {variable}: unneeded"
            )),
            "{said:?}"
        );
    }
}

// Each entry of a pack is proven on its own: with two variables, one needed and
// one not, the one fails and the other passes.
#[test]
fn each_entry_is_proven_on_its_own() {
    let mut toy = flag_toy(false);
    toy.flag_way(1, WEST, 0, 31, true);
    toy.door = Some((0, NORTH, 1));
    let entries =
        format!("open-attribute: {OPEN_ATTR}\nuntrusted-variable: 46\nuntrusted-variable: 47\n");
    let scratch = scratch("each", &toy, "open\nlook\n", &entries);
    let _ = command(&scratch.0, Mode::Write, &[]);
    let (said, status) = command(&scratch.0, Mode::Run, &[]);
    assert_eq!(status, 1);
    assert_eq!(
        said,
        lines(&[
            "toy entry open-attribute 9: proven",
            "toy entry untrusted-variable 46: proven",
            "toy entry untrusted-variable 47: unneeded",
        ])
    );
}

// FR-020; US2 scenario 5: a pack whose story is absent has its test reported as
// ignored, story absent, and not as passed; a pack whose story ran is a test.
#[test]
fn a_pack_whose_story_is_absent_is_ignored_not_passed() {
    let entries = format!("open-attribute: {OPEN_ATTR}\n");
    let scratch = scratch("absent", &door_toy(), "open\nlook\n", &entries);
    written(&scratch);
    let (said, passed) = corpus_lines(&scratch.0);
    assert!(passed, "{said:?}");
    assert!(said.contains(&"test toy ... ok".to_string()), "{said:?}");
    assert!(
        said.contains(&"test pack toy ... ok".to_string()),
        "{said:?}"
    );
    assert!(said.contains(&"test result: ok. 2 passed; 0 failed; 0 ignored".to_string()));

    std::fs::remove_file(scratch.path("stories/toy/toy.z3")).unwrap();
    let (said, passed) = corpus_lines(&scratch.0);
    assert!(passed, "{said:?}");
    assert!(
        said.contains(&"test pack toy ... ignored, story absent".to_string()),
        "{said:?}"
    );
    assert!(said.contains(&"test result: ok. 0 passed; 0 failed; 2 ignored".to_string()));
}

// SC-004; research R5: on Zork I, attribute 11 is proven to mean "open", and a
// pack that names any of the other 31 attributes is contradicted.
#[test]
fn only_attribute_11_is_zork1s_open() {
    let root = std::path::Path::new(ROOT);
    let (stories, ledger) = load(root).unwrap();
    let story = stories.iter().find(|s| s.id == "zork1").unwrap();
    let image = read("stories/zork1/zork1.z3");
    let script = read_text("stories/zork1/fidelity/walkthrough.txt");
    let mut outcomes = Vec::new();
    for attribute in 0..32 {
        let pack = pack_for(&image, &format!("open-attribute: {attribute}\n"));
        let (report, _, _) = run_story_with(
            story,
            image.clone(),
            Some(&script),
            &ledger,
            Some(&pack),
            None,
        );
        let m = &report.marked;
        let doors = m.open_confirmed
            + m.open_wrong_known
            + m.open_wrong_unexplained
            + m.closed_confirmed
            + m.closed_wrong_known
            + m.closed_wrong_unexplained
            + m.closed_other;
        assert!(
            doors >= 57,
            "attribute {attribute}: {doors} trials of doors"
        );
        outcomes.push(prove_open(&report));
    }
    for (attribute, outcome) in outcomes.iter().enumerate() {
        let expected = if attribute == 11 {
            Proof::Proven
        } else {
            Proof::Contradicted
        };
        assert_eq!(*outcome, expected, "attribute {attribute}");
    }
}
