//! Copying a machine (feature 003, T006; specs/003-introspection/contracts/
//! rust-api.md): "`Clone` gives an independent copy of a machine in its present
//! state, waiting for the same answer."

mod common;

use common::*;
use zmachine_core::{Event, Key, LoadOptions, Machine};
use zmachine_testkit::Story5;
use zmachine_testkit::op::{self, Arg::*};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn story(relative: &str) -> Vec<u8> {
    std::fs::read(format!("{ROOT}/{relative}")).unwrap()
}

fn lines(relative: &str) -> Vec<String> {
    let text = std::fs::read_to_string(format!("{ROOT}/{relative}")).unwrap();
    text.lines().map(str::to_string).collect()
}

/// Everything a host can see of a machine after it next stops.
#[derive(Debug, PartialEq)]
struct Seen {
    event: Event,
    output: String,
    transcript: String,
}

/// Runs to the next stop that is not a budget stop.
fn step(m: &mut Machine) -> Seen {
    let event = run_until_event(m, BIG);
    Seen {
        event,
        output: m.take_output(),
        transcript: m.take_transcript(),
    }
}

/// Answers a stop from a script, as the command-line host does.
fn answer(m: &mut Machine, event: &Event, script: &mut impl Iterator<Item = String>) -> bool {
    match event {
        Event::LineWanted { .. } => match script.next() {
            Some(line) => m.provide_line(&line).unwrap(),
            None => return false,
        },
        Event::KeyWanted => {
            let key = script
                .next()
                .and_then(|l| l.chars().next())
                .map_or(Key::Return, Key::Char);
            m.provide_key(key).unwrap();
        }
        Event::TranscriptRequested => {
            script.next();
            m.transcript_result(true).unwrap();
        }
        Event::SaveRequested(_) => m.save_result(true).unwrap(),
        Event::RestoreRequested => m.provide_restore(None).unwrap(),
        _ => return false,
    }
    true
}

/// Plays `count` lines of a script, returning the machine at its next stop.
fn play(image: Vec<u8>, seed: u32, script: &[String], count: usize) -> (Machine, Seen) {
    let mut m = Machine::load(
        image,
        LoadOptions {
            seed: Some(seed),
            entropy: 0,
        },
    )
    .unwrap();
    let mut script = script.iter().take(count).cloned();
    loop {
        let seen = step(&mut m);
        if !answer(&mut m, &seen.event, &mut script) {
            return (m, seen);
        }
    }
}

fn memory(m: &Machine) -> Vec<u8> {
    let view = m.view();
    (0..view.len()).map(|i| view.byte(i).unwrap()).collect()
}

// rust-api.md: a clone is independent. Given a different command it diverges, and
// the original is exactly what it would have been had no clone been made.
#[test]
fn a_clone_is_independent_of_the_original() {
    let script = lines("stories/zork1/fidelity/walkthrough.txt");
    let image = story("stories/zork1/zork1.z3");
    let (mut original, _) = play(image.clone(), 11, &script, 40);
    let (mut untouched, _) = play(image, 11, &script, 40);
    let before = memory(&original);

    let mut copy = original.clone();
    assert_eq!(memory(&copy), before, "the copy starts in the same state");
    copy.provide_line("inventory").unwrap();
    let copied = step(&mut copy);
    assert!(
        copied.output.contains("You are carrying"),
        "{}",
        copied.output
    );
    assert_eq!(
        memory(&original),
        before,
        "playing the copy leaves the original alone"
    );

    // The original now takes its own next command, as does a machine never cloned.
    original.provide_line(&script[40]).unwrap();
    untouched.provide_line(&script[40]).unwrap();
    assert_eq!(step(&mut original), step(&mut untouched));
    assert_eq!(memory(&original), memory(&untouched));
    assert_ne!(memory(&original), memory(&copy));
}

// A clone made in the middle of a game, given the same input from there on, does
// exactly what the original does: output, transcript and save file.
#[test]
fn a_clone_continues_exactly_as_the_original_does() {
    for (file, walkthrough, seed, at) in [
        (
            "stories/zork1/zork1.z3",
            "stories/zork1/fidelity/walkthrough.txt",
            11,
            150,
        ),
        (
            "stories/advent/advent.z5",
            "stories/advent/fidelity/walkthrough.txt",
            1,
            80,
        ),
        (
            "stories/advent/advent.z8",
            "stories/advent/fidelity/walkthrough.txt",
            1,
            120,
        ),
    ] {
        let script = lines(walkthrough);
        let (mut original, _) = play(story(file), seed, &script, at);
        let mut copy = original.clone();
        // Thirty more commands, then a save, through both.
        let mut rest: Vec<String> = script[at..at + 30].to_vec();
        rest.push("save".to_string());
        let (mut a, mut b) = (rest.clone().into_iter(), rest.into_iter());
        loop {
            original.provide_line(&a.next().unwrap()).unwrap();
            copy.provide_line(&b.next().unwrap()).unwrap();
            let (first, second) = (step(&mut original), step(&mut copy));
            assert_eq!(first, second, "{file}");
            if let Event::SaveRequested(bytes) = &first.event {
                assert!(bytes.len() > 100, "{file}: a real save file");
                break;
            }
            assert!(
                matches!(first.event, Event::LineWanted { .. }),
                "{file}: {:?}",
                first.event
            );
        }
    }
}

// The random generator's state is copied: the copy's next results are the
// original's (data-model of feature 001: the generator is part of the machine).
#[test]
fn a_clone_has_the_same_random_sequence() {
    let mut story = Story5::new(5);
    let mut code = vec![read_char(g(0))];
    for _ in 0..12 {
        code.push(to(op::var(op::RANDOM, &[L(1000)]), g(1)));
        code.push(show(1));
    }
    code.push(quit());
    story.main(&cat(&code));
    // No seed: the sequence comes from the host's entropy, and must still be copied.
    let options = LoadOptions {
        seed: None,
        entropy: 0x5EED_1234,
    };
    let mut original = Machine::load(story.build(), options).unwrap();
    assert_eq!(run_until_event(&mut original, BIG), Event::KeyWanted);
    let mut copy = original.clone();
    original.provide_key(Key::Return).unwrap();
    copy.provide_key(Key::Return).unwrap();
    assert_eq!(run_until_event(&mut original, BIG), Event::Finished);
    assert_eq!(run_until_event(&mut copy, BIG), Event::Finished);
    let numbers = original.take_output();
    assert_eq!(numbers.split_whitespace().count(), 12);
    assert_eq!(copy.take_output(), numbers);
}

// Undo positions are copied: the copy can undo as far as the original can.
#[test]
fn a_clone_has_the_same_undo_history() {
    let mut story = Story5::new(5);
    story.main(&cat(&[
        to(op::ext(op::SAVE_UNDO, &[]), g(0)),
        show(0),
        skip_if(
            op::op2(op::JE, V(g(0)), S(2)),
            false,
            &cat(&[print("undone"), quit()]),
        ),
        read_char(g(1)),
        to(op::ext(op::RESTORE_UNDO, &[]), g(2)),
        print("nothing to undo"),
        quit(),
    ]));
    let mut original = Machine::load(story.build(), OPTIONS).unwrap();
    assert_eq!(run_until_event(&mut original, BIG), Event::KeyWanted);
    let mut copy = original.clone();
    for machine in [&mut original, &mut copy] {
        machine.take_output();
        machine.provide_key(Key::Return).unwrap();
        assert_eq!(run_until_event(machine, BIG), Event::Finished);
        assert_eq!(machine.take_output(), "2 undone\n");
    }
}

// A clone waits for the same answer as the original, whatever that is, and
// output not yet taken from the original is in the copy too.
#[test]
fn a_clone_waits_for_the_same_answer() {
    let script = lines("stories/advent/fidelity/walkthrough.txt");
    let (original, seen) = play(story("stories/advent/advent.z5"), 1, &script, 0);
    // The very first stop: the walkthrough's `script` has not been given yet.
    assert!(matches!(seen.event, Event::LineWanted { .. }));
    let mut copy = original.clone();
    assert_eq!(
        copy.provide_key(Key::Return),
        Err(zmachine_core::HostError::NotWaitingForThis)
    );
    copy.provide_line("script").unwrap();
    assert_eq!(run_until_event(&mut copy, BIG), Event::TranscriptRequested);
    let mut waiting = copy.clone();
    waiting.transcript_result(true).unwrap();
    assert!(matches!(
        run_until_event(&mut waiting, BIG),
        Event::LineWanted { .. }
    ));

    // After a fault, a clone has faulted too.
    let mut bad = Story5::new(5);
    bad.main(&cat(&[
        print("before "),
        to(op::op2(op::DIV, S(1), S(0)), g(0)),
        quit(),
    ]));
    let mut faulted = Machine::load(bad.build(), OPTIONS).unwrap();
    let event = run_until_event(&mut faulted, BIG);
    assert!(matches!(event, Event::Fault(_)));
    let mut copy = faulted.clone();
    assert_eq!(
        copy.take_output(),
        "before ",
        "output not yet taken is copied"
    );
    // A machine that has faulted has stopped for good, and so has its copy.
    assert_eq!(copy.run(BIG), faulted.run(BIG));
    assert_eq!(copy.run(BIG), Err(zmachine_core::HostError::Stopped));
}
