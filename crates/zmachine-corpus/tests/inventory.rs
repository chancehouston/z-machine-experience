//! What the interface says is seen of the inventory, set against the game's own
//! listing (specs/007-inventory-panel/spec.md User Story 2; research R3, R4, R8;
//! docs/introspection.md I§7, I§7.2, I§12.8).

mod common;

use common::*;
use zmachine_core::Machine;
use zmachine_corpus::inventory::{Asked, Compared, ask};
use zmachine_corpus::walk::Driver;
use zmachine_introspect::Introspector;

/// A toy with things to carry, after these commands, at a command prompt; and
/// the interface for it with a pack of these entries (none for no pack).
fn at(toy: &Toy, commands: &[&str], entries: Option<&str>) -> (Machine, Introspector) {
    let image = toy.build();
    let mut driver = Driver::new(image.clone(), 1).unwrap();
    driver.stop();
    for command in commands {
        driver.answer(command);
        driver.stop();
    }
    let view = driver.machine.view();
    let introspector = match entries {
        Some(entries) => {
            let pack = pack_for(&image, entries).replace("pack: 1", "pack: 2");
            let i = Introspector::with_pack(&view, &pack);
            assert_eq!(i.pack(), zmachine_introspect::PackState::InForce);
            i
        }
        None => Introspector::new(&view),
    };
    (driver.machine, introspector)
}

fn toy(listing: Listing) -> Toy {
    let mut toy = Toy::new(2);
    toy.way(0, NORTH, 1).way(1, SOUTH, 0);
    toy.carrying = Some(listing);
    toy
}

fn open() -> String {
    format!("open-attribute: {OPEN_ATTR}\n")
}

fn compared(asked: Asked) -> Vec<Compared> {
    match asked {
        Asked::Listed(compared) => compared,
        other => panic!("not a listing: {other:?}"),
    }
}

// FR-011; data-model "Comparison": an honest listing agrees with the marks,
// closed and open.
#[test]
fn an_honest_listing_agrees() {
    let toy = toy(Listing::Honest);
    let (bag, gem, key) = toy.things();
    let (machine, i) = at(&toy, &["take"], Some(&open()));
    assert_eq!(
        compared(ask(&machine, &i, "inventory")),
        [
            Compared {
                object: bag,
                seen: Some(true),
                listed: true,
                through_closed: false
            },
            Compared {
                object: gem,
                seen: Some(false),
                listed: false,
                through_closed: false
            },
            Compared {
                object: key,
                seen: Some(true),
                listed: true,
                through_closed: false
            },
        ]
    );
    let (machine, i) = at(&toy, &["take", "open"], Some(&open()));
    let all = compared(ask(&machine, &i, "inventory"));
    assert_eq!(
        all[1],
        Compared {
            object: gem,
            seen: Some(true),
            listed: true,
            through_closed: false
        }
    );
}

// R8: a thing seen through a closed, see-through thing is the evidence a
// `transparent-attribute` needs.
#[test]
fn a_thing_seen_through_a_closed_thing_is_marked_so() {
    let mut toy = toy(Listing::Honest);
    toy.clear_bag = true;
    let (_, gem, _) = toy.things();
    let both = format!("{}transparent-attribute: {CLEAR_ATTR}\n", open());
    let (machine, i) = at(&toy, &["take"], Some(&both));
    let all = compared(ask(&machine, &i, "inventory"));
    assert_eq!(
        all[1],
        Compared {
            object: gem,
            seen: Some(true),
            listed: true,
            through_closed: true
        }
    );
    // Opened, it is seen because it is open, not because it is see-through.
    let (machine, i) = at(&toy, &["take", "open"], Some(&both));
    assert!(!compared(ask(&machine, &i, "inventory"))[1].through_closed);
    // Without the entry the gem is kept back, and the game lists it: a difference.
    let (machine, i) = at(&toy, &["take"], Some(&open()));
    let gem_now = compared(ask(&machine, &i, "inventory"))[1];
    assert_eq!((gem_now.seen, gem_now.listed), (Some(false), true));
}

// D§ "Comparison": the two ways to differ.
#[test]
fn a_listing_that_differs_is_seen_to() {
    let (_, gem, _) = toy(Listing::Always).things();
    let (machine, i) = at(&toy(Listing::Always), &["take"], Some(&open()));
    let got = compared(ask(&machine, &i, "inventory"))[1];
    assert_eq!((got.object, got.seen, got.listed), (gem, Some(false), true));
    let (machine, i) = at(&toy(Listing::Never), &["take", "open"], Some(&open()));
    let got = compared(ask(&machine, &i, "inventory"))[1];
    assert_eq!((got.object, got.seen, got.listed), (gem, Some(true), false));
}

// R4: nothing carried is empty; a reply that names nothing carried is no listing.
// I§7.2: with no pack what is in a container is unknown, and is not judged.
#[test]
fn empty_no_listing_and_unknown() {
    let (machine, i) = at(&toy(Listing::Honest), &[], Some(&open()));
    assert_eq!(ask(&machine, &i, "inventory"), Asked::Empty);
    let (machine, i) = at(&toy(Listing::Nonsense), &["take"], Some(&open()));
    assert_eq!(ask(&machine, &i, "inventory"), Asked::NoListing);
    let (machine, i) = at(&toy(Listing::Honest), &["take"], None);
    let all = compared(ask(&machine, &i, "inventory"));
    assert_eq!(all[1].seen, None);
    assert_eq!((all[0].seen, all[2].seen), (Some(true), Some(true)));
}

// FR-011: the game being walked is not touched by the asking.
#[test]
fn asking_changes_nothing() {
    let (machine, i) = at(&toy(Listing::Honest), &["take", "open"], Some(&open()));
    let memory = |m: &Machine| -> Vec<Option<u8>> {
        let v = m.view();
        (0..v.len()).map(|a| v.byte(a)).collect()
    };
    let before = memory(&machine);
    for _ in 0..3 {
        ask(&machine, &i, "inventory");
    }
    assert_eq!(memory(&machine), before);
    assert_eq!(
        machine.waiting_at(),
        at(&toy(Listing::Honest), &["take", "open"], None)
            .0
            .waiting_at()
    );
}

// ---------------------------------------------------------------------------
// The run: differences fail until explained; the entry is proven or not
// (FR-013 to FR-015, FR-017; research R8)
// ---------------------------------------------------------------------------

use zmachine_corpus::corpus::{Mode, command, corpus_lines};

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|l| l.to_string()).collect()
}

/// A scratch repository holding a toy with things to carry, walked by `walk`,
/// with an inventory command, further manifest lines, and a pack of version 2
/// holding these entries.
fn repository(name: &str, toy: &Toy, walk: &str, extra: &str, entries: &str) -> Scratch {
    let scratch = Scratch::new(name);
    let image = toy.build();
    let first: String = walk.lines().map(|l| format!("first: {l}\n")).collect();
    scratch.add(
        "toy",
        &image,
        &format!("{first}inventory: inventory\n{extra}"),
    );
    scratch.write(
        "packs/toy.pack",
        pack_for(&image, entries)
            .replace("pack: 1", "pack: 2")
            .as_bytes(),
    );
    scratch
}

fn both() -> String {
    format!("{}transparent-attribute: {CLEAR_ATTR}\n", open())
}

fn run(scratch: &Scratch) -> (Vec<String>, i32) {
    let _ = command(&scratch.0, Mode::Write, &[]);
    command(&scratch.0, Mode::Run, &[])
}

// US2 scenario 1: where the listing is honest and the marks are right, the run
// passes, and the report counts what was compared.
#[test]
fn an_honest_game_and_a_right_pack_pass() {
    let scratch = repository(
        "honest",
        &toy(Listing::Honest),
        "take\nopen\nclose",
        "",
        &open(),
    );
    let (said, status) = run(&scratch);
    assert_eq!(
        status, 1,
        "the door-less toy cannot prove an open-attribute by doors: {said:?}"
    );
    assert!(
        said.contains(&"toy entry open-attribute 9: not-proven".to_string()),
        "{said:?}"
    );
    assert!(
        !said.iter().any(|l| l.contains("inventory wrong")),
        "{said:?}"
    );
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    // Four prompts on the walk: nothing carried; closed; open; closed: three
    // listings of three objects. Then the crawl's one place, the other room,
    // reached from the first prompt, where nothing is carried yet.
    assert!(
        kept.contains(
            "\ninventory compared 9 agreed 9 differed-known 0 differed-unexplained 0 unjudged 0 no-listing 0 empty 2 unknown 0 through-closed 0\n"
        ),
        "{kept}"
    );
}

// US2 scenario 2; FR-013: a difference with no cause fails the run and is named
// in numbers. Scenario 4; FR-015: with its cause in the ledger it passes.
#[test]
fn a_difference_fails_until_it_is_explained() {
    let toy = toy(Listing::Always);
    let (_, gem, _) = toy.things();
    let scratch = repository("differs", &toy, "take", "", "");
    let (said, status) = run(&scratch);
    assert_eq!(
        status, 0,
        "with no entry the gem is unknown, and nothing is judged: {said:?}"
    );

    let scratch = repository("differs-open", &toy, "take", "", &open());
    let (said, status) = run(&scratch);
    assert_eq!(status, 1);
    let wrong = format!("toy inventory wrong 2 {gem} kept listed");
    assert!(said.contains(&wrong), "{said:?}");
    scratch.write(
        "stories/corpus/ledger.txt",
        format!("story: toy\nroom: 2\nobject: {gem}\nseen: kept\nlisted: listed\ncause: This toy lists what is in its bag whatever the bag's state.\n").as_bytes(),
    );
    let (said, _) = run(&scratch);
    assert!(
        !said.iter().any(|l| l.contains("inventory wrong")),
        "{said:?}"
    );
    let kept = scratch.read("stories/corpus/reports/toy.txt");
    assert!(kept.contains(" differed-unexplained 0 "), "{kept}");
    assert!(
        kept.contains(&format!("\ninventory 2 {gem} kept listed ")),
        "{kept}"
    );
    // An entry that matches no comparison has outlived its case.
    scratch.write(
        "stories/corpus/ledger.txt",
        format!(
            "story: toy\nroom: 3\nobject: {gem}\nseen: kept\nlisted: listed\ncause: Nothing.\n"
        )
        .as_bytes(),
    );
    let (said, status) = run(&scratch);
    assert_eq!(status, 1);
    assert!(
        said.contains(&"stories/corpus/ledger.txt:1: the entry matches no comparison".to_string()),
        "{said:?}"
    );
}

// FR-013; US2 scenarios 2 and 3: a see-through entry is proven by a thing seen
// through a closed thing and listed; contradicted by a difference; and not
// proven where the walk has no such place.
#[test]
fn a_see_through_entry_is_proven_contradicted_or_not_proven() {
    let entry = |said: &[String]| {
        said.iter()
            .find(|l| l.contains("transparent-attribute"))
            .cloned()
            .unwrap_or_default()
    };
    let mut clear = toy(Listing::Honest);
    clear.clear_bag = true;
    let (said, _) = run(&repository("proven", &clear, "take", "", &both()));
    assert_eq!(entry(&said), "toy entry transparent-attribute 8: proven");

    // The bag is not see-through, and the pack names an attribute it does not
    // have: nothing is ever seen through a closed thing.
    let (said, status) = run(&repository(
        "not-proven",
        &toy(Listing::Honest),
        "take",
        "",
        &both(),
    ));
    assert_eq!(
        entry(&said),
        "toy entry transparent-attribute 8: not-proven"
    );
    assert_eq!(status, 1);

    // The pack calls the bag see-through and the game does not list the gem.
    let mut liar = toy(Listing::Never);
    liar.clear_bag = true;
    let (said, status) = run(&repository("contradicted", &liar, "take", "", &both()));
    assert_eq!(
        entry(&said),
        "toy entry transparent-attribute 8: contradicted"
    );
    assert_eq!(status, 1);
}

// R2: an aside is a short walk from the start at whose prompts the inventory is
// compared and nothing else is done: it supplies the place a proof needs, and
// adds no trial of an exit.
#[test]
fn an_aside_supplies_a_place_and_tries_no_exit() {
    let mut clear = toy(Listing::Honest);
    clear.clear_bag = true;
    let without = repository("no-aside", &clear, "", "", &both());
    let (said, _) = run(&without);
    assert!(
        said.contains(&"toy entry transparent-attribute 8: not-proven".to_string()),
        "{said:?}"
    );
    let with = repository(
        "aside",
        &clear,
        "",
        "aside: north / south / take\n",
        &both(),
    );
    let (said, _) = run(&with);
    assert!(
        said.contains(&"toy entry transparent-attribute 8: proven".to_string()),
        "{said:?}"
    );
    let line = |scratch: &Scratch, start: &str| {
        scratch
            .read("stories/corpus/reports/toy.txt")
            .lines()
            .find(|l| l.starts_with(start))
            .unwrap()
            .to_string()
    };
    assert_eq!(line(&with, "trials "), line(&without, "trials "));
    assert_eq!(line(&with, "places "), line(&without, "places "));
    assert_ne!(line(&with, "inventory "), line(&without, "inventory "));
}

// FR-017; US2 scenario 5: where the story's file is absent, its tests are
// reported as ignored, story absent.
#[test]
fn an_absent_story_is_ignored() {
    let mut clear = toy(Listing::Honest);
    clear.clear_bag = true;
    let scratch = repository("absent", &clear, "take", "", &both());
    std::fs::remove_file(scratch.path("stories/toy/toy.z3")).unwrap();
    let (said, passed) = corpus_lines(&scratch.0);
    assert!(passed);
    assert!(
        said.contains(&"test toy ... ignored, story absent".to_string()),
        "{said:?}"
    );
}

// SC-003; research R2: on Zork I, with the short aside its manifest entry gives
// (it carries a closed thing that can be seen through), attribute 12 is proven
// to mean see-through, and a pack naming any of the other 31 is contradicted by
// the game's own listing.
#[test]
fn only_attribute_12_is_zork1s_see_through() {
    use zmachine_corpus::packs::{Proof, prove_transparent};
    let root = std::path::Path::new(ROOT);
    let (stories, ledger) = zmachine_corpus::corpus::load(root).unwrap();
    let story = stories.iter().find(|s| s.id == "zork1").unwrap();
    assert!(!story.asides.is_empty(), "zork1 has an aside");
    let image = read("stories/zork1/zork1.z3");
    let script = read_text("stories/zork1/fidelity/walkthrough.txt");
    for attribute in 0..32 {
        let pack = pack_for(
            &image,
            &format!("open-attribute: 11\ntransparent-attribute: {attribute}\n"),
        )
        .replace("pack: 1", "pack: 2");
        let (report, _, _) = zmachine_corpus::walk::run_story_with(
            story,
            image.clone(),
            Some(&script),
            &ledger,
            Some(&pack),
            None,
        );
        let expected = if attribute == 12 {
            Proof::Proven
        } else {
            Proof::Contradicted
        };
        assert_eq!(
            prove_transparent(&report),
            expected,
            "attribute {attribute}: {:?}",
            report.inventory
        );
    }
}
