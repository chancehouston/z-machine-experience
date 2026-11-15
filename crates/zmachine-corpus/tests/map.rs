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

// specs/013-room-look, SC-002: where a story's map offers a touch on the
// player's place, the word it sends is a word of that story's dictionary; a
// story without the word offers no touch. Counts only are printed.
#[test]
fn the_look_sends_only_a_word_the_story_has() {
    use zmachine_introspect::Introspector;
    let root = repo("");
    let root = Path::new(&root);
    let (stories, _) = corpus::load(root).expect("the manifest and the ledger are read");
    let (mut with, mut without, mut unmapped) = (0, 0, 0);
    for story in &stories {
        let Some(bytes) = corpus::present(root, story) else {
            continue;
        };
        let Ok(session) = Session::with_pack(bytes.clone(), Seed::Fixed(7), None) else {
            continue;
        };
        let Some(view) = session.map().view() else {
            unmapped += 1;
            continue;
        };
        let Ok(machine) = zmachine_corpus::walk::load(bytes, 7) else {
            continue;
        };
        let words: Vec<String> = Introspector::new(&machine.view())
            .dictionary(&machine.view())
            .value
            .map(|words| words.into_iter().map(|word| word.word).collect())
            .unwrap_or_default();
        match view.look {
            Some(look) => {
                assert!(words.contains(&look.command), "{}", story.id);
                with += 1;
            }
            None => {
                assert!(
                    !words.iter().any(|word| word == "look" || word == "l"),
                    "{}: the word is there and no touch is offered",
                    story.id
                );
                without += 1;
            }
        }
    }
    println!("look: {with} stories with the word, {without} without, {unmapped} with no map");
    assert!(with >= 4, "{with} stories offered the look");
}

// specs/016-map-untangle, SC-005: in every recognised story, wandered, a
// redraw of the map wherever one is offered faults nowhere, sends nothing,
// and leaves no two places in one position. Counts only are printed.
#[test]
fn a_redraw_is_sound_in_every_story() {
    let root = repo("");
    let root = Path::new(&root);
    let (stories, _) = corpus::load(root).expect("the manifest and the ledger are read");
    let mut redrawn_somewhere = 0;
    for story in &stories {
        let Some(bytes) = corpus::present(root, story) else {
            continue;
        };
        let pack = std::fs::read_to_string(root.join(format!("packs/{}.pack", story.id))).ok();
        let Ok(mut session) = Session::with_pack(bytes, Seed::Fixed(7), pack.as_deref()) else {
            continue;
        };
        let mut state = 99u32;
        let (mut offered, mut made) = (0, 0);
        for step in 0..200 {
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
            if step % 7 != 0 || session.map().view().is_none_or(|v| v.redraw.is_none()) {
                continue;
            }
            offered += 1;
            let (turns, before) = (session.turns().len(), session.map().page());
            session
                .redraw_map(session.stamp())
                .unwrap_or_else(|e| panic!("{}: an offered redraw was refused: {e:?}", story.id));
            assert_eq!(session.turns().len(), turns, "{}", story.id);
            let page = session.map().page();
            made += usize::from(page != before);
            let mut seen = BTreeSet::new();
            for line in page.iter().filter(|l| l.starts_with("place")) {
                let words: Vec<&str> = line.split_whitespace().collect();
                assert!(
                    seen.insert(words[3..7].join(" ")),
                    "{}: two places in one position after a redraw",
                    story.id
                );
            }
            // The same count of places, each on the sheet it was on.
            assert_eq!(
                page.iter().filter(|l| l.starts_with("place")).count(),
                before.iter().filter(|l| l.starts_with("place")).count(),
                "{}",
                story.id
            );
        }
        if made > 0 {
            redrawn_somewhere += 1;
        }
        println!(
            "{}: a redraw offered {offered} times, moved places {made} times",
            story.id
        );
    }
    assert!(
        redrawn_somewhere >= 3,
        "{redrawn_somewhere} stories were redrawn"
    );
}
