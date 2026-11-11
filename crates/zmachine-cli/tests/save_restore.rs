//! Saving and restoring with `zm`, and interchange of save files with the
//! reference interpreter (spec User Story 5, SC-006, FR-031).
//!
//! `stories/zork1/fidelity/saves/` holds, for each of five points in the
//! walkthrough: `make_n.txt`, which plays to that point and saves as
//! `save_n.qzl`; `frotz_n.qzl`, the save dfrotz wrote; `cont_n.txt`, which in a
//! fresh run restores `save_n.qzl` and plays twenty more commands; and
//! `cont_n.scr`, the transcript dfrotz wrote for that.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const POINTS: std::ops::RangeInclusive<u32> = 1..=5;

fn repo(relative: &str) -> PathBuf {
    Path::new(ROOT).join(relative)
}

fn saves(name: &str) -> PathBuf {
    repo("stories/zork1/fidelity/saves").join(name)
}

fn seed() -> String {
    std::fs::read_to_string(repo("stories/zork1/fidelity/seed.txt"))
        .unwrap()
        .trim()
        .to_string()
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-save-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Runs `zm` on Zork I with a script, with files in `dir`.
fn zm(dir: &Path, script: &Path) -> Output {
    let out = Command::new(env!("CARGO_BIN_EXE_zm"))
        .arg("--script")
        .arg(script)
        .args(["--seed", &seed(), "--max-instructions", "50000000", "--dir"])
        .arg(dir)
        .arg(repo("stories/zork1/zork1.z3"))
        .stdin(Stdio::null())
        .output()
        .expect("zm runs");
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    out
}

fn zm_lines(dir: &Path, lines: &str) -> Output {
    let script = dir.join("script.txt");
    std::fs::write(&script, lines).unwrap();
    zm(dir, &script)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

// FR-031, SC-006: a save written by the reference interpreter is restored by this
// one, and play continues exactly as it does in the reference.
#[test]
fn saves_written_by_frotz_restore_and_continue_as_in_frotz() {
    for n in POINTS {
        let dir = scratch_dir(&format!("from-frotz-{n}"));
        std::fs::copy(
            saves(&format!("frotz_{n}.qzl")),
            dir.join(format!("save_{n}.qzl")),
        )
        .unwrap();
        let out = zm(&dir, &saves(&format!("cont_{n}.txt")));
        assert_eq!(text(&out.stderr), "", "save point {n}");
        assert_eq!(
            read(&dir.join(format!("cont_{n}"))),
            read(&saves(&format!("cont_{n}.scr"))),
            "save point {n}"
        );
    }
}

// SC-006: a save written by this interpreter, restored in a fresh run, gives the
// same continuation as the reference's save does in the reference.
#[test]
fn saves_written_by_zm_restore_and_continue_as_in_frotz() {
    for n in POINTS {
        let dir = scratch_dir(&format!("from-zm-{n}"));
        let out = zm(&dir, &saves(&format!("make_{n}.txt")));
        assert_eq!(text(&out.stderr), "", "save point {n}");
        let save = dir.join(format!("save_{n}.qzl"));
        let bytes =
            std::fs::read(&save).unwrap_or_else(|e| panic!("save point {n}: no save file: {e}"));
        assert_eq!(&bytes[..4], b"FORM", "save point {n}: a Quetzal file");
        assert!(
            text(&out.stdout).contains(">save\nOk.\n"),
            "save point {n}: the game reported success"
        );

        zm(&dir, &saves(&format!("cont_{n}.txt")));
        assert_eq!(
            read(&dir.join(format!("cont_{n}"))),
            read(&saves(&format!("cont_{n}.scr"))),
            "save point {n}"
        );
    }
}

// The continuation transcripts are worth comparing: each restores, then plays on.
#[test]
fn the_reference_continuations_restored_successfully() {
    for n in POINTS {
        let transcript = read(&saves(&format!("cont_{n}.scr")));
        assert!(
            transcript.contains(">restore\nOk.\n"),
            "cont_{n}.scr: the restore succeeded in Frotz"
        );
        assert!(
            transcript.matches("\n>").count() >= 20,
            "cont_{n}.scr: twenty commands follow"
        );
        assert!(!transcript.contains("Failed."), "cont_{n}.scr");
    }
}

// Spec User Story 5 scenario 3, FR-017: a save from another story file, or a
// damaged one, is refused; the game says so and play continues.
#[test]
fn a_damaged_or_foreign_save_is_refused_and_play_continues() {
    let good = std::fs::read(saves("frotz_1.qzl")).unwrap();
    let mut foreign = good.clone();
    foreign[22] ^= 1; // the first byte of the serial number in IFhd
    let truncated = good[..good.len() / 2].to_vec();
    for (what, bytes) in [
        ("foreign", foreign),
        ("truncated", truncated),
        ("empty", Vec::new()),
    ] {
        let dir = scratch_dir(&format!("refused-{what}"));
        std::fs::write(dir.join("bad.qzl"), bytes).unwrap();
        let out = zm_lines(&dir, "restore\nbad.qzl\nopen mailbox\n");
        let stdout = text(&out.stdout);
        assert!(stdout.contains(">restore\nFailed.\n"), "{what}: {stdout}");
        assert!(
            stdout.contains("Opening the small mailbox reveals a leaflet."),
            "{what}: play continued"
        );
    }
}

// Spec User Story 5 scenario 3: restoring a file that is not there fails the same
// way.
#[test]
fn restoring_a_missing_file_fails_and_play_continues() {
    let dir = scratch_dir("missing");
    let out = zm_lines(&dir, "restore\nnothing-here.qzl\nopen mailbox\n");
    let stdout = text(&out.stdout);
    assert!(stdout.contains(">restore\nFailed.\n"), "{stdout}");
    assert!(stdout.contains("Opening the small mailbox reveals a leaflet."));
}

// Spec User Story 5 scenario 5: if the host cannot store the save, the game
// reports failure and play continues.
#[test]
fn a_save_that_cannot_be_written_fails_and_play_continues() {
    let dir = scratch_dir("unwritable");
    let script = dir.join("script.txt");
    std::fs::write(&script, "save\ngame.qzl\nopen mailbox\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_zm"))
        .arg("--script")
        .arg(&script)
        .args(["--seed", "1", "--dir"])
        .arg(dir.join("no-such-directory"))
        .arg(repo("stories/zork1/zork1.z3"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = text(&out.stdout);
    assert!(stdout.contains(">save\nFailed.\n"), "{stdout}");
    assert!(stdout.contains("Opening the small mailbox reveals a leaflet."));
    assert!(
        text(&out.stderr).contains("game.qzl"),
        "the reason is reported: {}",
        text(&out.stderr)
    );
}

// contracts/cli.md: save and restore file names are resolved inside --dir; one with
// a path separator is refused.
#[test]
fn save_file_names_with_a_path_separator_are_refused() {
    let dir = scratch_dir("separator");
    let inner = dir.join("inner");
    std::fs::create_dir_all(&inner).unwrap();
    std::fs::copy(saves("frotz_1.qzl"), dir.join("outside.qzl")).unwrap();
    let out = zm_lines(
        &inner,
        "save\n../escaped.qzl\nrestore\n../outside.qzl\nopen mailbox\n",
    );
    let stdout = text(&out.stdout);
    assert!(stdout.contains(">save\nFailed.\n"), "{stdout}");
    assert!(stdout.contains(">restore\nFailed.\n"), "{stdout}");
    assert!(!dir.join("escaped.qzl").exists());
    let stderr = text(&out.stderr);
    assert_eq!(
        stderr
            .lines()
            .filter(|l| l.contains("refused the file name"))
            .count(),
        2,
        "{stderr}"
    );
}

// Saving then restoring in the same session returns to the saved position.
#[test]
fn save_then_restore_in_one_session() {
    let dir = scratch_dir("session");
    let out = zm_lines(
        &dir,
        "open mailbox\nsave\nhere.qzl\ntake leaflet\ninventory\nrestore\nhere.qzl\ninventory\nlook\n",
    );
    let stdout = text(&out.stdout);
    assert!(stdout.contains(">save\nOk.\n"), "{stdout}");
    assert!(
        stdout.contains(">inventory\nYou are carrying:\n  A leaflet\n"),
        "{stdout}"
    );
    assert!(stdout.contains(">restore\nOk.\n"), "{stdout}");
    assert!(
        stdout.contains(">inventory\nYou are empty-handed.\n"),
        "the leaflet is back in the mailbox: {stdout}"
    );
    assert!(
        stdout.contains("The small mailbox contains:\n  A leaflet\n"),
        "{stdout}"
    );
}

// An empty line at the file-name prompt takes the default, the story's name with
// ".qzl", as the reference offers.
#[test]
fn an_empty_save_name_takes_the_default() {
    let dir = scratch_dir("default");
    zm_lines(&dir, "save\n\nlook\n");
    assert!(dir.join("zork1.qzl").exists());
}

// FR-031, the other direction: saves written by this interpreter are restored by
// the reference interpreter, which then continues exactly as it does from its own.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn saves_written_by_zm_restore_in_dfrotz() {
    use std::io::Write;
    let version = Command::new("dfrotz")
        .arg("-v")
        .output()
        .expect("dfrotz is installed");
    assert!(
        text(&version.stdout).starts_with("FROTZ V2.55"),
        "the reference is Frotz 2.55"
    );

    for n in POINTS {
        let dir = scratch_dir(&format!("to-frotz-{n}"));
        zm(&dir, &saves(&format!("make_{n}.txt")));
        assert!(dir.join(format!("save_{n}.qzl")).exists(), "save point {n}");

        let mut child = Command::new("dfrotz")
            .args(["-m", "-q", "-w", "80", "-S", "0", "-s", &seed()])
            .arg(repo("stories/zork1/zork1.z3"))
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
            .write_all(read(&saves(&format!("cont_{n}.txt"))).as_bytes())
            .unwrap();
        child.wait().unwrap();
        assert_eq!(
            read(&dir.join(format!("cont_{n}.scr"))),
            read(&saves(&format!("cont_{n}.scr"))),
            "save point {n}: dfrotz continuing from a save written by zm"
        );
    }
}

// FR-028 for the saves: dfrotz still writes the committed saves and continuations.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn dfrotz_still_writes_the_reference_saves_and_continuations() {
    use std::io::Write;
    let run = |dir: &Path, script: &Path| {
        let mut child = Command::new("dfrotz")
            .args(["-m", "-q", "-w", "80", "-S", "0", "-s", &seed()])
            .arg(repo("stories/zork1/zork1.z3"))
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(read(script).as_bytes())
            .unwrap();
        child.wait().unwrap();
    };
    for n in POINTS {
        let dir = scratch_dir(&format!("regen-{n}"));
        run(&dir, &saves(&format!("make_{n}.txt")));
        assert_eq!(
            std::fs::read(dir.join(format!("save_{n}.qzl"))).unwrap(),
            std::fs::read(saves(&format!("frotz_{n}.qzl"))).unwrap(),
            "save point {n}"
        );
        run(&dir, &saves(&format!("cont_{n}.txt")));
        assert_eq!(
            read(&dir.join(format!("cont_{n}.scr"))),
            read(&saves(&format!("cont_{n}.scr"))),
            "save point {n}"
        );
    }
}
