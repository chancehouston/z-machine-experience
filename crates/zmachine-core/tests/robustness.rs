//! contracts/core-api.md guarantee 3: "No panics on any story file." A story file
//! is data from outside; whatever it contains, the core must answer with an error,
//! a fault or ordinary play, and never take the host down.
//!
//! These tests damage real story files in many ways and run the results. They
//! assert nothing about what a damaged file does, only that the core survives it:
//! a panic anywhere fails the test.

mod common;

use zmachine_core::{Event, Key, LoadOptions, Machine, Settings};
use zmachine_testkit::Story5;
use zmachine_testkit::op::{self, Arg::*};
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
    exercise_with(image, saved, &lines)
}

/// The same with lines that reach what versions 5 and 8 add: undo, praxix's
/// tests, and Advent's menus, which read single keys.
fn exercise5(image: Vec<u8>, saved: Option<&[u8]>) -> End {
    let lines = [
        "enter building",
        "take all",
        "undo",
        "all",
        "save",
        "restore",
        "help",
        "",
        "undo",
        "quit",
    ];
    exercise_with(image, saved, &lines)
}

fn exercise_with(image: Vec<u8>, saved: Option<&[u8]>, lines: &[&str]) -> End {
    let options = LoadOptions {
        seed: Some(3),
        entropy: 0,
    };
    let Ok(mut m) = Machine::load(image, options) else {
        return End::Refused;
    };
    let keys = [
        Key::Char('y'),
        Key::Return,
        Key::Char('n'),
        Key::Up,
        Key::Escape,
        Key::Char('q'),
    ];
    let mut last_save: Option<Vec<u8>> = None;
    for turn in 0..40 {
        let event = m.run(20_000).expect("the driver answers in turn");
        m.take_styled_output();
        m.take_screen_changed();
        let _ = m.screen().row_text(1);
        m.take_transcript();
        m.take_status();
        m.take_diagnostics();
        match event {
            Event::Finished => return End::Finished,
            Event::Fault(_) => return End::Fault,
            Event::BudgetExhausted => {}
            Event::LineWanted { .. } => m.provide_line(lines[turn % lines.len()]).unwrap(),
            Event::TranscriptRequested => m.transcript_result(turn % 2 == 0).unwrap(),
            Event::KeyWanted => m.provide_key(keys[turn % keys.len()]).unwrap(),
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

// ---- Versions 5 and 8 (feature 002, T057; 002 FR-024) ----

const ADVENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/advent");
const PRAXIX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../stories/praxix/praxix.z5"
);

fn stories5() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (
            "advent.z5",
            std::fs::read(format!("{ADVENT}/advent.z5")).unwrap(),
        ),
        (
            "advent.z8",
            std::fs::read(format!("{ADVENT}/advent.z8")).unwrap(),
        ),
        ("praxix.z5", std::fs::read(PRAXIX).unwrap()),
    ]
}

// The undamaged files themselves run under this driver without a fault, so the
// damage below is what causes whatever follows.
#[test]
fn the_undamaged_version_5_and_8_stories_run() {
    for (name, story) in stories5() {
        let end = exercise5(story, None);
        assert!(
            matches!(end, End::StillRunning | End::Finished),
            "{name}: {end:?}"
        );
    }
}

#[test]
fn truncated_version_5_and_8_story_files_never_panic() {
    for (_, story) in stories5() {
        let mut lengths: Vec<usize> = (0..=70).collect();
        lengths.extend((1..24).map(|i| story.len() * i / 24));
        lengths.extend([story.len() - 2, story.len() - 1]);
        for len in lengths {
            exercise5(story[..len.min(story.len())].to_vec(), None);
        }
    }
}

#[test]
fn damaged_version_5_and_8_headers_never_panic() {
    for (_, story) in stories5() {
        for offset in 0..64 {
            for value in [0x00, 0xFF] {
                let mut image = story.clone();
                image[offset] = value;
                exercise5(image, None);
            }
        }
    }
}

#[test]
fn randomly_damaged_version_5_and_8_story_files_never_panic() {
    let mut rng = Lcg(2027);
    let mut ends = std::collections::HashMap::new();
    for (_, story) in stories5() {
        // Damage early in the file reaches the tables; damage anywhere reaches code.
        for trial in 0..60 {
            let mut image = story.clone();
            let span = if trial % 2 == 0 {
                image.len()
            } else {
                0x4000.min(image.len())
            };
            for _ in 0..1 + rng.below(12) {
                let at = rng.below(span);
                image[at] = rng.next() as u8;
            }
            *ends.entry(exercise5(image, None)).or_insert(0) += 1;
        }
    }
    assert!(ends.get(&End::Fault).copied().unwrap_or(0) > 10, "{ends:?}");
    assert!(
        ends.get(&End::StillRunning).copied().unwrap_or(0) > 10,
        "{ends:?}"
    );
}

#[test]
fn noise_with_a_version_5_or_8_header_never_panics() {
    let mut rng = Lcg(6);
    for version in [5u8, 8] {
        for len in [64usize, 65, 200, 3000, 70_000, 200_000] {
            for _ in 0..8 {
                let mut image: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
                image[0] = version;
                exercise5(image, None);
            }
        }
        exercise5(vec![version; 64], None);
        exercise5(vec![version; 100_000], None);
    }
}

#[test]
fn damaged_advent_save_files_never_panic() {
    let mut rng = Lcg(12);
    for build in ["z5", "z8"] {
        let story = std::fs::read(format!("{ADVENT}/advent.{build}")).unwrap();
        // The driver's sixth line is `restore`, which is when the save is offered.
        let lines = ["restore", "look", "undo", "restore", "inventory"];
        for n in 1..=3 {
            let save =
                std::fs::read(format!("{ADVENT}/fidelity/saves/frotz_{build}_{n}.qzl")).unwrap();
            assert_eq!(
                exercise_with(story.clone(), Some(&save), &lines),
                End::StillRunning,
                "the intact save"
            );
            for len in (0..save.len()).step_by(97) {
                exercise_with(story.clone(), Some(&save[..len]), &lines);
            }
            for _ in 0..25 {
                let mut damaged = save.clone();
                for _ in 0..1 + rng.below(4) {
                    let at = rng.below(damaged.len());
                    damaged[at] = rng.next() as u8;
                }
                exercise_with(story.clone(), Some(&damaged), &lines);
            }
        }
    }
}

// contracts/core-api.md guarantee 7: no story can make the core allocate without
// limit. A story asks for an enormous upper window, draws far outside the screen,
// and saves undo positions for ever; the canvas stays the size the host set and
// the run simply uses up its budget.
#[test]
fn a_greedy_story_stays_within_bounds() {
    let mut story = Story5::new(5);
    let print = |s: &str| [op::op0(op::PRINT), zmachine_testkit::zencode(s)].concat();
    story.main(
        &[
            op::var(op::SPLIT_WINDOW, &[L(0xFFFF)]),
            op::var(op::SET_WINDOW, &[S(1)]),
            op::var(op::SET_CURSOR, &[L(60_000), L(60_000)]),
            print("far away"),
            op::var(op::ERASE_WINDOW, &[L(0x7FFF)]),
            // save_undo for ever: four bytes, then a jump back over both.
            [op::ext(op::SAVE_UNDO, &[]), op::store(16)].concat(),
            print("x"),
            op::op1(op::JUMP, L(0xFFF8)),
        ]
        .concat(),
    );
    let settings = Settings {
        screen_columns: 12,
        screen_rows: 3,
        undo_slots: 4,
        ..Settings::default()
    };
    let options = LoadOptions {
        seed: Some(1),
        entropy: 0,
    };
    let mut m = Machine::load_with(story.build(), options, settings).unwrap();
    for _ in 0..5 {
        assert_eq!(m.run(50_000), Ok(Event::BudgetExhausted));
    }
    assert_eq!((m.screen().width(), m.screen().rows()), (12, 3));
    assert_eq!(m.screen().upper_height(), 3);
    assert!(
        m.screen().cell(3, 12).is_some(),
        "the far corner was written, nothing beyond it"
    );
}

// Seventeen memory streams is a fault, however many the story then tries.
#[test]
fn a_deep_memory_stream_nest_is_a_fault_not_a_crash() {
    let mut story = Story5::new(5);
    let table = story.reserve(4);
    story.main(
        &[
            op::var(op::OUTPUT_STREAM, &[S(3), L(table)]),
            op::op1(op::JUMP, L(0xFFFA)),
        ]
        .concat(),
    );
    let options = LoadOptions {
        seed: Some(1),
        entropy: 0,
    };
    let mut m = Machine::load(story.build(), options).unwrap();
    assert!(matches!(m.run(10_000), Ok(Event::Fault(_))));
}
