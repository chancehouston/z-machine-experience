//! contracts/core-api.md guarantee 3: "No panics on any story file." A story file
//! is data from outside; whatever it contains, the core must answer with an error,
//! a fault or ordinary play, and never take the host down.
//!
//! These tests damage real story files in many ways and run the results. They
//! assert nothing about what a damaged file does, only that the core survives it:
//! a panic anywhere fails the test.

mod common;

use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_testkit::programs::transcript_probes;

const ZORK1: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");
const CZECH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/czech/czech.z3");

/// A small deterministic generator, so that every run tests the same damage.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// How a run of a damaged file ended.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum End {
    Refused,
    Fault,
    Finished,
    StillRunning,
}

/// Loads and runs an image for a bounded amount of work, answering every event.
fn exercise(image: Vec<u8>, saved: Option<&[u8]>) -> End {
    let options = LoadOptions {
        seed: Some(3),
        entropy: 0,
    };
    let Ok(mut m) = Machine::load(image, options) else {
        return End::Refused;
    };
    let lines = [
        "look",
        "north",
        "take all",
        "",
        "inventory",
        "open mailbox",
        "save",
        "restore",
    ];
    let mut last_save: Option<Vec<u8>> = None;
    for turn in 0..40 {
        let event = m.run(20_000).expect("the driver answers in turn");
        m.take_output();
        m.take_transcript();
        m.take_status();
        m.take_diagnostics();
        match event {
            Event::Finished => return End::Finished,
            Event::Fault(_) => return End::Fault,
            Event::BudgetExhausted => {}
            Event::LineWanted { .. } => m.provide_line(lines[turn % lines.len()]).unwrap(),
            Event::TranscriptRequested => m.transcript_result(turn % 2 == 0).unwrap(),
            Event::SaveRequested(bytes) => {
                last_save = Some(bytes);
                m.save_result(true).unwrap();
            }
            Event::RestoreRequested => {
                let offered = saved.or(last_save.as_deref());
                m.provide_restore(offered).unwrap();
            }
        }
    }
    End::StillRunning
}

fn stories() -> Vec<(&'static str, Vec<u8>)> {
    let mut all = vec![
        ("zork1", std::fs::read(ZORK1).unwrap()),
        ("czech", std::fs::read(CZECH).unwrap()),
    ];
    all.extend(
        transcript_probes()
            .into_iter()
            .take(3)
            .map(|p| (p.name, p.image)),
    );
    all
}

// Every prefix length that matters: a file cut anywhere must not crash the loader
// or the interpreter that then runs off its end.
#[test]
fn truncated_story_files_never_panic() {
    for (name, story) in stories() {
        let mut lengths: Vec<usize> = (0..=70).collect();
        lengths.extend((1..40).map(|i| story.len() * i / 40));
        lengths.extend([story.len() - 2, story.len() - 1]);
        for len in lengths {
            let len = len.min(story.len());
            let _ = (name, exercise(story[..len].to_vec(), None));
        }
    }
}

// Every header byte set to each of several values, one at a time.
#[test]
fn damaged_headers_never_panic() {
    for (_, story) in stories() {
        for offset in 0..64 {
            for value in [0x00, 0x7F, 0xFF] {
                let mut image = story.clone();
                image[offset] = value;
                exercise(image, None);
            }
        }
    }
}

// Random damage anywhere in the file: a few bytes changed per trial.
#[test]
fn randomly_damaged_story_files_never_panic() {
    let mut rng = Lcg(2026);
    let mut ends = std::collections::HashMap::new();
    for (_, story) in stories() {
        for _ in 0..50 {
            let mut image = story.clone();
            for _ in 0..1 + rng.below(6) {
                let at = rng.below(image.len());
                image[at] = rng.next() as u8;
            }
            *ends.entry(exercise(image, None)).or_insert(0) += 1;
        }
    }
    // The damage must actually have reached interesting code: some runs fault and
    // some carry on.
    assert!(ends.get(&End::Fault).copied().unwrap_or(0) > 20, "{ends:?}");
    assert!(
        ends.get(&End::StillRunning).copied().unwrap_or(0) > 20,
        "{ends:?}"
    );
}

// Damage concentrated where the interpreter's data structures live: the object
// table, the dictionary, the globals and the abbreviations.
#[test]
fn damaged_tables_never_panic() {
    let mut rng = Lcg(77);
    let story = std::fs::read(ZORK1).unwrap();
    let word = |at: usize| u16::from_be_bytes([story[at], story[at + 1]]) as usize;
    let tables = [
        (word(0x0A), 62 + 9 * 255),
        (word(0x08), 700 * 7),
        (word(0x0C), 480),
        (word(0x18), 192),
    ];
    for (start, len) in tables {
        for _ in 0..100 {
            let mut image = story.clone();
            for _ in 0..1 + rng.below(8) {
                let at = start + rng.below(len);
                if at < image.len() {
                    image[at] = rng.next() as u8;
                }
            }
            exercise(image, None);
        }
    }
}

// A story file of pure noise, with only the version byte right.
#[test]
fn noise_with_a_version_3_header_never_panics() {
    let mut rng = Lcg(5);
    for len in [64usize, 65, 100, 1000, 20_000, 70_000, 130_000] {
        for _ in 0..12 {
            let mut image: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
            image[0] = 3;
            exercise(image, None);
        }
    }
    exercise(vec![3; 64], None);
    exercise(vec![3; 100_000], None);
    exercise(vec![0xFF; 100_000], None);
}

// Save files are data from outside too. A damaged save must be refused or
// restored, never crash, whatever the game then does with it.
#[test]
fn damaged_save_files_never_panic() {
    let story = std::fs::read(ZORK1).unwrap();
    let root = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../stories/zork1/fidelity/saves"
    );
    let mut rng = Lcg(11);
    for n in 1..=5 {
        let save = std::fs::read(format!("{root}/frotz_{n}.qzl")).unwrap();
        for len in (0..save.len()).step_by(23) {
            exercise(story.clone(), Some(&save[..len]));
        }
        for _ in 0..50 {
            let mut damaged = save.clone();
            for _ in 0..1 + rng.below(4) {
                let at = rng.below(damaged.len());
                damaged[at] = rng.next() as u8;
            }
            exercise(story.clone(), Some(&damaged));
        }
    }
}
