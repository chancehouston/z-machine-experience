//! The project's headline test (spec User Story 4, SC-002): a full game of Zork I,
//! from the opening to winning with the maximum score, whose transcript matches the
//! one Frotz 2.55 writes, byte for byte, apart from differences recorded in the
//! divergence ledger. See `stories/zork1/fidelity/README.md`.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use zmachine_cli::ledger;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn path(relative: &str) -> PathBuf {
    Path::new(ROOT).join(relative)
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(path(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

fn story_digest() -> String {
    let bytes = std::fs::read(path("stories/zork1/zork1.z3")).unwrap();
    Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn seed() -> String {
    read("stories/zork1/fidelity/seed.txt").trim().to_string()
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-fidelity-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The walkthrough: the transcript's file name (its second line) and the commands
/// echoed in the transcript (everything after it).
fn walkthrough() -> (String, Vec<String>) {
    let script = read("stories/zork1/fidelity/walkthrough.txt");
    let mut lines = script.lines();
    assert_eq!(
        lines.next(),
        Some("script"),
        "the walkthrough starts the game's transcript"
    );
    let name = lines
        .next()
        .expect("then names the transcript file")
        .to_string();
    (name, lines.map(str::to_string).collect())
}

/// Plays the walkthrough with `zm` and returns the transcript it wrote.
fn our_transcript(dir: &Path) -> String {
    let (name, _) = walkthrough();
    let out = Command::new(env!("CARGO_BIN_EXE_zm"))
        .arg("--script")
        .arg(path("stories/zork1/fidelity/walkthrough.txt"))
        .args(["--seed", &seed(), "--max-instructions", "50000000", "--dir"])
        .arg(dir)
        .arg(path("stories/zork1/zork1.z3"))
        .stdin(Stdio::null())
        .output()
        .expect("zm runs");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "zm did not finish the walkthrough: {stderr}"
    );
    assert_eq!(
        stderr, "",
        "the walkthrough should cause no faults or diagnostics"
    );
    std::fs::read_to_string(dir.join(name)).expect("zm wrote the transcript")
}

// Principle VI: the story file is the one its record describes.
#[test]
fn the_story_file_matches_its_record() {
    let record = read("stories/zork1/SOURCE.md");
    let digest = story_digest();
    assert!(
        record.contains(&digest),
        "stories/zork1/zork1.z3 has digest {digest}, not in SOURCE.md"
    );
}

// contracts/transcript-format.md and data-model.md "Command script": the limits a
// script must keep to be usable with the reference interpreter.
#[test]
fn the_walkthrough_keeps_to_the_rules_for_scripts() {
    let (name, commands) = walkthrough();
    assert!(
        !name.contains(['/', '\\']),
        "the transcript name is a plain file name"
    );
    for (n, line) in commands.iter().enumerate() {
        assert!(
            !line.starts_with('\\'),
            "line {}: dfrotz takes a backslash as its own command",
            n + 3
        );
        assert!(
            line.len() < 198,
            "line {}: dfrotz keeps only 198 characters",
            n + 3
        );
        let word = line.split_whitespace().next().unwrap_or("");
        for banned in ["save", "restore", "unscript", "script"] {
            assert_ne!(
                word,
                banned,
                "line {}: {banned} would ask for a file name",
                n + 3
            );
        }
    }
    assert!(
        seed().parse::<u32>().is_ok_and(|s| s <= 2_147_483_647),
        "the seed is one dfrotz accepts"
    );
}

// SC-002: zero unledgered byte differences from the reference transcript.
// FR-023: the suite fails on any difference not recorded in the ledger.
// SC-008: every ledger rule for this story must correspond to a difference.
#[test]
fn the_walkthrough_transcript_matches_the_reference() {
    let (_, commands) = walkthrough();
    let commands: Vec<&str> = commands.iter().map(String::as_str).collect();
    let reference = read("stories/zork1/fidelity/reference.scr");
    let rules =
        ledger::parse(&read("docs/divergences.md")).unwrap_or_else(|e| panic!("ledger: {e}"));
    let expected = ledger::apply(&reference, &rules, &story_digest(), &commands)
        .unwrap_or_else(|e| panic!("applying the ledger to the reference: {e}"));

    let ours = our_transcript(&scratch_dir("walkthrough"));
    if let Some(difference) = ledger::first_difference(&ours, &expected, &commands) {
        panic!(
            "the transcript differs from the reference ({} bytes against {}):\n{difference}",
            ours.len(),
            expected.len()
        );
    }
}

// Spec User Story 4 scenario 4: the transcript shows the game was won with the
// maximum score.
#[test]
fn the_walkthrough_wins_the_game() {
    let ours = our_transcript(&scratch_dir("wins"));
    assert!(
        ours.contains("Your score is 350 (total of 350 points)"),
        "the maximum score"
    );
    assert!(ours.contains("This gives you the rank of Master Adventurer."));
    assert!(
        ours.contains("Inside the Barrow\n"),
        "the game's ending was reached"
    );
    let reference = read("stories/zork1/fidelity/reference.scr");
    assert!(
        reference.contains("Your score is 350 (total of 350 points)"),
        "so does the reference"
    );
}

// SC-003 on the real game: the same script and seed give the same transcript.
#[test]
fn two_runs_of_the_walkthrough_are_identical() {
    assert_eq!(
        our_transcript(&scratch_dir("run-a")),
        our_transcript(&scratch_dir("run-b"))
    );
}

// FR-028: with the reference interpreter installed, confirm it still writes the
// committed transcript. A different result means dfrotz has changed or the
// committed file was edited.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn dfrotz_still_writes_the_reference_transcript() {
    use std::io::Write;
    let version = Command::new("dfrotz")
        .arg("-v")
        .output()
        .expect("dfrotz is installed");
    let version = String::from_utf8_lossy(&version.stdout).into_owned();
    assert!(
        version.starts_with("FROTZ V2.55"),
        "the reference is Frotz 2.55, found: {version}"
    );

    let dir = scratch_dir("dfrotz");
    let (name, _) = walkthrough();
    let mut child = Command::new("dfrotz")
        .args(["-m", "-q", "-w", "80", "-S", "0", "-s", &seed()])
        .arg(path("stories/zork1/zork1.z3"))
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(read("stories/zork1/fidelity/walkthrough.txt").as_bytes())
        .unwrap();
    child.wait().unwrap();

    let regenerated = std::fs::read_to_string(dir.join(format!("{name}.scr"))).unwrap();
    let committed = read("stories/zork1/fidelity/reference.scr");
    if let Some(difference) = ledger::first_difference(&regenerated, &committed, &[]) {
        panic!("dfrotz no longer writes the committed reference transcript:\n{difference}");
    }
}

// SC-005: the full walkthrough completes in under five seconds in an optimized
// build, so the fidelity test is cheap enough to run on every change.
#[test]
#[ignore = "timing; run with --release"]
fn the_walkthrough_takes_under_five_seconds() {
    let dir = scratch_dir("timing");
    let started = std::time::Instant::now();
    our_transcript(&dir);
    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs_f64() < 5.0,
        "the walkthrough took {elapsed:?}"
    );
}
