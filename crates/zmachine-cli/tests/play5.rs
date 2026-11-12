//! Playing version 5 and 8 stories with the `zm` binary (002 spec User Stories 1
//! and 2; specs/002-v5-v8/contracts/cli.md).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Story5, programs, zencode};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const KEYS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../zmachine-core/tests/fixtures/keys");

fn advent(build: &str) -> String {
    format!("{ROOT}/stories/advent/advent.{build}")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-play5-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn zm(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zm"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("zm starts");
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn print(s: &str) -> Vec<u8> {
    [op::op0(op::PRINT), zencode(s)].concat()
}

fn write_story(dir: &Path, name: &str, bytes: &[u8]) -> String {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path.to_string_lossy().into_owned()
}

/// The key probe story, written into `dir`.
fn key_probe(dir: &Path) -> String {
    write_story(dir, "keys.z5", &programs::key_probe())
}

// 002 spec User Story 1, Independent Test: the opening of Advent, a few commands,
// and a clean exit.
#[test]
fn advent_plays_from_the_terminal() {
    let out = zm(&[&advent("z5")], "enter building\ntake lamp\ninventory\nquit\ny\n");
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(text(&out.stderr), "");
    assert!(stdout.contains("Welcome to Adventure!"), "{stdout}");
    assert!(stdout.contains("At End Of Road\nYou are standing at the end of a road"), "{stdout}");
    assert!(stdout.contains(">enter building\n\nInside Building\n"), "{stdout}");
    assert!(stdout.contains(">take lamp\nTaken.\n"), "{stdout}");
    assert!(stdout.contains(">inventory\nYou're carrying:\n  a brass lantern\n"), "{stdout}");
    assert!(stdout.contains("Are you sure you want to quit?"), "{stdout}");
}

// contracts/cli.md: versions 3, 5 and 8 are played; any other is refused with exit
// status 2 and a message naming those three.
#[test]
fn other_versions_are_refused() {
    let dir = scratch_dir("versions");
    for version in [1u8, 2, 4, 6, 7, 9] {
        let mut story = Story5::new(5);
        story.main(&op::op0(op::QUIT));
        let mut image = story.build();
        image[0] = version;
        let path = write_story(&dir, &format!("v{version}.z{version}"), &image);
        let out = zm(&[&path], "");
        assert_eq!(out.status.code(), Some(2), "version {version}");
        let stderr = text(&out.stderr);
        assert!(stderr.contains(&format!("version {version}")), "{stderr}");
        assert!(stderr.contains("3, 5 and 8"), "names the supported versions: {stderr}");
        assert_eq!(text(&out.stdout), "");
    }
}

/// Runs the key probe on one of the recorded scripts and returns the transcript.
fn key_transcript(name: &str) -> String {
    let dir = scratch_dir(&format!("keys-{name}"));
    let story = key_probe(&dir);
    let script = format!("{KEYS}/{name}.in");
    let out = zm(&["--script", &script, "--dir", &dir.to_string_lossy(), &story], "");
    assert_eq!(out.status.code(), Some(0), "{name}: {}", text(&out.stderr));
    assert_eq!(text(&out.stderr), "", "{name}");
    std::fs::read_to_string(dir.join(name)).expect("zm wrote the transcript")
}

fn key_fixture(name: &str) -> String {
    std::fs::read_to_string(format!("{KEYS}/{name}.scr")).unwrap()
}

// contracts/cli.md "Keypresses": each key is the next unused character of the
// current input line; an empty line is the return key; `\_`, `\[`, `\?` and `\\`
// are return, escape, delete and a backslash. The transcripts are dfrotz's.
#[test]
fn keys_are_taken_from_lines_as_the_reference_takes_them() {
    assert_eq!(key_transcript("escapes"), key_fixture("escapes"));
    assert!(key_fixture("escapes").starts_with("keys: 97 98 99 13 13 27 8 92 120 121 122 33 \n"));
}

// "Characters left unused are dropped when the game next asks for a whole line."
#[test]
fn unused_keys_are_dropped_at_a_line_read() {
    assert_eq!(key_transcript("leftovers"), key_fixture("leftovers"));
    assert!(key_fixture("leftovers").contains(">hello\n13 5\n"));
}

// One key to a line, and `\\` inside a line of input is one backslash.
#[test]
fn one_key_to_a_line() {
    assert_eq!(key_transcript("one_per_line"), key_fixture("one_per_line"));
    assert!(key_fixture("one_per_line").contains(">LOOK \\ x\n13 8\n"));
}

// contracts/cli.md "Special keys": the arrows and function keys. dfrotz 2.55
// documents these escapes but its UTF-8 build turns their codes into question
// marks, so there is no reference transcript; the codes are §3.8.4's.
#[test]
fn arrow_and_function_key_escapes() {
    let dir = scratch_dir("special");
    let story = key_probe(&dir);
    // The probe starts a transcript, so the first line names its file.
    let dir_arg = dir.to_string_lossy().into_owned();
    let out = zm(
        &["--script", "-", "--dir", &dir_arg, &story],
        "t\n\\^\\.\\<\\>\\1\\2\\9\\0\\\\~ \\_\nlook\n\\[\\?\n\n",
    );
    assert_eq!(out.status.code(), Some(0));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("keys: 129 130 131 132 133 134 141 142 92 126 32 13"), "{stdout}");
    assert!(stdout.contains("more: 27 8 13"), "{stdout}");
}

// An escape dfrotz does not know is reported and dropped, as dfrotz does.
#[test]
fn an_unknown_escape_is_reported() {
    let dir = scratch_dir("unknown-escape");
    let story = key_probe(&dir);
    let dir_arg = dir.to_string_lossy().into_owned();
    let out = zm(&["--script", "-", "--dir", &dir_arg, &story], "t\na\\qb\n");
    assert!(text(&out.stdout).contains("keys: 97 98"), "{}", text(&out.stdout));
    assert!(text(&out.stderr).contains("unknown escape \\q"), "{}", text(&out.stderr));
}

// contracts/cli.md: a line beginning with a backslash and a lower-case letter is a
// command to dfrotz itself. `zm` refuses it with a message and reads on.
#[test]
fn a_dfrotz_command_line_is_refused() {
    let out = zm(&[&advent("z5")], "\\help\nenter building\n\\s\nquit\ny\n");
    assert_eq!(out.status.code(), Some(0));
    let stdout = text(&out.stdout);
    let stderr = text(&out.stderr);
    assert!(stdout.contains("Inside Building"), "the next line was taken as input: {stdout}");
    assert!(!stdout.contains("I didn't understand that sentence"), "{stdout}");
    assert_eq!(stderr.lines().filter(|l| l.contains("command to dfrotz")).count(), 2, "{stderr}");
    assert!(stderr.contains("\\\\help"), "{stderr}");
}

// contracts/cli.md: `--interpreter-version` sets the letter a game may print.
#[test]
fn the_interpreter_version_letter_can_be_set() {
    let default = text(&zm(&[&advent("z5")], "version\n").stdout);
    assert!(default.contains("Standard interpreter 1.1 (1Z)"), "{default}");
    let set = text(&zm(&["--interpreter-version", "F", &advent("z5")], "version\n").stdout);
    assert!(set.contains("Standard interpreter 1.1 (1F)"), "{set}");

    for bad in ["", "FF", "\u{e9}"] {
        let out = zm(&["--interpreter-version", bad, &advent("z5")], "");
        assert_eq!(out.status.code(), Some(2), "{bad:?}");
        assert!(text(&out.stderr).contains("--interpreter-version"), "{bad:?}");
    }
}

// contracts/cli.md: `--width` also sets the screen width the game sees (§8.4,
// header byte 0x21); with wrapping off the game is told 80.
#[test]
fn the_width_is_the_screen_width_the_game_sees() {
    let dir = scratch_dir("width");
    let mut story = Story5::new(5);
    story.main(
        &[
            [op::op2(op::LOADB, S(0), S(0x21)), op::store(16)].concat(),
            op::var(op::PRINT_NUM, &[V(16)]),
            print(" columns"),
            op::op0(op::QUIT),
        ]
        .concat(),
    );
    let path = write_story(&dir, "width.z5", &story.build());
    for (width, seen) in [("60", "60"), ("132", "132"), ("0", "80"), ("80", "80")] {
        let out = zm(&["--width", width, &path], "");
        assert_eq!(text(&out.stdout), format!("{seen} columns\n"), "--width {width}");
    }
    assert_eq!(text(&zm(&[&path], "").stdout), "80 columns\n");
}

// The new options are in the usage text and are checked.
#[test]
fn the_new_options_are_documented_and_checked() {
    let help = text(&zm(&["--help"], "").stdout);
    for option in ["--upper", "--style MODE", "--interpreter-version C"] {
        assert!(help.contains(option), "{option} in:\n{help}");
    }
    assert!(help.contains("version 3, 5 or 8"), "{help}");
    let out = zm(&["--style", "loud", &advent("z5")], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("--style must be ansi, plain or auto"));
}

// 002 spec User Story 2, scenario 1: one script of twenty commands gives the same
// output from the version 5 build and the version 8 build.
#[test]
fn the_two_builds_of_advent_play_alike() {
    let script = "enter building\ntake all\ninventory\nexamine lamp\nturn on lamp\nout\nsouth\n\
south\nsouth\nunlock grate with keys\nopen grate\ndown\nwest\ntake cage\nwest\nlook\nscore\n\
verbose\nwait\nxyzzy\n";
    assert_eq!(script.lines().count(), 20);
    let run = |build: &str| {
        let out = zm(&["--seed", "7", &advent(build)], script);
        assert_eq!(out.status.code(), Some(0), "{build}: {}", text(&out.stderr));
        assert_eq!(text(&out.stderr), "", "{build}");
        text(&out.stdout)
    };
    let (five, eight) = (run("z5"), run("z8"));
    assert!(five.contains("You are in a splendid chamber thirty feet high") || five.contains("Debris Room"), "the script reached the cave: {five}");
    assert_eq!(five, eight);
}

// FR-028 for the key fixtures: dfrotz still writes them.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn dfrotz_still_writes_the_key_fixtures() {
    let dir = scratch_dir("dfrotz-keys");
    let story = key_probe(&dir);
    for name in ["escapes", "leftovers", "one_per_line"] {
        let mut child = Command::new("dfrotz")
            .args(["-m", "-q", "-w", "80", "-S", "0", "-s", "1", &story])
            .current_dir(&dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("dfrotz is installed");
        let script = std::fs::read(format!("{KEYS}/{name}.in")).unwrap();
        child.stdin.take().unwrap().write_all(&script).unwrap();
        child.wait().unwrap();
        let written = std::fs::read_to_string(dir.join(format!("{name}.scr"))).unwrap();
        assert_eq!(written, key_fixture(name), "{name}");
    }
}
