//! SC-010: asking all six queries at a prompt takes less time than a person
//! would notice. Timing is only meaningful in an optimized build.

mod common;

use common::*;
use std::time::{Duration, Instant};
use zmachine_introspect::Introspector;

#[test]
#[ignore = "timing; run with --release"]
fn all_six_queries_take_under_five_milliseconds() {
    for game in [ZORK1, ADVENT5, ADVENT8] {
        let machine = load(game.story, game.seed);
        let started = Instant::now();
        let prepared = Introspector::new(&machine.view());
        let preparing = started.elapsed();
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
            slowest < Duration::from_millis(5),
            "{}: the slowest prompt took {slowest:?}",
            game.name
        );
    }
}
