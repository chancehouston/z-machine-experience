//! Playing a story with the `zm` binary (contracts/cli.md, spec User Story 1).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Object, Story, zencode};

const ZORK1: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");

/// A fresh directory for one test's files.
fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-play-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_story(dir: &Path, bytes: &[u8]) -> PathBuf {
    let path = dir.join("story.z3");
    std::fs::write(&path, bytes).unwrap();
    path
}

fn zm(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zm"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("zm starts");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn print(s: &str) -> Vec<u8> {
    [op::op0(op::PRINT), zencode(s)].concat()
}

/// Prints a greeting, reads a line, says how many words it had, and quits.
fn echo_story() -> Vec<u8> {
    let mut story = Story::new();
    let room = story.object(Object::new("Room"));
    story.global(0, room as u16);
    let mut t = vec![0u8; 64];
    t[0] = 60;
    let t = story.scratch(&t);
    let mut p = vec![0u8; 22];
    p[0] = 5;
    let p = story.scratch(&p);
    let code = [
        print("Welcome.\n>"),
        op::var(op::SREAD, &[L(t), L(p)]),
        [op::op2(op::LOADB, L(p), S(1)), op::store(0)].concat(),
        print("Words: "),
        op::var(op::PRINT_NUM, &[V(0)]),
        op::op0(op::NEW_LINE),
        op::op0(op::QUIT),
    ]
    .concat();
    story.main(&code);
    story.build()
}

// US1 scenarios 1, 2 and 4: opening text, a response to a command, a clean exit.
#[test]
fn plays_a_story_and_exits_zero_on_quit() {
    let dir = scratch_dir("echo");
    let story = write_story(&dir, &echo_story());
    let out = zm(&[story.to_str().unwrap()], "one two three\n");
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(stdout.starts_with("Welcome.\n>"), "{stdout:?}");
    assert!(stdout.contains("Words: 3\n"), "{stdout:?}");
}

// contracts/cli.md: when input ends while the game wants a line, the run ends
// normally.
#[test]
fn end_of_input_while_waiting_exits_zero() {
    let dir = scratch_dir("eof");
    let story = write_story(&dir, &echo_story());
    let out = zm(&[story.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(text(&out.stdout), "Welcome.\n>");
}

// US1 scenarios 1 to 3 on the real game: the opening, a command it understands,
// and a word it does not know.
#[test]
fn zork1_opening_and_first_commands() {
    let out = zm(
        &["--seed", "1", ZORK1],
        "open mailbox\nfrobnicate\nquit\ny\n",
    );
    let stdout = text(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}\nstdout: {stdout}",
        text(&out.stderr)
    );
    assert!(
        stdout.contains("ZORK I: The Great Underground Empire"),
        "{stdout}"
    );
    assert!(
        stdout.contains("West of House\nYou are standing in an open field"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Opening the small mailbox reveals a leaflet."),
        "{stdout}"
    );
    assert!(
        stdout.contains("I don't know the word \"frobnicate\"."),
        "{stdout}"
    );
    assert!(
        stdout.contains("Do you wish to leave the game?"),
        "{stdout}"
    );
    assert_eq!(text(&out.stderr), "");
}

// FR-009: terminal output is wrapped at word boundaries to the width given; the
// default is 80 and 0 turns wrapping off.
#[test]
fn width_controls_wrapping() {
    let longest = |args: &[&str]| {
        let out = zm(args, "");
        text(&out.stdout)
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap()
    };
    assert!(longest(&[ZORK1]) <= 80);
    assert!(longest(&["--width", "40", ZORK1]) <= 40);
    assert!(longest(&["--width", "0", ZORK1]) > 80);

    let out = zm(&["--width", "40", ZORK1], "");
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("You are standing in an open field west\nof a white house,"),
        "{stdout}"
    );
}

// US1 scenario 5, FR-002: a file that is not a version 3 story is refused with a
// reason, and the exit status is 2.
#[test]
fn files_that_are_not_version_3_stories_are_refused() {
    let dir = scratch_dir("refuse");
    let mut v5 = echo_story();
    v5[0] = 5;
    let truncated = echo_story()[..40].to_vec();
    let unrelated =
        b"#!/bin/sh\necho this is a shell script, not a story file, and it is long enough\n";
    for (bytes, reason) in [
        (&v5[..], "version 5"),
        (&truncated[..], "too short"),
        (&unrelated[..], "version"),
    ] {
        let story = write_story(&dir, bytes);
        let out = zm(&[story.to_str().unwrap()], "");
        assert_eq!(out.status.code(), Some(2), "{reason}");
        assert!(
            text(&out.stderr).contains(reason),
            "{reason}: {}",
            text(&out.stderr)
        );
        assert_eq!(text(&out.stdout), "");
    }
    let out = zm(&[dir.join("missing.z3").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(!out.stderr.is_empty());
}

// contracts/cli.md: usage errors exit 2 with a message; --help and --version exit 0.
#[test]
fn usage_errors_and_help() {
    assert_eq!(zm(&[], "").status.code(), Some(2));
    assert_eq!(zm(&["--no-such-option", ZORK1], "").status.code(), Some(2));
    assert_eq!(zm(&["--width", "wide", ZORK1], "").status.code(), Some(2));
    assert_eq!(
        zm(&["--seed", "2147483648", ZORK1], "").status.code(),
        Some(2)
    );
    assert_eq!(zm(&[ZORK1, ZORK1], "").status.code(), Some(2));
    let help = zm(&["--help"], "");
    assert_eq!(help.status.code(), Some(0));
    assert!(text(&help.stdout).contains("--script"));
    let version = zm(&["--version"], "");
    assert_eq!(version.status.code(), Some(0));
    assert!(text(&version.stdout).contains(env!("CARGO_PKG_VERSION")));
}

// FR-018, FR-021: a faulting story exits 1 and names the fault and its address on
// standard error. Text printed before the fault still appears.
#[test]
fn a_faulting_story_exits_one_with_a_diagnostic() {
    let dir = scratch_dir("fault");
    let mut story = Story::new();
    let at = story.next_code_address();
    let before = print("About to divide.\n");
    story.main(
        &[
            before.clone(),
            op::op2(op::DIV, S(1), S(0)),
            op::store(0),
            op::op0(op::QUIT),
        ]
        .concat(),
    );
    let path = write_story(&dir, &story.build());
    let out = zm(&[path.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(text(&out.stdout), "About to divide.\n");
    let stderr = text(&out.stderr).to_lowercase();
    assert!(stderr.contains("division by zero"), "{stderr}");
    assert!(
        stderr.contains(&format!("{:#x}", at + before.len() as u32)),
        "{stderr}"
    );
}

// FR-032: a story that runs on without asking for input can be stopped.
#[test]
fn max_instructions_stops_an_endless_loop() {
    let dir = scratch_dir("loop");
    let mut story = Story::new();
    let inc = op::op1(op::INC, S(16));
    let back = -(inc.len() as i16 + 3) + 2;
    story.main(&[inc, op::op1(op::JUMP, L(back as u16))].concat());
    let path = write_story(&dir, &story.build());
    let out = zm(&["--max-instructions", "5000", path.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("5000"), "{}", text(&out.stderr));
}

// The limit counts work between requests for input, so a long game that keeps
// asking is not cut off.
#[test]
fn max_instructions_is_reset_by_each_request_for_input() {
    let out = zm(
        &["--max-instructions", "200000", "--seed", "1", ZORK1],
        "look\nlook\nlook\nquit\ny\n",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
}

// A tolerated illegal operation is reported on standard error, not in the game text.
#[test]
fn diagnostics_go_to_standard_error() {
    let dir = scratch_dir("diag");
    let mut story = Story::new();
    story.main(
        &[
            op::op1(op::REMOVE_OBJ, S(0)),
            print("fine\n"),
            op::op0(op::QUIT),
        ]
        .concat(),
    );
    let path = write_story(&dir, &story.build());
    let out = zm(&[path.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(text(&out.stdout), "fine\n");
    assert!(
        text(&out.stderr).contains("object 0"),
        "{}",
        text(&out.stderr)
    );
}
