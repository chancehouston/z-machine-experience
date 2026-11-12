//! The random number generator reproduces Frotz 2.55's (research.md R2), so that a
//! game with random events takes the same course in both interpreters (FR-012).

mod common;

use common::*;
use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_testkit::Story;
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::programs::random_probe;

fn run_to_end(mut m: Machine) -> String {
    let mut out = String::new();
    loop {
        let event = m.run(BIG).unwrap();
        out.push_str(&m.take_output());
        match event {
            Event::Finished => return out,
            // The probe turns transcription on; these tests read the screen text.
            Event::TranscriptRequested => m.transcript_result(false).unwrap(),
            other => panic!("unexpected {other:?}"),
        }
    }
}

fn probe(options: LoadOptions) -> String {
    run_to_end(Machine::load(random_probe(), options).unwrap())
}

fn fixture(seed: u32) -> String {
    let path = format!(
        "{}/tests/fixtures/random/seed_{seed}.txt",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(path).unwrap()
}

// §2.4, §15 random leave the generator to the interpreter. The fixtures are what
// dfrotz 2.55 printed for the same program: 200 results of `random 100` from the
// host seed, then 20 after each of `random -5`, `random -1000` and `random 0`.
#[test]
fn the_sequence_matches_frotz_for_every_fixture_seed() {
    for seed in [0u32, 1, 999, 1000, 123456, 2147483647] {
        let got = probe(LoadOptions {
            seed: Some(seed),
            entropy: 0,
        });
        assert_eq!(got, fixture(seed), "seed {seed}");
    }
}

/// Prints `count` results of `random range`, space separated, after running `setup`.
fn sample(setup: Vec<u8>, range: u16, count: u8, options: LoadOptions) -> Vec<i32> {
    let body = cat(&[
        cat(&[op::var(op::RANDOM, &[L(range)]), op::store(0)]),
        print_num(V(0)),
        print(" "),
        op::op2(op::INC_CHK, S(g(9)), S(count - 1)),
    ]);
    let back = -(body.len() as i16 + 2) + 2;
    let mut story = Story::new();
    story.main(&cat(&[setup, body, op::branch_long(false, back), quit()]));
    let out = run_to_end(Machine::load(story.build(), options).unwrap());
    out.split_whitespace().map(|n| n.parse().unwrap()).collect()
}

/// `random value -> (stack)`, then print the result and a space.
fn seed_with(value: i16) -> Vec<u8> {
    cat(&[
        op::var(op::RANDOM, &[L(value as u16)]),
        op::store(0),
        print_num(V(0)),
        print(" "),
    ])
}

const SEEDED: LoadOptions = LoadOptions {
    seed: Some(4242),
    entropy: 0,
};

// §15 random: "If range is positive, returns a uniformly random number between 1
// and range."
#[test]
fn results_lie_between_1_and_the_range() {
    for range in [1u16, 2, 6, 100, 32767] {
        let got = sample(Vec::new(), range, 200, SEEDED);
        assert_eq!(got.len(), 200);
        assert!(
            got.iter().all(|&n| n >= 1 && n <= range as i32),
            "range {range}"
        );
    }
    assert!(sample(Vec::new(), 1, 50, SEEDED).iter().all(|&n| n == 1));
    let spread: std::collections::HashSet<_> =
        sample(Vec::new(), 6, 200, SEEDED).into_iter().collect();
    assert_eq!(
        spread.len(),
        6,
        "every face of a die turns up in 200 throws"
    );
}

// §15 random: a negative or zero range seeds the generator and "returns 0".
// research.md R2: magnitudes 1 to 999 give the counting sequence 1, 2, ... wrapping
// at the magnitude.
#[test]
fn seeding_with_a_small_negative_gives_the_counting_sequence() {
    let got = sample(seed_with(-5), 100, 12, SEEDED);
    assert_eq!(got, vec![0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2]);
    // The result is still reduced into the requested range.
    let got = sample(seed_with(-5), 3, 7, SEEDED);
    assert_eq!(got, vec![0, 1, 2, 3, 1, 2, 1, 2]);
    let got = sample(seed_with(-999), 2000, 3, SEEDED);
    assert_eq!(got, vec![0, 1, 2, 3]);
}

// research.md R2: a magnitude of 1000 or more seeds the generator itself, so the
// same value gives the same sequence whatever the host seed was.
#[test]
fn seeding_with_a_large_negative_fixes_the_sequence() {
    let a = sample(seed_with(-1000), 100, 30, SEEDED);
    let b = sample(
        seed_with(-1000),
        100,
        30,
        LoadOptions {
            seed: Some(7),
            entropy: 0,
        },
    );
    assert_eq!(a, b);
    assert_eq!(a[0], 0);
    let c = sample(seed_with(-1001), 100, 30, SEEDED);
    assert_ne!(a, c);
}

// -32768 has no positive counterpart in 16 bits; its magnitude is 32768.
#[test]
fn seeding_with_the_most_negative_value_does_not_overflow() {
    let a = sample(seed_with(-32768), 100, 20, SEEDED);
    assert_eq!(a[0], 0);
    assert!(a[1..].iter().all(|&n| (1..=100).contains(&n)));
    assert_eq!(
        a,
        sample(
            seed_with(-32768),
            100,
            20,
            LoadOptions {
                seed: Some(1),
                entropy: 0
            }
        )
    );
}

// §15 random with 0 re-seeds "randomly"; research.md R2: from the host seed again,
// so the sequence starts over.
#[test]
fn seeding_with_zero_returns_to_the_host_seed() {
    let from_start = sample(Vec::new(), 100, 20, SEEDED);
    let setup = cat(&[seed_with(-7), seed_with(0)]);
    let after = sample(setup, 100, 20, SEEDED);
    assert_eq!(after[..2], [0, 0]);
    assert_eq!(after[2..], from_start[..]);
}

// FR-011: the same seed gives the same sequence on every run; another seed differs.
#[test]
fn the_seed_determines_the_sequence() {
    let a = sample(Vec::new(), 1000, 50, SEEDED);
    assert_eq!(a, sample(Vec::new(), 1000, 50, SEEDED));
    assert_ne!(
        a,
        sample(
            Vec::new(),
            1000,
            50,
            LoadOptions {
                seed: Some(4243),
                entropy: 0
            }
        )
    );
}

// A fixed seed makes the entropy value irrelevant.
#[test]
fn entropy_is_ignored_when_a_seed_is_given() {
    let a = sample(
        Vec::new(),
        1000,
        50,
        LoadOptions {
            seed: Some(5),
            entropy: 1,
        },
    );
    let b = sample(
        Vec::new(),
        1000,
        50,
        LoadOptions {
            seed: Some(5),
            entropy: 2,
        },
    );
    assert_eq!(a, b);
}

// FR-014: without a seed, the host's entropy decides the sequence.
#[test]
fn without_a_seed_the_entropy_decides() {
    let a = sample(
        Vec::new(),
        1000,
        50,
        LoadOptions {
            seed: None,
            entropy: 1,
        },
    );
    let b = sample(
        Vec::new(),
        1000,
        50,
        LoadOptions {
            seed: None,
            entropy: 2,
        },
    );
    assert_ne!(a, b);
    assert_eq!(
        a,
        sample(
            Vec::new(),
            1000,
            50,
            LoadOptions {
                seed: None,
                entropy: 1
            }
        )
    );
}

// research.md R2: "At start and on every restart the generator is seeded" from the
// host seed, so a restarted game sees the same numbers again.
#[test]
fn restart_reseeds_from_the_host_seed() {
    // Print five numbers. On the first pass (Flags 2 bit 1 clear) set the bit and
    // restart; on the second, quit.
    let five = {
        let body = cat(&[
            cat(&[op::var(op::RANDOM, &[S(100)]), op::store(0)]),
            print_num(V(0)),
            print(" "),
            op::op2(op::INC_CHK, S(g(9)), S(4)),
        ]);
        let back = -(body.len() as i16 + 2) + 2;
        cat(&[body, op::branch_long(false, back)])
    };
    let first_pass = cat(&[
        op::var(op::STOREB, &[S(0), S(0x11), S(2)]),
        op::op0(op::RESTART),
    ]);
    let mut story = Story::new();
    story.main(&cat(&[
        five,
        cat(&[op::op2(op::LOADB, S(0), S(0x11)), op::store(0)]),
        skip_if(op::op2(op::TEST, V(0), S(2)), true, &first_pass),
        quit(),
    ]));
    let out = run_to_end(Machine::load(story.build(), SEEDED).unwrap());
    let numbers: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(numbers.len(), 10);
    assert_eq!(numbers[..5], numbers[5..]);
}

// research.md R2: "Restore does not touch the generator." A restore the host
// declines must leave the sequence exactly where it was.
#[test]
fn a_declined_restore_leaves_the_generator_alone() {
    let plain = sample(Vec::new(), 100, 20, SEEDED);
    // restore ?(next instruction): on failure execution simply continues.
    let restore = cat(&[op::op0(op::RESTORE), op::branch(true, 2)]);
    let body = cat(&[
        cat(&[op::var(op::RANDOM, &[S(100)]), op::store(0)]),
        print_num(V(0)),
        print(" "),
        op::op2(op::INC_CHK, S(g(9)), S(19)),
    ]);
    let back = -(body.len() as i16 + 2) + 2;
    let mut story = Story::new();
    story.main(&cat(&[restore, body, op::branch_long(false, back), quit()]));
    let mut m = Machine::load(story.build(), SEEDED).unwrap();
    let mut out = String::new();
    loop {
        let event = m.run(BIG).unwrap();
        out.push_str(&m.take_output());
        match event {
            Event::Finished => break,
            Event::RestoreRequested => m.provide_restore(None).unwrap(),
            other => panic!("unexpected {other:?}"),
        }
    }
    let got: Vec<i32> = out.split_whitespace().map(|n| n.parse().unwrap()).collect();
    assert_eq!(got, plain);
}
