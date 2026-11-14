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
