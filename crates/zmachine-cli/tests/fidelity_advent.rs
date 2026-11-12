//! The headline test for versions 5 and 8 (002 spec User Story 5, SC-003): a full
//! game of Advent, compiled both ways from one source, whose transcripts match the
//! ones Frotz 2.55 writes, byte for byte, apart from differences recorded in the
//! divergence ledger. See `stories/advent/fidelity/README.md`.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use zmachine_cli::ledger;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const BUILDS: [&str; 2] = ["z5", "z8"];

fn path(relative: &str) -> PathBuf {
    Path::new(ROOT).join(relative)
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(path(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

fn story(build: &str) -> PathBuf {
    path(&format!("stories/advent/advent.{build}"))
}

fn story_digest(build: &str) -> String {
    let bytes = std::fs::read(story(build)).unwrap();
    Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn seed() -> String {
    read("stories/advent/fidelity/seed.txt").trim().to_string()
}

fn reference(build: &str) -> String {
    read(&format!("stories/advent/fidelity/reference_{build}.scr"))
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-advent-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The walkthrough: the transcript's file name (its second line) and the commands
/// echoed in the transcript (everything after it).
fn walkthrough() -> (String, Vec<String>) {
    let script = read("stories/advent/fidelity/walkthrough.txt");
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

/// Plays the walkthrough with `zm` and returns the transcript it wrote. The
/// reference shows the interpreter as `1F`, so `zm` is told to call itself `F`.
fn our_transcript(build: &str, dir: &Path) -> String {
    let (name, _) = walkthrough();
    let out = Command::new(env!("CARGO_BIN_EXE_zm"))
        .arg("--script")
        .arg(path("stories/advent/fidelity/walkthrough.txt"))
        .args(["--seed", &seed(), "--interpreter-version", "F"])
        .args(["--max-instructions", "50000000", "--dir"])
        .arg(dir)
        .arg(story(build))
        .stdin(Stdio::null())
        .output()
        .expect("zm runs");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{build}: zm did not finish the walkthrough: {stderr}"
    );
    assert_eq!(
        stderr, "",
        "{build}: the walkthrough should cause no faults or diagnostics"
    );
    std::fs::read_to_string(dir.join(name)).expect("zm wrote the transcript")
}

// Principle VI: each story file is the one its record describes.
#[test]
fn the_story_files_match_their_record() {
    let record = read("stories/advent/SOURCE.md");
    for build in BUILDS {
        let digest = story_digest(build);
        assert!(
            record.contains(&digest),
            "advent.{build} has digest {digest}, not in SOURCE.md"
        );
    }
}

// contracts/transcript-format.md, "Differences from the reference interpreter": the
// limits a script must keep so that it means the same to both interpreters.
#[test]
fn the_walkthrough_keeps_to_the_rules_for_scripts() {
    let (name, commands) = walkthrough();
    assert!(
        !name.contains(['/', '\\']),
        "the transcript name is a plain file name"
    );
    for (n, line) in commands.iter().enumerate() {
        assert!(
            !line.contains('\\'),
            "line {}: a backslash is an escape to dfrotz",
            n + 3
        );
        assert!(
            line.len() < 198,
            "line {}: dfrotz keeps only 198 characters",
            n + 3
        );
        let word = line.split_whitespace().next().unwrap_or("");
        // `script` a second time would make Frotz ask for a file name again.
        for banned in ["save", "restore", "unscript", "script", "undo"] {
            assert_ne!(word, banned, "line {}: {banned}", n + 3);
        }
    }
    assert!(
        commands.iter().any(|c| c == "version"),
        "the walkthrough prints the interpreter's number and letter"
    );
    assert!(seed().parse::<u32>().is_ok_and(|s| s <= 2_147_483_647));
}

// SC-003: zero unledgered byte differences from the reference transcript, for the
// version 5 build and for the version 8 build.
#[test]
fn the_walkthrough_transcripts_match_the_reference() {
    let (_, commands) = walkthrough();
    let commands: Vec<&str> = commands.iter().map(String::as_str).collect();
    let rules =
        ledger::parse(&read("docs/divergences.md")).unwrap_or_else(|e| panic!("ledger: {e}"));
    for build in BUILDS {
        let expected = ledger::apply(&reference(build), &rules, &story_digest(build), &commands)
            .unwrap_or_else(|e| panic!("{build}: applying the ledger to the reference: {e}"));
        let ours = our_transcript(build, &scratch_dir(&format!("walk-{build}")));
        if let Some(difference) = ledger::first_difference(&ours, &expected, &commands) {
            panic!(
                "advent.{build}: the transcript differs from the reference ({} bytes against {}):\n{difference}",
                ours.len(),
                expected.len()
            );
        }
    }
}

// Spec User Story 5: the transcript shows the game won with every point, and the
// interpreter named as the reference names itself.
#[test]
fn the_walkthrough_wins_the_game() {
    for build in BUILDS {
        let ours = our_transcript(build, &scratch_dir(&format!("wins-{build}")));
        for transcript in [&ours, &reference(build)] {
            assert!(
                transcript.contains("you scored 350 out of a possible 350"),
                "{build}: the maximum score"
            );
            assert!(
                transcript.contains("Standard interpreter 1.1 (1F)"),
                "{build}"
            );
        }
    }
}

// Spec User Story 2: the two builds are the same game and print the same text.
#[test]
fn the_two_builds_give_the_same_transcript() {
    assert_eq!(reference("z5"), reference("z8"));
    assert_eq!(
        our_transcript("z5", &scratch_dir("same-z5")),
        our_transcript("z8", &scratch_dir("same-z8"))
    );
}

// SC-003 of feature 001 on this game: the same script and seed give the same
// transcript.
#[test]
fn two_runs_of_the_walkthrough_are_identical() {
    for build in BUILDS {
        assert_eq!(
            our_transcript(build, &scratch_dir(&format!("a-{build}"))),
            our_transcript(build, &scratch_dir(&format!("b-{build}")))
        );
    }
}

// FR-028: with the reference interpreter installed, confirm it still writes the
// committed transcripts.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn dfrotz_still_writes_the_reference_transcripts() {
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

    let (name, _) = walkthrough();
    for build in BUILDS {
        let dir = scratch_dir(&format!("dfrotz-{build}"));
        let mut child = Command::new("dfrotz")
            .args(["-m", "-q", "-w", "80", "-S", "0", "-s", &seed()])
            .arg(story(build))
            .current_dir(&dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let script = read("stories/advent/fidelity/walkthrough.txt");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        child.wait().unwrap();
        let regenerated = std::fs::read_to_string(dir.join(format!("{name}.scr"))).unwrap();
        if let Some(difference) = ledger::first_difference(&regenerated, &reference(build), &[]) {
            panic!("{build}: dfrotz no longer writes the committed reference:\n{difference}");
        }
    }
}

// SC-009: each walkthrough completes in under five seconds in an optimized build.
#[test]
#[ignore = "timing; run with --release"]
fn each_walkthrough_takes_under_five_seconds() {
    for build in BUILDS {
        let dir = scratch_dir(&format!("timing-{build}"));
        let started = std::time::Instant::now();
        our_transcript(build, &dir);
        let elapsed = started.elapsed();
        assert!(
            // Five seconds, times TIME_LIMIT_FACTOR on a machine that is not the
            // measure of the interpreter.
            elapsed < zmachine_testkit::time_limit(std::time::Duration::from_secs(5)),
            "{build}: the walkthrough took {elapsed:?}"
        );
    }
}
