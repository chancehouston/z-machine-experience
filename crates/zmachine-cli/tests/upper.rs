//! `--upper` and `--style` (002 spec User Story 3; specs/002-v5-v8/contracts/cli.md):
//! showing what a game draws outside its main text, and its text styles.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Story5, programs, zencode};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn advent() -> String {
    format!("{ROOT}/stories/advent/advent.z5")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-upper-{}-{name}", std::process::id()));
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

fn write_story(dir: &Path, name: &str, bytes: &[u8]) -> String {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path.to_string_lossy().into_owned()
}

fn screen_program(dir: &Path, name: &str) -> String {
    let program = programs::screen_programs().into_iter().find(|p| p.name == name).unwrap();
    write_story(dir, &format!("{name}.z5"), &program.image)
}

/// The canvases `--upper` printed: the lines between each pair of rules.
fn canvases(stdout: &str, width: usize) -> Vec<Vec<String>> {
    let rule = "-".repeat(width);
    let mut found = vec![];
    let mut lines = stdout.lines();
    while let Some(line) = lines.next() {
        if line == rule {
            found.push(lines.by_ref().take_while(|l| *l != rule).map(str::to_string).collect());
        }
    }
    found
}

// Spec User Story 3 scenario 1: with `--upper`, Advent's status line is printed
// above the prompt, and again when it changes after a move.
#[test]
fn the_status_line_is_shown_above_the_prompt_and_follows_the_game() {
    let out = zm(&["--upper", "--seed", "1", &advent()], "enter building\ninventory\n");
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    let shown = canvases(&stdout, 80);
    assert_eq!(shown.len(), 3, "once for each prompt at which it had changed: {stdout}");
    assert_eq!(shown[0].len(), 1, "one row: the status line");
    let row = |n: usize| shown[n][0].split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(row(0), "At End Of Road Score: 36 Moves: 0");
    assert_eq!(row(1), "Inside Building Score: 36 Moves: 1");
    assert_eq!(row(2), "Inside Building Score: 36 Moves: 2");
    // The canvas comes after the game's text and directly above the prompt.
    let rule = "-".repeat(80);
    assert!(stdout.contains(&format!("down a gully.\n\n{rule}\n{}\n{rule}\n>", shown[0][0])), "{stdout}");
}

// The canvas is printed only when it has changed since it was last printed.
#[test]
fn an_unchanged_canvas_is_not_printed_again() {
    // `score` and `verbose`'s replies do not count as moves: the status line stays.
    let out = zm(&["--upper", "--seed", "1", &advent()], "score\nscore\nscore\n");
    let stdout = text(&out.stdout);
    assert_eq!(canvases(&stdout, 80).len(), 1, "{stdout}");
}

// Without `--upper` nothing of the canvas appears.
#[test]
fn without_upper_the_canvas_is_not_shown() {
    let stdout = text(&zm(&["--seed", "1", &advent()], "enter building\n").stdout);
    assert!(!stdout.contains("Score: 36"), "{stdout}");
    assert!(!stdout.contains(&"-".repeat(80)), "{stdout}");

    let dir = scratch_dir("box-hidden");
    let story = screen_program(&dir, "quotation_box");
    let stdout = text(&zm(&[&story], "look\nlook\n").stdout);
    assert!(!stdout.contains("Plugh"), "{stdout}");
}

// Spec User Story 3 scenario 3: a boxed quotation is shown when it appears and is
// gone at the next prompt.
#[test]
fn a_quotation_box_is_shown_once() {
    let dir = scratch_dir("box");
    let story = screen_program(&dir, "quotation_box");
    let out = zm(&["--upper", "--width", "40", &story], "look\nlook\n");
    let stdout = text(&out.stdout);
    let shown = canvases(&stdout, 40);
    assert_eq!(shown.len(), 2, "{stdout}");
    assert_eq!(stdout.matches("Plugh, he said").count(), 1, "{stdout}");
    assert_eq!(shown[0].len(), 4, "the status line and the three rows of the box");
    assert_eq!(shown[0][2], "       Plugh, he said ");
    assert_eq!(shown[1].len(), 1, "only the status line is left");
    assert!(shown[1][0].contains("Moves: 1"));
}

// A full-screen menu is shown at each keypress as it stands.
#[test]
fn a_menu_is_shown_at_each_keypress() {
    let dir = scratch_dir("menu");
    let story = screen_program(&dir, "menu");
    let stdout = text(&zm(&["--upper", "--width", "40", &story], "n\nq\n").stdout);
    let shown = canvases(&stdout, 40);
    // The canvas is shown when the game waits; after the last key it does not.
    assert_eq!(shown.len(), 2, "two keypresses: {stdout}");
    assert!(shown[0].contains(&"  > Instructions".to_string()), "{stdout}");
    assert!(shown[1].contains(&"  > History".to_string()), "{stdout}");
    assert!(stdout.trim_end().ends_with("Back in the game."));
}

fn print(s: &str) -> Vec<u8> {
    [op::op0(op::PRINT), zencode(s)].concat()
}

fn style(n: u8) -> Vec<u8> {
    op::var(op::SET_TEXT_STYLE, &[S(n)])
}

/// Prints plain, bold, italic, reverse and fixed-pitch words, and bold in the
/// upper window.
fn styled_story(dir: &Path) -> String {
    let mut story = Story5::new(5);
    story.main(
        &[
            op::var(op::SPLIT_WINDOW, &[S(1)]),
            op::var(op::SET_WINDOW, &[S(1)]),
            style(2),
            print("TOP"),
            style(0),
            op::var(op::SET_WINDOW, &[S(0)]),
            print("plain "),
            style(2),
            print("bold"),
            style(0),
            print(" "),
            style(4),
            print("italic"),
            style(0),
            print(" "),
            style(1),
            print("reverse"),
            style(0),
            print(" "),
            style(8),
            print("fixed"),
            style(0),
            print(" "),
            style(2),
            style(4),
            print("both"),
            style(0),
            print(" end\n>"),
            [op::var(op::READ_CHAR, &[S(1)]), op::store(16)].concat(),
            style(2),
            print("last"),
            op::op0(op::QUIT),
        ]
        .concat(),
    );
    write_story(dir, "styled.z5", &story.build())
}

// contracts/cli.md `--style`: `ansi` renders bold, italic and reverse with
// terminal escape codes; `plain` prints the text only; and when standard output
// is not a terminal `auto` is plain.
#[test]
fn styles_are_rendered_as_escape_codes_only_when_asked() {
    let dir = scratch_dir("styles");
    let story = styled_story(&dir);
    let plain = "plain bold italic reverse fixed both end\n>last\n";
    for args in [vec!["--style", "plain"], vec!["--style", "auto"], vec![]] {
        let mut args = args;
        args.push(&story);
        assert_eq!(text(&zm(&args, "x\n").stdout), plain, "{args:?}");
    }
    let ansi = text(&zm(&["--style", "ansi", &story], "x\n").stdout);
    assert_eq!(
        ansi,
        "plain \x1b[0;1mbold\x1b[0m \x1b[0;3mitalic\x1b[0m \x1b[0;7mreverse\x1b[0m fixed \
\x1b[0;1;3mboth\x1b[0m end\n>\x1b[0;1mlast\n\x1b[0m"
    );
}

// Escape codes take no room on the line: styled text wraps where plain text does.
#[test]
fn styled_text_wraps_like_plain_text() {
    let dir = scratch_dir("wrap");
    let story = styled_story(&dir);
    let strip = |s: &str| {
        let mut out = String::new();
        let mut chars = s.chars();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                }
            } else {
                out.push(ch);
            }
        }
        out
    };
    for width in ["12", "20", "33"] {
        let plain = text(&zm(&["--style", "plain", "--width", width, &story], "x\n").stdout);
        let ansi = text(&zm(&["--style", "ansi", "--width", width, &story], "x\n").stdout);
        assert_ne!(plain, ansi);
        assert_eq!(strip(&ansi), plain, "--width {width}");
    }
}

// With `--upper` and `--style ansi` the canvas carries its styles too.
#[test]
fn the_canvas_is_styled_with_ansi() {
    let dir = scratch_dir("canvas-style");
    let story = styled_story(&dir);
    let stdout = text(&zm(&["--upper", "--style", "ansi", "--width", "30", &story], "x\n").stdout);
    assert!(stdout.contains(&format!("{0}\n\x1b[0;1mTOP\x1b[0m\n{0}\n", "-".repeat(30))), "{stdout:?}");
    let stdout = text(&zm(&["--upper", "--style", "plain", "--width", "30", &story], "x\n").stdout);
    assert!(stdout.contains(&format!("{0}\nTOP\n{0}\n>", "-".repeat(30))), "{stdout:?}");
}

// contracts/cli.md: neither the canvas nor any escape code appears in a transcript.
#[test]
fn a_transcript_has_no_canvas_and_no_escape_codes() {
    let dir = scratch_dir("transcript");
    let out = zm(
        &["--upper", "--style", "ansi", "--seed", "1", "--dir", &dir.to_string_lossy(), &advent()],
        "script\nlog\nenter building\nversion\nquit\ny\n",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("\x1b[0;1m"), "Advent prints its title in bold: {stdout:?}");
    assert!(stdout.contains("Score: 36"));
    let transcript = std::fs::read_to_string(dir.join("log")).unwrap();
    assert!(transcript.contains("Inside Building"));
    assert!(!transcript.contains('\x1b'), "{transcript:?}");
    assert!(!transcript.contains("Score: 36"), "{transcript}");
    assert!(!transcript.contains("-----"), "{transcript}");
}

// 002 FR-033: Advent's status row after the walkthrough's first five commands
// reads as it does on dfrotz's screen.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn advents_status_row_matches_dfrotz() {
    let walkthrough = std::fs::read_to_string(format!("{ROOT}/stories/advent/fidelity/walkthrough.txt")).unwrap();
    // Skip `script` and the transcript's name.
    let commands: String = walkthrough.lines().skip(2).take(5).map(|l| format!("{l}\n")).collect();

    let ours = text(&zm(&["--upper", "--seed", "1", &advent()], &commands).stdout);
    let ours = canvases(&ours, 80);
    let our_row = ours.last().expect("a canvas was shown")[0].trim_end().to_string();

    let mut child = Command::new("dfrotz")
        .args(["-m", "-r", "cn", "-r", "ln", "-r", "lt", "-h", "24", "-w", "80", "-s", "1", &advent()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("dfrotz is installed");
    child.stdin.take().unwrap().write_all(commands.as_bytes()).unwrap();
    let theirs = text(&child.wait_with_output().unwrap().stdout);
    // dfrotz prints its whole screen at each prompt, rows numbered; row 1 follows
    // the echo of the previous prompt on the same line.
    let their_row = theirs
        .lines()
        .filter_map(|l| l.rfind("01  ").map(|at| l[at + 4..].trim_end().to_string()))
        .next_back()
        .expect("dfrotz printed its screen");
    assert_eq!(our_row, their_row);
    // `version`, the first of the five, takes no turn.
    assert!(our_row.contains("Moves: 4"), "{our_row}");
}
