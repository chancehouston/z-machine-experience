//! SC-010: asking all six queries at a prompt takes less time than a person
//! would notice. Timing is only meaningful in an optimized build.

mod common;

use common::*;
use std::time::{Duration, Instant};
use zmachine_introspect::Introspector;

// The limit is the specification's five milliseconds. QUERY_LIMIT_MILLIS gives
// another, for a machine that is not the measure of the interface: a shared CI
// runner once took 7.1 ms over one prompt, with a mean of 0.53 ms.
#[test]
#[ignore = "timing; run with --release"]
fn all_six_queries_take_under_five_milliseconds() {
    let limit = std::env::var("QUERY_LIMIT_MILLIS")
        .ok()
        .and_then(|millis| millis.parse().ok())
        .unwrap_or(5);
    for game in [ZORK1, ADVENT5, ADVENT8] {
        let machine = load(game.story, game.seed);
        // With the story's game pack, where the repository holds one (feature
        // 006, SC-011): preparing then includes working out the story's digest.
        let pack = std::fs::read_to_string(format!("{ROOT}/packs/{}.pack", game.name)).ok();
        let started = Instant::now();
        let prepared = match &pack {
            Some(text) => Introspector::with_pack(&machine.view(), text),
            None => Introspector::new(&machine.view()),
        };
        let preparing = started.elapsed();
        if game.name == ZORK1.name {
            assert_eq!(prepared.pack(), zmachine_introspect::PackState::InForce);
        }
        assert!(
            preparing < Duration::from_millis(50),
            "{}: preparing took {preparing:?}",
            game.name
        );

        let (mut slowest, mut total, mut prompts) = (Duration::ZERO, Duration::ZERO, 0u32);
        walk(game, |machine, stop| {
            if !stop.command_prompt {
                return;
            }
            let view = machine.view();
            let started = Instant::now();
            let answers = (
                prepared.object_tree(&view),
                prepared.dictionary(&view),
                prepared.location(&view),
                prepared.inventory(&view),
                prepared.exits(&view),
                prepared.scope(&view),
            );
            let taken = started.elapsed();
            std::hint::black_box(answers);
            slowest = slowest.max(taken);
            total += taken;
            prompts += 1;
        });
        println!(
            "{}: slowest {slowest:?}, mean {:?} over {prompts} prompts; preparing {preparing:?}",
            game.name,
            total / prompts
        );
        assert!(
            slowest < Duration::from_millis(limit),
            "{}: the slowest prompt took {slowest:?}",
            game.name
        );
    }
}
