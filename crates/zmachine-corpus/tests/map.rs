//! The map across the corpus (specs/012-map-first-slice, FR-044): every story
//! the repository can read is wandered with a session keeping its map, and
//! the map obeys its rules. Nothing of any story's text is printed or kept:
//! counts only.

mod common;

use common::repo;
use std::collections::BTreeSet;
use std::path::Path;
use zmachine_corpus::corpus;
use zmachine_experience::{Seed, Session};

#[test]
fn every_story_can_be_wandered_with_a_sound_map() {
    let root = repo("");
    let root = Path::new(&root);
    let (stories, _) = corpus::load(root).expect("the manifest and the ledger are read");
    let mut wandered = 0;
    let mut mapped = 0;
    for story in &stories {
        let Some(bytes) = corpus::present(root, story) else {
            continue;
        };
        let pack = std::fs::read_to_string(root.join(format!("packs/{}.pack", story.id))).ok();
        let Ok(mut session) = Session::with_pack(bytes, Seed::Fixed(7), pack.as_deref()) else {
            continue;
        };
        let mut state = 99u32;
        let mut moves = 0;
        for _ in 0..300 {
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
            let line = match ways.len() {
                0 => "look".to_string(),
                n => ways[(state >> 16) as usize % n].clone(),
            };
            if session.send_line(session.stamp(), &line).is_err() {
                break;
            }
            moves += 1;
        }
        wandered += 1;
        let map = session.map();
        let Some(view) = map.view() else {
            // A story of neither family: no map, and nothing on its page.
            assert!(map.page().is_empty(), "{}", story.id);
            println!("{}: no map; {moves} moves", story.id);
            continue;
        };
        mapped += 1;
        // P§7: no two places in one position, on either grid of any sheet.
        let page = map.page();
        let mut seen = BTreeSet::new();
        let mut places = 0;
        for line in page.iter().filter(|l| l.starts_with("place")) {
            let words: Vec<&str> = line.split_whitespace().collect();
            assert!(
                // The sheet, which grid, and the position.
                seen.insert(words[3..7].join(" ")),
                "{}: two places in one position",
                story.id
            );
            places += 1;
        }
        // Every line joins two places that are on the map.
        let measures = map.measures();
        assert_eq!(
            measures.iter().map(|m| m.0).sum::<usize>(),
            places,
            "{}",
            story.id
        );
        println!(
            "{}: {moves} moves, {places} places on {} sheets, sheet {} shown, {} crossings",
            story.id,
            measures.len(),
            view.sheet,
            map.crossings()
        );
    }
    assert!(wandered >= 5, "{wandered} stories were found to wander");
    assert!(mapped >= 4, "{mapped} stories gave a map");
}
