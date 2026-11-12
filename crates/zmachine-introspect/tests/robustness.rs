//! No story file can make a query fail (docs/introspection.md section 1.6, cited
//! I§1.6; SC-009; rust-api.md guarantees 2, 3 and 5). Damaged copies of the
//! committed story files are loaded, run, and asked everything; a panic, or a
//! query that does not return, fails the test. Nothing is asserted about what a
//! damaged file answers, only that it answers.

mod common;

use common::*;
use zmachine_core::{Event, Key, LoadOptions, Machine};
use zmachine_introspect::{Family, Introspector, Unknown};

const STORIES: [&str; 5] = [
    "stories/zork1/zork1.z3",
    "stories/advent/advent.z5",
    "stories/advent/advent.z8",
    "stories/czech/czech.z5",
    "stories/praxix/praxix.z5",
];

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

fn ask_everything(machine: &Machine, introspector: &Introspector) -> usize {
    let view = machine.view();
    let mut answered = 0;
    answered += introspector.object_tree(&view).value.map_or(0, |v| v.len());
    answered += introspector.dictionary(&view).value.map_or(0, |v| v.len());
    answered += introspector.location(&view).value.is_ok() as usize;
    answered += introspector.inventory(&view).value.map_or(0, |v| v.len());
    answered += introspector
        .exits(&view)
        .value
        .map_or(0, |v| v.exits.len() + v.unknown.len());
    answered += introspector
        .scope(&view)
        .value
        .map_or(0, |v| v.objects.len());
    answered
}

/// How a damaged file fared.
#[derive(Default, Debug)]
struct Tally {
    files: usize,
    loaded: usize,
    recognised: usize,
    stops: usize,
    answered: usize,
}

/// Loads an image if the core will have it, and asks everything at load and at
/// up to `stops` stops of a short game.
fn exercise(image: Vec<u8>, stops: usize, tally: &mut Tally) {
    tally.files += 1;
    let Ok(mut machine) = Machine::load(
        image,
        LoadOptions {
            seed: Some(3),
            entropy: 0,
        },
    ) else {
        return;
    };
    tally.loaded += 1;
    let introspector = Introspector::new(&machine.view());
    tally.recognised += (introspector.family() != Family::Unrecognised) as usize;
    tally.answered += ask_everything(&machine, &introspector);
    let lines = [
        "north",
        "take all",
        "inventory",
        "in",
        "open mailbox",
        "south",
        "look",
        "down",
    ];
    for turn in 0..stops {
        let event = machine.run(20_000).expect("the driver answers in turn");
        machine.take_output();
        tally.stops += 1;
        tally.answered += ask_everything(&machine, &introspector);
        match event {
            Event::Finished | Event::Fault(_) => return,
            Event::BudgetExhausted => {}
            Event::LineWanted { .. } => machine.provide_line(lines[turn % lines.len()]).unwrap(),
            Event::KeyWanted => machine.provide_key(Key::Return).unwrap(),
            Event::TranscriptRequested => machine.transcript_result(false).unwrap(),
            Event::SaveRequested(_) => machine.save_result(false).unwrap(),
            Event::RestoreRequested => machine.provide_restore(None).unwrap(),
        }
    }
}

// The undamaged files, driven the same way, are answered at every stop.
#[test]
fn the_undamaged_files_are_answered() {
    for story in STORIES {
        let mut tally = Tally::default();
        exercise(read(story), 20, &mut tally);
        assert_eq!(tally.loaded, 1, "{story}");
        assert!(
            tally.stops >= 1 && tally.answered >= 20,
            "{story}: {tally:?}"
        );
    }
}

// I§1.6: a story file cut short anywhere.
#[test]
fn truncated_story_files_are_answered_or_unknown() {
    let mut tally = Tally::default();
    for story in STORIES {
        let image = read(story);
        let mut lengths: Vec<usize> = (0..=70).collect();
        lengths.extend((1..30).map(|i| image.len() * i / 30));
        lengths.extend([image.len() - 2, image.len() - 1]);
        for len in lengths {
            exercise(image[..len.min(image.len())].to_vec(), 3, &mut tally);
        }
    }
    assert!(tally.files >= 500 && tally.loaded >= 50, "{tally:?}");
}

// I§1.6: every header byte set to each of two values, one at a time.
#[test]
fn damaged_headers_are_answered_or_unknown() {
    let mut tally = Tally::default();
    for story in STORIES {
        let image = read(story);
        for offset in 0..64 {
            for value in [0x00, 0xFF] {
                let mut damaged = image.clone();
                damaged[offset] = value;
                exercise(damaged, 3, &mut tally);
            }
        }
    }
    assert!(tally.files >= 640 && tally.loaded >= 200, "{tally:?}");
}

// I§1.6: random bytes changed anywhere in the file, and concentrated in the first
// part of it, where the object table, globals and dictionary are.
#[test]
fn randomly_damaged_story_files_are_answered_or_unknown() {
    let mut rng = Lcg(2028);
    let mut tally = Tally::default();
    for story in STORIES {
        let image = read(story);
        for trial in 0..260 {
            let mut damaged = image.clone();
            let span = match trial % 3 {
                0 => damaged.len(),
                1 => 0x5000.min(damaged.len()),
                _ => 0x1800.min(damaged.len()),
            };
            for _ in 0..1 + rng.below(24) {
                let at = rng.below(span);
                damaged[at] = rng.next() as u8;
            }
            exercise(damaged, 6, &mut tally);
        }
    }
    assert!(tally.files >= 1300, "{tally:?}");
    // The damage reaches the queries: many damaged files are still recognised as
    // their family and go on to be asked about a game whose tables are wrong.
    assert!(
        tally.loaded >= 600 && tally.recognised >= 100 && tally.stops >= 1500,
        "{tally:?}"
    );
}

// I§1.6: damage aimed at the tables the queries read: every link of the object
// tree, property tables, the dictionary's header and the globals.
#[test]
fn damaged_tables_are_answered_or_unknown() {
    let mut rng = Lcg(91);
    let mut tally = Tally::default();
    for story in [
        "stories/zork1/zork1.z3",
        "stories/advent/advent.z5",
        "stories/advent/advent.z8",
    ] {
        let image = read(story);
        let word = |at: usize| u16::from_be_bytes([image[at], image[at + 1]]) as usize;
        let (objects, dictionary, globals) = (word(0x0A), word(0x08), word(0x0C));
        let regions = [
            (objects, 4000),
            (dictionary, 64),
            (globals, 480),
            (objects + 4000, 6000),
        ];
        for (start, len) in regions {
            for _ in 0..70 {
                let mut damaged = image.clone();
                for _ in 0..1 + rng.below(10) {
                    let at = start + rng.below(len);
                    if at < damaged.len() {
                        damaged[at] = rng.next() as u8;
                    }
                }
                exercise(damaged, 6, &mut tally);
            }
        }
    }
    assert!(tally.files >= 800 && tally.recognised >= 100, "{tally:?}");
}

// SC-009: the tests above ask at least 3,000 damaged files between them.
#[test]
fn at_least_three_thousand_damaged_files_are_asked() {
    let truncated = STORIES.len() * (71 + 29 + 2);
    let headers = STORIES.len() * 64 * 2;
    let random = STORIES.len() * 260;
    let tables = 3 * 4 * 70;
    assert!(
        truncated + headers + random + tables >= 3000,
        "{}",
        truncated + headers + random + tables
    );
}

// I§1.6: noise with a valid version byte.
#[test]
fn noise_is_answered_or_unknown() {
    let mut rng = Lcg(7);
    let mut tally = Tally::default();
    for version in [3u8, 5, 8] {
        for len in [64usize, 100, 3000, 70_000] {
            for _ in 0..10 {
                let mut image: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
                image[0] = version;
                exercise(image, 2, &mut tally);
            }
        }
    }
    assert_eq!(tally.files, 120);
}

// rust-api.md guarantee 5: an introspector made for one story file, given a view
// of another, answers `inconsistent` to every family-dependent query.
#[test]
fn an_introspector_knows_its_own_story_file() {
    let zork = load("stories/zork1/zork1.z3", 1);
    let advent = load("stories/advent/advent.z5", 1);
    let for_zork = Introspector::new(&zork.view());
    let other = advent.view();
    assert_eq!(
        for_zork.location(&other).value.err(),
        Some(Unknown::Inconsistent)
    );
    assert_eq!(
        for_zork.inventory(&other).value.err(),
        Some(Unknown::Inconsistent)
    );
    assert_eq!(
        for_zork.exits(&other).value.err(),
        Some(Unknown::Inconsistent)
    );
    assert_eq!(
        for_zork.scope(&other).value.err(),
        Some(Unknown::Inconsistent)
    );
    // The machine-level queries must not panic on it either.
    let _ = for_zork.object_tree(&other);
    let _ = for_zork.dictionary(&other);
    // After a restart or a restore the same introspector is still good: the
    // story file is the same one.
    let again = load("stories/zork1/zork1.z3", 99);
    assert!(
        for_zork.location(&again.view()).value.is_err(),
        "not yet started: no player"
    );
    assert_ne!(
        for_zork.location(&again.view()).value.err(),
        Some(Unknown::Inconsistent)
    );
}
