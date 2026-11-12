//! Prints all six answers, and the family, at one moment of a scripted game, as
//! one JSON object. For comparing this implementation with another reading of
//! `docs/introspection.md`.
//!
//! ```text
//! cargo run -p zmachine-introspect --example answers -- STORY SCRIPT SEED LINE
//! ```
//!
//! The moment is the prompt at which line LINE of SCRIPT (from 1) would be given.
//! Lines 1 and 2 of a walkthrough start a transcript and are skipped.

use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_introspect::Introspector;
use zmachine_introspect::json::to_json;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [story, script, seed, line] = args.as_slice() else {
        eprintln!("usage: answers STORY SCRIPT SEED LINE");
        std::process::exit(2);
    };
    let story = std::fs::read(story).expect("read the story file");
    let script = std::fs::read_to_string(script).expect("read the script");
    let line: usize = line.parse().expect("a line number");
    let options = LoadOptions {
        seed: Some(seed.parse().expect("a seed")),
        entropy: 0,
    };
    let mut machine = Machine::load(story, options).expect("load the story file");
    let mut commands = script.lines().skip(2).take(line.saturating_sub(3));
    loop {
        match machine.run(1_000_000).expect("run") {
            Event::BudgetExhausted => {}
            Event::LineWanted { .. } => match commands.next() {
                Some(command) => machine.provide_line(command).expect("a line was wanted"),
                None => break,
            },
            other => panic!("the script met {other:?}"),
        }
    }
    let view = machine.view();
    let i = Introspector::new(&view);
    println!(
        r#"{{"family":{},"location":{},"exits":{},"inventory":{},"scope":{},"object_tree":{},"dictionary":{}}}"#,
        to_json(&i.family()),
        to_json(&i.location(&view)),
        to_json(&i.exits(&view)),
        to_json(&i.inventory(&view)),
        to_json(&i.scope(&view)),
        to_json(&i.object_tree(&view)),
        to_json(&i.dictionary(&view)),
    );
}
