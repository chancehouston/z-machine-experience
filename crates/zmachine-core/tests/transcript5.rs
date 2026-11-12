//! The transcript in versions 5 and 8 (feature 002, T029; §7.1.1.2, §7.1.2.2;
//! specs/002-v5-v8/contracts/transcript-format.md rules 13 to 21).

mod common;

use common::*;
use zmachine_core::{Event, Key, Machine};
use zmachine_testkit::programs::{TranscriptProbe5, transcript5_probes};

struct Run {
    transcript: String,
    screen: String,
    /// How many times the game asked for a transcript.
    requests: usize,
}

/// Runs an image to the end. A read takes the next input line; a key request takes
/// the first character of the next line, or return if the line is empty.
fn drive(image: Vec<u8>, input: &[&str]) -> Run {
    let mut m = Machine::load(image, OPTIONS).unwrap();
    let mut run = Run { transcript: String::new(), screen: String::new(), requests: 0 };
    let mut input = input.iter();
    loop {
        let event = m.run(BIG).unwrap();
        run.screen.push_str(&m.take_output());
        run.transcript.push_str(&m.take_transcript());
        match event {
            Event::Finished => return run,
            Event::TranscriptRequested => {
                run.requests += 1;
                m.transcript_result(true).unwrap();
            }
            Event::LineWanted { .. } => m.provide_line(input.next().expect("a line")).unwrap(),
            Event::KeyWanted => {
                let key = input.next().expect("a key").chars().next().map_or(Key::Return, Key::Char);
                m.provide_key(key).unwrap();
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/transcript5/{name}.scr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

// contracts/transcript-format.md rules 13 to 21: for each probe program the
// transcript equals, byte for byte, the one dfrotz 2.55 wrote
// (tests/fixtures/README.md).
#[test]
fn every_version_5_probe_transcript_matches_the_reference() {
    let probes = transcript5_probes();
    assert!(probes.len() >= 17);
    for TranscriptProbe5 { name, purpose, image, input, .. } in probes {
        let run = drive(image, input);
        assert_eq!(run.transcript, fixture(name), "{name}: {purpose}");
    }
}

// §7.1.1.2: "the interpreter should ask for a file name the first time" only.
// Frotz asks a version 5 game every time; this is the documented difference.
#[test]
fn the_transcript_file_name_is_asked_for_once() {
    for probe in transcript5_probes() {
        let run = drive(probe.image, probe.input);
        assert_eq!(run.requests, 1, "{}", probe.name);
        assert!(probe.starts >= 1);
    }
}

// Each rule from 13 to 21 has at least one probe that names it.
#[test]
fn every_new_rule_has_a_probe() {
    let probes = transcript5_probes();
    for rule in 13..=21 {
        let needle = format!("rule {rule}");
        assert!(probes.iter().any(|p| p.purpose.contains(&needle)), "no probe for {needle}");
    }
}

// Rule 16 seen from the other side: text printed with the screen stream off is in
// the transcript and not on the screen; rule 15: upper-window text is in neither.
#[test]
fn what_the_screen_shows_differs_from_the_transcript_as_the_rules_say() {
    let probes = transcript5_probes();
    let run_of = |name: &str| {
        let probe = probes.iter().find(|p| p.name == name).unwrap();
        drive(probe.image.clone(), probe.input)
    };
    let run = run_of("screen_off");
    assert_eq!(run.screen, "a b\n");
    assert_eq!(run.transcript, "a [quoted] b\n");
    let run = run_of("upper_window_mid_word");
    assert_eq!(run.screen, "pending done\n");
    let run = run_of("memory_stream");
    assert_eq!(run.screen, "before after\n");
    let run = run_of("quotation_box");
    assert!(!run.screen.contains("[ A quotation]"), "{}", run.screen);
    assert!(run.transcript.contains("[ A quotation]\n"));
    assert!(!run.transcript.contains(" A quotation \n"));
}
