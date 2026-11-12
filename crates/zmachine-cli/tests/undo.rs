//! Undo on a real game, against the reference interpreter (002 spec User Story 6,
//! SC-007). `stories/advent/fidelity/undo.txt` makes thirty moves and then asks to
//! undo thirty times; Frotz 2.55 keeps twenty-five positions, so twenty-five undos
//! succeed and five are refused, and this interpreter must do the same.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const BUILDS: [&str; 2] = ["z5", "z8"];

fn repo(relative: &str) -> PathBuf {
    Path::new(ROOT).join(relative)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-undo-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn reference(build: &str) -> String {
    read(&repo(&format!("stories/advent/fidelity/undo_{build}.scr")))
}

/// The transcript's file name: the script's second line.
fn transcript_name() -> String {
    let script = read(&repo("stories/advent/fidelity/undo.txt"));
    let mut lines = script.lines();
    assert_eq!(lines.next(), Some("script"));
    lines.next().unwrap().to_string()
}

// SC-007: the undo script's transcript equals the reference's, for each build.
#[test]
fn undo_matches_the_reference() {
    for build in BUILDS {
        let dir = scratch_dir(build);
        let out = Command::new(env!("CARGO_BIN_EXE_zm"))
            .arg("--script")
            .arg(repo("stories/advent/fidelity/undo.txt"))
            .args(["--seed", "1", "--interpreter-version", "F", "--dir"])
            .arg(&dir)
            .arg(repo(&format!("stories/advent/advent.{build}")))
            .stdin(Stdio::null())
            .output()
            .expect("zm runs");
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            read(&dir.join(transcript_name())),
            reference(build),
            "advent.{build}"
        );
    }
}

// The references are worth matching: twenty-five undos succeed and five are refused.
#[test]
fn the_reference_undoes_twenty_five_times() {
    for build in BUILDS {
        let reference = reference(build);
        assert_eq!(
            reference.matches("[Previous turn undone.]").count(),
            25,
            "{build}"
        );
        assert_eq!(
            reference
                .matches("\"Undo\" failed.  [Not all interpreters provide it.]")
                .count(),
            5,
            "{build}"
        );
    }
}

// FR-028: dfrotz still writes the committed undo transcripts.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn dfrotz_still_writes_the_undo_transcripts() {
    use std::io::Write;
    let version = Command::new("dfrotz")
        .arg("-v")
        .output()
        .expect("dfrotz is installed");
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("FROTZ V2.55"));
    for build in BUILDS {
        let dir = scratch_dir(&format!("dfrotz-{build}"));
        let mut child = Command::new("dfrotz")
            .args(["-m", "-q", "-w", "80", "-S", "0", "-s", "1"])
            .arg(repo(&format!("stories/advent/advent.{build}")))
            .current_dir(&dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let script = read(&repo("stories/advent/fidelity/undo.txt"));
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        child.wait().unwrap();
        assert_eq!(
            read(&dir.join(format!("{}.scr", transcript_name()))),
            reference(build),
            "{build}"
        );
    }
}
