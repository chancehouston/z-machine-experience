//! The inventory card across the corpus (specs/014-inventory-card, SC-001,
//! SC-003): every story the repository can read is wandered, taking what it
//! can, and every item that can be tapped is peeked at every stop. The game
//! must be exactly as it was after each peek. Counts only are printed: no
//! name, word or text of any story.

mod common;

use common::repo;
use std::path::Path;
use zmachine_corpus::corpus;
use zmachine_experience::{Seed, Session, StripItem};

fn each(items: &[StripItem]) -> Vec<&StripItem> {
    items
        .iter()
        .flat_map(|item| std::iter::once(item).chain(each(&item.contents)))
        .collect()
}

#[test]
fn every_item_that_can_be_tapped_is_peeked_and_the_game_is_unchanged() {
    let root = repo("");
    let root = Path::new(&root);
    let (stories, _) = corpus::load(root).expect("the manifest and the ledger are read");
    let mut peeked_somewhere = 0;
    for story in &stories {
        let Some(bytes) = corpus::present(root, story) else {
            continue;
        };
        let pack = std::fs::read_to_string(root.join(format!("packs/{}.pack", story.id))).ok();
        let Ok(mut session) = Session::with_pack(bytes, Seed::Fixed(7), pack.as_deref()) else {
            continue;
        };
        let mut state = 99u32;
        let (mut shown, mut tappable, mut peeks, mut described) = (0, 0, 0, 0);
        for step in 0..120 {
            let items: Vec<(usize, bool)> = each(&session.inventory().items)
                .iter()
                .map(|item| (item.index, item.card))
                .collect();
            shown += items.len();
            for (index, card) in items {
                if !card {
                    assert!(
                        session.card(session.stamp(), index).is_err(),
                        "{}",
                        story.id
                    );
                    continue;
                }
                tappable += 1;
                let (kept, snapshot) = (session.kept(), session.snapshot_json(0));
                let card = session
                    .card(session.stamp(), index)
                    .unwrap_or_else(|e| panic!("{}: an offered card was refused: {e:?}", story.id));
                peeks += 1;
                described += usize::from(card.description.is_some());
                assert_eq!(
                    card.description.is_none(),
                    card.note.is_some(),
                    "{}",
                    story.id
                );
                assert!(
                    session.kept() == kept,
                    "{}: the kept game changed",
                    story.id
                );
                assert!(
                    session.snapshot_json(0) == snapshot,
                    "{}: the snapshot changed",
                    story.id
                );
            }
            // Every third move, try to pick things up; otherwise walk.
            let ways: Vec<String> = session
                .map()
                .view()
                .map(|view| {
                    view.ways
                        .into_iter()
                        .filter_map(|way| way.command)
                        .collect()
                })
                .unwrap_or_default();
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let line = if step % 3 == 0 || ways.is_empty() {
                "take all".to_string()
            } else {
                ways[(state >> 16) as usize % ways.len()].clone()
            };
            if session.send_line(session.stamp(), &line).is_err() {
                break;
            }
        }
        if peeks > 0 {
            peeked_somewhere += 1;
        }
        println!(
            "{}: {shown} items shown over the wander, {tappable} could be tapped, {peeks} peeks, {described} gave a description",
            story.id
        );
    }
    assert!(
        peeked_somewhere >= 4,
        "{peeked_somewhere} stories gave a peek"
    );
}

// specs/014-inventory-card/contracts/card.md, P§1 (research R13): along the
// walkthroughs, each action a card offers is tried on a copy of the game, made
// by bringing back what the game has kept, and must do what it says: after
// Open the card offers Close, and the other way; after Turn on, Turn off, and
// the other way; after Wear, Take off; after Eat or Drink the thing is no
// longer carried; Read and Drop leave the game waiting for a command, with
// something said. Each distinct item and set of actions is tried once.
// Counts only are printed.
#[test]
fn every_action_offered_does_what_it_says() {
    use std::collections::{BTreeMap, BTreeSet};
    let root = repo("");
    let read = |path: &str| std::fs::read(format!("{root}/{path}")).unwrap();
    let text = |path: &str| String::from_utf8(read(path)).unwrap();
    for (name, story, pack, script) in [
        (
            "zork1",
            "stories/zork1/zork1.z3",
            Some("packs/zork1.pack"),
            "stories/zork1/fidelity/walkthrough.txt",
        ),
        (
            "advent",
            "stories/advent/advent.z5",
            None,
            "stories/advent/fidelity/walkthrough.txt",
        ),
    ] {
        let pack = pack.map(text);
        let open = || Session::with_pack(read(story), Seed::Fixed(11), pack.as_deref()).unwrap();
        let mut session = open();
        let mut tried: BTreeSet<(String, Vec<String>)> = BTreeSet::new();
        // For each label: how many times it did what it says, and did not.
        let mut counts: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        let mut differed: Vec<String> = Vec::new();
        for line in text(script).lines().skip(2) {
            if session.send_line(session.stamp(), line).is_err() {
                break;
            }
            for item in each(&session.inventory().items) {
                if !item.card {
                    continue;
                }
                let Ok(card) = session.card(session.stamp(), item.index) else {
                    continue;
                };
                let labels: Vec<String> = card.actions.iter().map(|a| a.label.clone()).collect();
                if labels.is_empty() || !tried.insert((item.name.clone(), labels.clone())) {
                    continue;
                }
                for (place, label) in labels.iter().enumerate() {
                    // A copy: the game as kept, brought back.
                    let (mut copy, _) = Session::open_kept(
                        read(story),
                        pack.as_deref(),
                        Seed::Fresh(5),
                        Some(&session.kept()),
                    )
                    .unwrap();
                    let turns = copy.turns().len();
                    copy.tap_action(copy.stamp(), item.index, place)
                        .unwrap_or_else(|e| panic!("{name}: an offered action was refused: {e:?}"));
                    let still = each(&copy.inventory().items)
                        .into_iter()
                        .find(|i| i.name == item.name)
                        .cloned();
                    let now: Vec<String> = still
                        .as_ref()
                        .and_then(|i| copy.card(copy.stamp(), i.index).ok())
                        .map(|c| c.actions.into_iter().map(|a| a.label).collect())
                        .unwrap_or_default();
                    let has = |label: &str| now.iter().any(|l| l == label);
                    let said = copy.turns().len() == turns + 1
                        && !copy.turns().last().unwrap().text.trim().is_empty();
                    let did = match label.as_str() {
                        "Open" => has("Close"),
                        "Close" => has("Open"),
                        "Turn on" => has("Turn off"),
                        "Turn off" => has("Turn on"),
                        "Wear" => has("Take off"),
                        "Take off" => has("Wear"),
                        "Eat" | "Drink" | "Drop" => still.is_none(),
                        "Read" => said,
                        other => panic!("{name}: an action with no rule here: {other}"),
                    };
                    let count = counts.entry(label.clone()).or_default();
                    if did {
                        count.0 += 1;
                    } else {
                        count.1 += 1;
                        // By its place among the items and the turn, not by name.
                        differed.push(format!(
                            "{label} on item {} at turn {}",
                            item.index,
                            session.turns().len()
                        ));
                    }
                }
            }
        }
        let summary: Vec<String> = counts
            .iter()
            .map(|(label, (did, not))| format!("{label} {did}/{}", did + not))
            .collect();
        println!(
            "{name}: actions that did what they say: {}",
            summary.join(", ")
        );
        for case in &differed {
            println!("{name}: differed: {case}");
        }
        assert!(!counts.is_empty(), "{name}: no action was offered");
        // Every case that did not do what it says has its cause written here,
        // by the action, the item's place and the turn; any other fails.
        let known: &[(&str, &str, &str)] = match name {
            "zork1" => &[
                (
                    "Open on item 0 at turn 5",
                    "refused",
                    "The thing has the attribute of things that hold others, and its own \
                     routine refuses to open it: opening it is a puzzle.",
                ),
                (
                    "Open on item 0 at turn 120",
                    "ended",
                    "It opened. The copy's game then ended in the same turn, the player \
                     being where the walkthrough does something else at once.",
                ),
                (
                    "Open on item 0 at turn 170",
                    "refused",
                    "The thing has the attribute and its own routine answers that it is \
                     open already; the game does not keep it as open or closed.",
                ),
                (
                    "Open on item 0 at turn 304",
                    "refused",
                    "The thing has the attribute and cannot be closed or opened: it is a \
                     vessel with no lid.",
                ),
            ],
            _ => &[],
        };
        for case in &differed {
            assert!(
                known.iter().any(|(k, ..)| k == case),
                "{name}: {case} did not do what it says, and no cause is written"
            );
        }
        for (case, ..) in known {
            assert!(
                differed.contains(&case.to_string()),
                "{name}: {case} no longer differs"
            );
        }
    }
}
