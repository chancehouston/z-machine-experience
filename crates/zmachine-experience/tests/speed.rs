//! How long a command takes (SC-003, SC-007). Timing depends on the machine and
//! the build, so these run only when asked for:
//! `cargo test --release -p zmachine-experience --test speed -- --ignored`.

mod common;

use common::*;
use std::time::{Duration, Instant};
use zmachine_experience::{Seed, Session};
use zmachine_testkit::time_limit;

/// The limit for one command and its snapshot: the specification's ten
/// milliseconds, times `TIME_LIMIT_FACTOR` where a machine is not the measure of
/// the layer. A shared CI runner once took 44 ms over one command, where a
/// developer's Mac takes 0.3 ms over the slowest and 0.1 ms on average.
fn command_limit() -> Duration {
    time_limit(Duration::from_millis(10))
}

// SC-003: over the whole walkthrough, the slowest command together with the
// snapshot that follows it takes under 10 ms. SC-007: so does one more command
// after the whole game, when the transcript is at its longest.
#[test]
#[ignore = "timing; run with --release"]
fn every_command_and_its_snapshot_is_quick() {
    let script = read_text("stories/zork1/fidelity/walkthrough.txt");
    let mut session = game(11);
    let mut slowest = (Duration::ZERO, 0, String::new());
    let mut total = Duration::ZERO;
    let mut count = 0u32;
    for (index, line) in script.lines().enumerate().skip(2) {
        let at = session.stamp();
        let began = Instant::now();
        let _ = session.send_line(at, line);
        let json = session.snapshot_json(at.turn_count);
        let took = began.elapsed();
        assert!(!json.is_empty());
        total += took;
        count += 1;
        if took > slowest.0 {
            slowest = (took, index + 1, line.to_string());
        }
    }
    println!(
        "{count} commands; mean {:?}; slowest {:?} at line {} ({})",
        total / count,
        slowest.0,
        slowest.1,
        slowest.2
    );
    assert!(count > 300);
    assert!(slowest.0 < command_limit(), "{slowest:?}");

    // The walkthrough ends the game; a long transcript that is still live comes
    // from playing on without the last two lines.
    let mut long = game(11);
    let lines: Vec<&str> = script.lines().skip(2).collect();
    for line in &lines[..lines.len() - 2] {
        let _ = long.send_line(long.stamp(), line);
    }
    let at = long.stamp();
    assert!(at.turn_count > 300);
    let began = Instant::now();
    long.send_line(at, "look")
        .expect("the game is still waiting");
    let json = long.snapshot_json(at.turn_count);
    let took = began.elapsed();
    assert!(json.contains("\"command\":\"look\""));
    println!("one more command after {} turns: {took:?}", at.turn_count);
    assert!(took < command_limit(), "{took:?}");

    // And the whole transcript at once, as a new screen would ask for it.
    let began = Instant::now();
    let all = long.snapshot_json(0);
    println!(
        "the whole transcript ({} bytes): {:?}",
        all.len(),
        began.elapsed()
    );
}

// SC-001's share: creating a session, to its first stop, takes under 50 ms.
#[test]
#[ignore = "timing; run with --release"]
fn a_session_starts_quickly() {
    let story = read("stories/zork1/zork1.z3");
    let mut slowest = Duration::ZERO;
    for seed in 0..20 {
        let began = Instant::now();
        let session = Session::new(story.clone(), Seed::Fixed(seed)).unwrap();
        let json = session.snapshot_json(0);
        slowest = slowest.max(began.elapsed());
        assert!(json.len() > 100);
    }
    println!("slowest of 20 starts: {slowest:?}");
    assert!(
        slowest < time_limit(Duration::from_millis(50)),
        "{slowest:?}"
    );
}
