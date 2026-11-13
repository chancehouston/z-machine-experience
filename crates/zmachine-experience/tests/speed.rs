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

// Feature 009, research R7 (tasks.md T018): what look-ahead's copies run, over
// each walkthrough the project holds. The limits in `lookahead::LIMITS` are set
// from these figures; this prints them so that they can be set again.
#[test]
#[ignore = "a measurement; run with --release -- --ignored --nocapture"]
fn what_look_ahead_runs_over_the_walkthroughs() {
    for (story, script, seed) in [
        (
            "stories/zork1/zork1.z3",
            "stories/zork1/fidelity/walkthrough.txt",
            11,
        ),
        (
            "stories/advent/advent.z5",
            "stories/advent/fidelity/walkthrough.txt",
            1,
        ),
    ] {
        let mut session = Session::new(read(story), Seed::Fixed(seed)).unwrap();
        session.set_look_ahead_limits(zmachine_experience::lookahead::Limits {
            copy: 50_000_000,
            total: u64::MAX,
        });
        let (mut most_copy, mut most_prompt, mut prompts, mut tried) = (0, 0, 0, 0);
        let mut slowest = Duration::ZERO;
        for line in read_text(script).lines() {
            if line.starts_with('#') {
                continue;
            }
            let began = Instant::now();
            if session.send_line(session.stamp(), line).is_err() {
                break;
            }
            let _ = session.snapshot_json(session.turns().len() - 1);
            slowest = slowest.max(began.elapsed());
            let (all, one) = session.look_ahead_cost();
            most_copy = most_copy.max(one);
            most_prompt = most_prompt.max(all);
            prompts += 1;
            tried += (all > 0) as u32;
        }
        println!(
            "{story}: {prompts} prompts, look-ahead at {tried}; most one copy ran {most_copy}; \
             most all copies ran at one prompt {most_prompt}; slowest command and snapshot {slowest:?}"
        );
    }
}

// Feature 011 (specs/011-resume-game): FR-020, FR-021, FR-022, SC-004, SC-005.
// A game is brought back by giving it its inputs again, so the time grows with
// the game. The limit for the layer's share of opening the app on a kept game
// is one second of the launch's two.
#[test]
#[ignore = "timing; run with --release -- --ignored --nocapture"]
fn a_kept_game_is_made_and_brought_back_quickly() {
    use zmachine_experience::Outcome;
    let story = read("stories/zork1/zork1.z3");
    let pack = read_text("packs/zork1.pack");
    let open = |kept: Option<&[u8]>| {
        Session::open_kept(story.clone(), Some(&pack), Seed::Fixed(11), kept).unwrap()
    };
    let limit = time_limit(Duration::from_secs(1));

    // The walkthrough: each command, its snapshot and its kept game together
    // stay within the limit for a command (SC-005).
    let script = read_text("stories/zork1/fidelity/walkthrough.txt");
    let (mut walked, _) = open(None);
    let mut slowest = Duration::ZERO;
    let mut slowest_keep = Duration::ZERO;
    for line in script.lines().skip(2) {
        let at = walked.stamp();
        let began = Instant::now();
        if walked.send_line(at, line).is_err() {
            break;
        }
        let json = walked.snapshot_json(at.turn_count);
        let before_keep = began.elapsed();
        let kept = walked.kept();
        let took = began.elapsed();
        assert!(!json.is_empty() && !kept.is_empty());
        slowest = slowest.max(took);
        slowest_keep = slowest_keep.max(took - before_keep);
    }
    let kept = walked.kept();
    println!(
        "walkthrough, {} turns: slowest command with snapshot and kept game {slowest:?}; \
         slowest kept game alone {slowest_keep:?}; kept game {} bytes",
        walked.turns().len(),
        kept.len()
    );
    assert!(slowest < command_limit(), "{slowest:?}");
    let began = Instant::now();
    let (back, outcome) = open(Some(&kept));
    let took = began.elapsed();
    assert_eq!(outcome, Outcome::BroughtBack);
    assert_eq!(back.turns().len(), walked.turns().len());
    println!("bringing back the walkthrough: {took:?}");
    assert!(took < limit, "{took:?}");

    // A game of 2,000 turns in which every command does work: a walk round
    // the house, each step parsed, moved and described.
    let (mut long, _) = open(None);
    let first = {
        let began = Instant::now();
        let kept = long.kept();
        (began.elapsed(), kept.len())
    };
    let round = ["north", "east", "south", "west"];
    let mut at_1000 = 0;
    for step in 0..2000 {
        say(&mut long, round[step % round.len()]);
        if step + 1 == 1000 {
            at_1000 = long.kept().len();
        }
    }
    assert_eq!(long.turns().len(), 2001);
    let places: std::collections::BTreeSet<&str> = long.turns()[1..]
        .iter()
        .filter_map(|turn| turn.text.lines().next())
        .collect();
    assert!(places.len() >= 4, "the walk goes round: {places:?}");
    let began = Instant::now();
    let kept = long.kept();
    let last = began.elapsed();
    println!(
        "kept game: at the first turn {:?} and {} bytes; at 1,000 turns {at_1000} bytes; \
         at 2,000 turns {last:?} and {} bytes",
        first.0,
        first.1,
        kept.len()
    );
    assert!(last < command_limit(), "{last:?}");
    // FR-022: in proportion to the turns, and no faster.
    let per_turn = (at_1000 - first.1) as f64 / 1000.0;
    let second_thousand = (kept.len() - at_1000) as f64 / 1000.0;
    assert!(
        (second_thousand - per_turn).abs() < 1.0,
        "{per_turn} {second_thousand}"
    );
    let began = Instant::now();
    let (back, outcome) = open(Some(&kept));
    let took = began.elapsed();
    assert_eq!(outcome, Outcome::BroughtBack);
    assert_eq!(back.snapshot_json(0), long.snapshot_json(0));
    println!("bringing back 2,000 turns: {took:?}");
    assert!(took < limit, "{took:?}");
}
