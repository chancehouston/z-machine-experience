//! What the interface says about light, set against whether the game describes
//! the room (specs/008-darkness/spec.md User Story 2; research R6, R7, R9;
//! docs/introspection.md I§12.9, I§12.10).

mod common;

use common::*;
use zmachine_core::Machine;
use zmachine_corpus::corpus::{Mode, command, corpus_lines};
use zmachine_corpus::light::described;
use zmachine_corpus::walk::Driver;
use zmachine_introspect::Introspector;

/// Three rooms in a row, the first lit of itself, with a lamp in it.
fn toy(looking: Looking) -> Toy {
    let mut toy = Toy::new(3);
    toy.way(0, NORTH, 1)
        .way(1, SOUTH, 0)
        .way(1, NORTH, 2)
        .way(2, SOUTH, 1);
    toy.lit_rooms = vec![0];
    toy.lamp = Some(looking);
    toy
}

fn lit_entry() -> String {
    format!("lit-attribute: {LIT_ATTR}\n")
}

/// The toy after these commands, and the interface for it with these entries.
fn at(toy: &Toy, commands: &[&str], entries: &str) -> (Machine, Introspector) {
    let image = toy.build();
    let mut driver = Driver::new(image.clone(), 1).unwrap();
    driver.stop();
    for command in commands {
        driver.answer(command);
        driver.stop();
    }
    let pack = pack_for(&image, entries).replace("pack: 1", "pack: 3");
    let i = Introspector::with_pack(&driver.machine.view(), &pack);
    assert_eq!(i.pack(), zmachine_introspect::PackState::InForce);
    (driver.machine, i)
}

// FR-009; research R6: the game described the room if its reply names it.
#[test]
fn a_copy_is_asked_to_describe_the_room() {
    let honest = toy(Looking::Honest);
    let (m, i) = at(&honest, &[], &lit_entry());
    assert_eq!(described(&m, &i, "look"), Some(true));
    let (m, i) = at(&honest, &["north"], &lit_entry());
    assert_eq!(described(&m, &i, "look"), Some(false));
    let (m, i) = at(&honest, &["take", "on", "north"], &lit_entry());
    assert_eq!(described(&m, &i, "look"), Some(true));
    let (m, i) = at(&honest, &["take", "on", "north", "off"], &lit_entry());
    assert_eq!(described(&m, &i, "look"), Some(false));
    // The game being walked is not touched by the asking.
    let memory = |m: &Machine| -> Vec<Option<u8>> {
        let v = m.view();
        (0..v.len()).map(|a| v.byte(a)).collect()
    };
    let before = memory(&m);
    described(&m, &i, "look");
    assert_eq!(memory(&m), before);
}

/// A scratch repository holding the toy, walked by `walk`, with a look command
/// and a pack of version 3 with these entries.
fn repository(name: &str, toy: &Toy, walk: &str, entries: &str) -> Scratch {
    let scratch = Scratch::new(name);
    let image = toy.build();
    let first: String = walk.lines().map(|l| format!("first: {l}\n")).collect();
    scratch.add("toy", &image, &format!("{first}look: look\n"));
    scratch.write(
        "packs/toy.pack",
        pack_for(&image, entries)
            .replace("pack: 1", "pack: 3")
            .as_bytes(),
    );
    scratch
}

fn run(scratch: &Scratch) -> (Vec<String>, i32) {
    let _ = command(&scratch.0, Mode::Write, &[]);
    command(&scratch.0, Mode::Run, &[])
}

fn line(scratch: &Scratch, start: &str) -> String {
    scratch
        .read("stories/corpus/reports/toy.txt")
        .lines()
        .find(|l| l.starts_with(start))
        .unwrap_or_else(|| panic!("no line {start}"))
        .to_string()
}

// US2 scenario 1; FR-011, FR-012: with the right attribute, and a room seen both
// dark and lit, the entry is proven; light agrees at every place and on every
// way walked.
#[test]
fn the_right_attribute_is_proven() {
    let scratch = repository(
        "proven",
        &toy(Looking::Honest),
        "north\nsouth\ntake\non\nnorth",
        &lit_entry(),
    );
    let (said, status) = run(&scratch);
    assert_eq!(
        (said.clone(), status),
        (
            vec![
                "toy entry lit-attribute 7: proven".to_string(),
                "toy ok".to_string()
            ],
            0
        )
    );
    let light = line(&scratch, "light ");
    assert!(
        light.contains(" differed-known 0 differed-unexplained 0 unjudged 0 "),
        "{light}"
    );
    assert!(
        !light.contains(" dark 0 ") && !light.contains(" rooms-both 0 "),
        "{light}"
    );
    let ahead = line(&scratch, "ahead ");
    assert!(
        ahead.contains(" differed-known 0 differed-unexplained 0 "),
        "{ahead}"
    );
    assert!(!ahead.contains("compared 0 "), "{ahead}");
}

// US2 scenario 2: a pack naming another attribute is contradicted; the run fails
// and names a place in numbers.
#[test]
fn a_wrong_attribute_is_contradicted() {
    let entries = format!("lit-attribute: {}\n", LIT_ATTR + 1);
    let scratch = repository(
        "contradicted",
        &toy(Looking::Honest),
        "north\nsouth\ntake\non\nnorth",
        &entries,
    );
    let (said, status) = run(&scratch);
    assert_eq!(status, 1);
    assert!(
        said.contains(&"toy entry lit-attribute 8: contradicted".to_string()),
        "{said:?}"
    );
    // The first room is lit of itself; the interface, reading the wrong
    // attribute, says dark, and the game describes it.
    assert!(
        said.contains(&"toy light wrong 2 dark described".to_string()),
        "{said:?}"
    );
}

// US2 scenario 3; FR-011: a walk that is never dark proves nothing.
#[test]
fn a_walk_that_is_never_dark_is_not_proven() {
    let mut bright = toy(Looking::Honest);
    bright.lit_rooms = vec![0, 1, 2];
    let scratch = repository("never-dark", &bright, "north", &lit_entry());
    let (said, status) = run(&scratch);
    assert_eq!(
        (said[0].as_str(), status),
        ("toy entry lit-attribute 7: not-proven", 1)
    );
    // Dark, but never lit by a light: the same room is not seen both ways.
    let scratch = repository("never-lamp", &toy(Looking::Honest), "north", &lit_entry());
    let (said, status) = run(&scratch);
    assert_eq!(
        (said[0].as_str(), status),
        ("toy entry lit-attribute 7: not-proven", 1)
    );
}

// FR-012, FR-014; US2 scenario 6: a difference about dark ahead fails, names the
// room and the direction, and passes with its cause in the ledger.
#[test]
fn a_difference_fails_until_it_is_explained() {
    // This toy names every room: the dark rooms look lit to the comparison.
    let scratch = repository(
        "lying",
        &toy(Looking::AlwaysNames),
        "north\nsouth\ntake\non\nnorth",
        &lit_entry(),
    );
    let (said, status) = run(&scratch);
    assert_eq!(status, 1);
    assert!(
        said.contains(&"toy light wrong 3 dark described".to_string()),
        "{said:?}"
    );
    let _ = command(&scratch.0, Mode::Write, &[]);
    let detail = scratch.read("stories/corpus/reports/toy.txt");
    let ledger: String = detail
        .lines()
        .filter(|l| l.starts_with("light ") && !l.starts_with("light compared") || l.starts_with("ahead ") && !l.starts_with("ahead compared"))
        .map(|l| {
            let w: Vec<&str> = l.split(' ').collect();
            let described = |game: &str| if game == "described" { "yes" } else { "no" };
            if w[0] == "light" {
                format!("story: toy\nroom: {}\nlight: {}\ndescribed: {}\ncause: This toy names its rooms in the dark.\n\n", w[1], w[2], described(w[3]))
            } else {
                format!("story: toy\nroom: {}\ndirection: {}\nahead: {}\ndescribed: {}\ncause: This toy names its rooms in the dark.\n\n", w[1], w[2], w[3], described(w[4]))
            }
        })
        .collect();
    assert!(
        ledger.contains("ahead: dark"),
        "a way was marked and the room described: {ledger}"
    );
    scratch.write("stories/corpus/ledger.txt", ledger.as_bytes());
    let (said, _) = run(&scratch);
    assert!(!said.iter().any(|l| l.contains(" wrong ")), "{said:?}");
    assert!(line(&scratch, "light ").contains(" differed-unexplained 0 "));
    // An entry that matches nothing has outlived its case.
    scratch.write(
        "stories/corpus/ledger.txt",
        b"story: toy\nroom: 9\nlight: dark\ndescribed: yes\ncause: Nothing.\n",
    );
    let (said, status) = run(&scratch);
    assert_eq!(status, 1);
    assert!(
        said.contains(&"stories/corpus/ledger.txt:1: the entry matches no comparison".to_string()),
        "{said:?}"
    );
}

// FR-013; US2 scenario 4: at a place the interface reports as dark, a report a
// trial does not bear out is counted as made in the dark, is not wrong, and
// needs no ledger entry. Elsewhere it is wrong as before.
#[test]
fn a_report_made_in_the_dark_is_not_counted_as_wrong() {
    // In the dark middle room the code goes east though the table has no east.
    let mut wandering = toy(Looking::Honest);
    wandering.does(1, EAST, Toy::room(0) as u8);
    let scratch = repository(
        "in-the-dark",
        &wandering,
        "north\nsouth\ntake\non\nnorth",
        &lit_entry(),
    );
    let (said, status) = run(&scratch);
    // With the lamp on, the same room is lit, and there the report is wrong.
    assert_eq!(status, 1);
    assert!(
        said.contains(&"toy wrong 3 29 none moved 2".to_string()),
        "{said:?}"
    );
    let light = line(&scratch, "light ");
    assert!(!light.ends_with(" in-the-dark 0"), "{light}");
    // Without the lamp ever lit, the room is only ever dark: nothing is wrong.
    let scratch = repository("only-dark", &wandering, "north\nsouth", &lit_entry());
    let (said, _) = run(&scratch);
    assert!(!said.iter().any(|l| l.contains("toy wrong")), "{said:?}");
    assert!(!line(&scratch, "light ").ends_with(" in-the-dark 0"));
    let written = scratch.read("stories/corpus/reports/toy.txt");
    assert!(
        !written.contains("\n3 29 none moved"),
        "not in the detail: {written}"
    );
    // With no look command nothing is forgiven.
    let plain = Scratch::new("no-look");
    let image = wandering.build();
    plain.add("toy", &image, "first: north\n");
    let (said, status) = run(&plain);
    assert_eq!(status, 1);
    assert!(
        said.contains(&"toy wrong 3 29 none moved 2".to_string()),
        "{said:?}"
    );
}

// FR-016; US2 scenario 7: an absent story's tests are ignored, not passed.
#[test]
fn an_absent_story_is_ignored() {
    let scratch = repository("absent", &toy(Looking::Honest), "north", &lit_entry());
    std::fs::remove_file(scratch.path("stories/toy/toy.z3")).unwrap();
    let (said, passed) = corpus_lines(&scratch.0);
    assert!(passed);
    assert!(
        said.contains(&"test toy ... ignored, story absent".to_string()),
        "{said:?}"
    );
}

/// Zork I's run with a pack of these entries.
fn zork1_with(entries: &str) -> zmachine_corpus::report::Report {
    let root = std::path::Path::new(ROOT);
    let (stories, ledger) = zmachine_corpus::corpus::load(root).unwrap();
    let story = stories.iter().find(|s| s.id == "zork1").unwrap();
    assert!(story.look.is_some() && !story.asides.is_empty());
    let image = read("stories/zork1/zork1.z3");
    let script = read_text("stories/zork1/fidelity/walkthrough.txt");
    let pack = pack_for(&image, entries).replace("pack: 1", "pack: 3");
    zmachine_corpus::walk::run_story_with(story, image, Some(&script), &ledger, Some(&pack), None).0
}

// SC-003; research R9: on Zork I, attribute 19 is proven to mean lit, and a pack
// naming any of the other 31 is contradicted by the game or proves nothing.
#[test]
fn only_attribute_19_is_zork1s_lit() {
    use zmachine_corpus::packs::{Proof, prove_lit};
    for attribute in 0..32 {
        let report = zork1_with(&format!(
            "open-attribute: 11\ntransparent-attribute: 12\nlit-attribute: {attribute}\ninvisible-attribute: 7\n"
        ));
        let proof = prove_lit(&report);
        assert_eq!(
            proof == Proof::Proven,
            attribute == 19,
            "attribute {attribute}: {proof:?} {:?}",
            report.light
        );
    }
}

// I§12.11: on Zork I, attribute 7 is proven to mean hidden: without the entry the
// game and the interface differ about light, and with it they do not. One other
// attribute does the same on this evidence, 30: it is set on the character who
// takes the light and never changes, so naming it hides whatever that character
// holds. The game sets and clears 7 on the taken thing itself, and 7 is the one
// the pack names; the run cannot tell the two apart (packs/README.md).
#[test]
fn attribute_7_is_zork1s_invisible_and_only_30_does_as_well() {
    use zmachine_corpus::packs::{Proof, prove_invisible};
    let rest = "open-attribute: 11\ntransparent-attribute: 12\nlit-attribute: 19\n";
    let without = zork1_with(rest);
    let mut standing = Vec::new();
    for attribute in 0..32 {
        let report = zork1_with(&format!("{rest}invisible-attribute: {attribute}\n"));
        if prove_invisible(&report, &without) == Proof::Proven {
            standing.push(attribute);
        }
    }
    assert_eq!(standing, [7, 30]);
}
