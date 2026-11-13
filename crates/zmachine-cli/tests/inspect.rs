//! `zm --inspect`, `--tree` and `--dictionary` (feature 003;
//! specs/003-introspection/contracts/cli.md; FR-032 to FR-035). The answers
//! themselves are tested in the `zmachine-introspect` crate; these tests are of
//! where `zm` writes them and that playing is unaffected.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn story(relative: &str) -> String {
    format!("{ROOT}/stories/{relative}")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-inspect-{}-{name}", std::process::id()));
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
    // A run that ends before reading its input (a usage error) closes the pipe;
    // on some systems the write then fails, which is not what is being tested.
    match child.stdin.take().unwrap().write_all(stdin.as_bytes()) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
        Err(e) => panic!("writing to zm: {e}"),
    }
    child.wait_with_output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn path(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// A line is one complete JSON object if its braces and brackets balance outside
/// strings and it has nothing after the last. (The JSON itself is read strictly
/// in the introspection crate's tests.)
fn is_one_json_object(line: &str) -> bool {
    let (mut depth, mut in_string, mut escaped) = (0i32, false, false);
    for (i, ch) in line.char_indices() {
        if in_string {
            match (escaped, ch) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth -= 1;
                if depth == 0 && i + 1 != line.len() {
                    return false;
                }
            }
            _ => {}
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0 && !in_string && line.starts_with('{') && line.ends_with('}')
}

// contracts/cli.md: `--inspect FILE` writes one line of JSON per stop, with the
// specification's version, the stop's number from 1, what the game wants, the
// family, and the four answers, in that order.
#[test]
fn inspect_writes_one_line_per_stop() {
    for (file, family, room) in [
        ("zork1/zork1.z3", "zil", "West of House"),
        ("advent/advent.z5", "inform6", "At End Of Road"),
        ("advent/advent.z8", "inform6", "At End Of Road"),
    ] {
        let dir = scratch_dir(&format!("lines-{}", file.replace(['/', '.'], "-")));
        let out_file = dir.join("inspect.jsonl");
        let out = zm(
            &["--seed", "1", "--inspect", &path(&out_file), &story(file)],
            "north\ninventory\nlook\n",
        );
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert_eq!(text(&out.stderr), "", "{file}");
        let written = std::fs::read_to_string(&out_file).unwrap();
        let lines: Vec<&str> = written.lines().collect();
        // Three commands, and the prompt at which the input ended: four stops.
        assert_eq!(lines.len(), 4, "{file}: {written}");
        assert!(written.ends_with('\n'));
        for (n, line) in lines.iter().enumerate() {
            assert!(
                is_one_json_object(line),
                "{file}: line {} is not one JSON object: {line}",
                n + 1
            );
            let start = format!(
                r#"{{"spec":"0.4.0","stop":{},"wants":"line","family":"{family}","pack":"none","location":{{"exactness":"exact","value":"#,
                n + 1
            );
            assert!(line.starts_with(&start), "{file}: {line}");
            for key in [
                r#","exits":{"exactness":"exact","#,
                r#","inventory":{"exactness":"exact","#,
                r#","scope":{"exactness":"approximate","#,
            ] {
                assert_eq!(line.matches(key).count(), 1, "{file}: {key} in {line}");
            }
            let order: Vec<usize> = [
                "\"location\":",
                "\"exits\":",
                "\"inventory\":",
                "\"scope\":",
            ]
            .iter()
            .map(|k| line.find(k).unwrap())
            .collect();
            assert!(
                order.windows(2).all(|pair| pair[0] < pair[1]),
                "{file}: the answers are in order"
            );
        }
        assert!(
            lines[0].contains(&format!(r#""name":"{room}""#)),
            "{file}: {}",
            lines[0]
        );
        assert!(!lines[1].contains(&format!(
            r#""location":{{"exactness":"exact","value":{{"room":{}"#,
            "x"
        )));
        assert_ne!(
            lines[0].split("\"exits\"").next(),
            lines[1].split("\"exits\"").next(),
            "{file}: north moved the player"
        );
    }
}

// contracts/cli.md: a stop for a single key says so.
#[test]
fn inspect_says_when_the_game_wants_a_key() {
    let dir = scratch_dir("key");
    let out_file = dir.join("inspect.jsonl");
    // Advent's help is a menu read a key at a time; `q` leaves it.
    let out = zm(
        &[
            "--seed",
            "1",
            "--inspect",
            &path(&out_file),
            &story("advent/advent.z5"),
        ],
        "help\nq\nlook\n",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let written = std::fs::read_to_string(&out_file).unwrap();
    let wants: Vec<&str> = written
        .lines()
        .map(|l| {
            if l.contains(r#""wants":"key""#) {
                "key"
            } else {
                "line"
            }
        })
        .collect();
    assert_eq!(wants, ["line", "key", "line", "line"], "{written}");
    let stops: Vec<bool> = written
        .lines()
        .enumerate()
        .map(|(n, l)| l.contains(&format!(r#""stop":{},"#, n + 1)))
        .collect();
    assert!(
        stops.iter().all(|&ok| ok),
        "stops are numbered from 1 across both kinds"
    );
}

// contracts/cli.md: with a script, a line for every stop; FILE is emptied when
// `zm` starts, so a second run leaves only its own lines.
#[test]
fn inspect_follows_a_whole_script_and_starts_the_file_afresh() {
    let dir = scratch_dir("script");
    let out_file = dir.join("inspect.jsonl");
    std::fs::write(&out_file, "left over from an earlier run\n").unwrap();
    let walkthrough = format!("{ROOT}/stories/zork1/fidelity/walkthrough.txt");
    let run = || {
        let out = zm(
            &[
                "--script",
                &walkthrough,
                "--seed",
                "11",
                "--dir",
                &path(&dir),
                "--inspect",
                &path(&out_file),
                &story("zork1/zork1.z3"),
            ],
            "",
        );
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        std::fs::read_to_string(&out_file).unwrap()
    };
    let first = run();
    assert!(!first.contains("left over"));
    // One line for each request for a line of input: the 333 lines of the
    // walkthrough less the one that names the transcript's file, which answers a
    // question from `zm` itself and not from the game.
    assert_eq!(first.lines().count(), 332);
    assert!(first.lines().all(is_one_json_object));
    assert_eq!(
        first
            .lines()
            .filter(|l| l.contains(r#""unknown":"ambiguous-player""#))
            .count(),
        6,
        "the six ties"
    );
    assert_eq!(run(), first, "a second run writes the same and no more");
}

// contracts/cli.md, FR-033, constitution Principle IV: standard output, the
// transcript and a save file are byte for byte the same with and without
// `--inspect`, and nothing of the inspection is in any of them.
#[test]
fn playing_is_unaffected_by_inspect() {
    let script = "script\nlog\nnorth\ntake all\nsave\ngame.qzl\ninventory\nquit\ny\n";
    let run = |name: &str, inspect: bool| {
        let dir = scratch_dir(name);
        let out_file = dir.join("inspect.jsonl");
        let mut args = vec![
            "--seed".to_string(),
            "5".to_string(),
            "--dir".to_string(),
            path(&dir),
        ];
        if inspect {
            args.extend(["--inspect".to_string(), path(&out_file)]);
        }
        args.push(story("zork1/zork1.z3"));
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = zm(&args, script);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        (
            out.stdout,
            std::fs::read(dir.join("log")).unwrap(),
            std::fs::read(dir.join("game.qzl")).unwrap(),
            dir,
        )
    };
    let (plain_out, plain_log, plain_save, _) = run("plain", false);
    let (out, log, save, dir) = run("inspected", true);
    assert_eq!(out, plain_out);
    assert_eq!(log, plain_log);
    assert_eq!(save, plain_save);
    assert!(!text(&out).contains("\"spec\"") && !text(&log).contains("\"spec\""));
    // The file is the developer's own output: it is where it was asked for, not
    // resolved inside --dir by the rules for names a game gives.
    assert!(dir.join("inspect.jsonl").exists());
    assert!(
        std::fs::read_to_string(dir.join("inspect.jsonl"))
            .unwrap()
            .lines()
            .count()
            >= 6
    );
}

// contracts/cli.md: `--inspect -` writes the lines to standard error, each
// beginning `{"spec"`, and nothing but the game's text to standard output.
#[test]
fn inspect_dash_writes_to_standard_error() {
    let out = zm(
        &["--seed", "1", "--inspect", "-", &story("zork1/zork1.z3")],
        "north\n",
    );
    assert_eq!(out.status.code(), Some(0));
    let stderr = text(&out.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 2, "{stderr}");
    assert!(
        lines
            .iter()
            .all(|l| l.starts_with(r#"{"spec":"0.4.0","stop":"#) && is_one_json_object(l))
    );
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("North of House") && !stdout.contains("\"spec\""),
        "{stdout}"
    );
    let plain = zm(&["--seed", "1", &story("zork1/zork1.z3")], "north\n");
    assert_eq!(out.stdout, plain.stdout);
}

// contracts/cli.md: a FILE that cannot be written is a usage error.
#[test]
fn an_unwritable_inspect_file_is_a_usage_error() {
    let dir = scratch_dir("unwritable");
    let missing = dir.join("no-such-directory").join("inspect.jsonl");
    let out = zm(
        &["--inspect", &path(&missing), &story("zork1/zork1.z3")],
        "north\n",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("inspect.jsonl"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(text(&out.stdout), "", "the game did not start");
    let out = zm(&["--inspect"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("--inspect needs a value"));
}

// contracts/cli.md, FR-034: `--tree` and `--dictionary` print and exit without
// playing, for a recognised story file and an unrecognised one.
#[test]
fn tree_and_dictionary_print_and_exit() {
    let out = zm(&["--tree", &story("zork1/zork1.z3")], "");
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert_eq!(stdout.lines().count(), 1);
    assert!(is_one_json_object(stdout.trim_end()));
    assert!(stdout.starts_with(r#"{"spec":"0.4.0","family":"zil","object_tree":{"exactness":"exact","value":[{"number":1,"#), "{}", &stdout[..120]);
    assert!(stdout.contains(r#""name":"West of House""#) && !stdout.contains("\"dictionary\""));
    assert!(
        !stdout.contains("ZORK I: The Great Underground Empire"),
        "the game is not played"
    );

    let out = zm(&["--dictionary", &story("advent/advent.z8")], "");
    let stdout = text(&out.stdout);
    assert!(stdout.starts_with(r#"{"spec":"0.4.0","family":"inform6","dictionary":{"exactness":"exact","value":[{"word":"#), "{}", &stdout[..120]);
    assert!(
        stdout.contains(r#"{"word":"north","parts":["noun","direction"],"direction":7}"#),
        "north is a direction"
    );
    assert!(is_one_json_object(stdout.trim_end()) && !stdout.contains("\"object_tree\""));

    // Both together: one object with both. czech is of neither family, and both
    // are still answered in full.
    let out = zm(&["--tree", "--dictionary", &story("czech/czech.z5")], "");
    assert_eq!(out.status.code(), Some(0));
    let stdout = text(&out.stdout);
    assert!(stdout.starts_with(
        r#"{"spec":"0.4.0","family":"unrecognised","object_tree":{"exactness":"exact","value":[{"#
    ));
    assert!(stdout.contains(r#","dictionary":{"exactness":"exact","value":["#));
    assert!(is_one_json_object(stdout.trim_end()));
}

// contracts/cli.md: a file that cannot be loaded is refused as usual; the new
// options are in the usage text.
#[test]
fn bad_files_are_refused_and_the_options_are_documented() {
    let dir = scratch_dir("bad");
    let bad = dir.join("bad.z5");
    std::fs::write(&bad, [5u8; 10]).unwrap();
    for option in ["--tree", "--dictionary"] {
        let out = zm(&[option, &path(&bad)], "");
        assert_eq!(out.status.code(), Some(2), "{option}");
        assert_eq!(text(&out.stdout), "");
        assert!(
            text(&out.stderr).contains("too short"),
            "{}",
            text(&out.stderr)
        );
    }
    let help = text(&zm(&["--help"], "").stdout);
    for option in ["--inspect FILE", "--tree", "--dictionary"] {
        assert!(help.contains(option), "{option} in:\n{help}");
    }
}

/// A game pack for Zork I as committed, with these entries. The identity is the
/// corpus manifest's, the header's release and the header's serial.
fn zork1_pack(entries: &str) -> String {
    format!(
        "pack: 1\nsha256: 37084966477dff679282de42974b2077156b1bd68fad92a65d4ea94d8eb64d79\nrelease: 119\nserial: 880429\n{entries}"
    )
}

// Feature 006 (specs/006-game-packs/contracts/commands.md, "`zm`"; FR-006).
// I§12.2, I§2.4: `--pack FILE` gives the reader a game pack, and each line says
// in the field `pack`, after `family`, what became of it. A pack that is refused
// or is another story's is not an error; a file that cannot be read is.
#[test]
fn pack_gives_a_game_pack_and_the_line_says_what_became_of_it() {
    let dir = scratch_dir("pack");
    let run = |pack: Option<&str>| {
        let out_file = dir.join("inspect.jsonl");
        let pack_file = dir.join("story.pack");
        let mut args = vec!["--seed".to_string(), "1".to_string()];
        if let Some(text) = pack {
            std::fs::write(&pack_file, text).unwrap();
            args.extend(["--pack".to_string(), path(&pack_file)]);
        }
        args.extend([
            "--inspect".to_string(),
            path(&out_file),
            story("zork1/zork1.z3"),
        ]);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = zm(&args, "north\n");
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        std::fs::read_to_string(&out_file).unwrap()
    };
    let field = |written: &str| {
        let line = written.lines().next().unwrap().to_string();
        let start = r#"{"spec":"0.4.0","stop":1,"wants":"line","family":"zil","pack":""#;
        assert!(line.starts_with(start), "{line}");
        line[start.len()..].split('"').next().unwrap().to_string()
    };
    assert_eq!(field(&run(None)), "none");
    let right = zork1_pack("open-attribute: 11\n");
    let with = run(Some(&right));
    assert_eq!(field(&with), "in-force");
    assert_eq!(
        field(&run(Some(&right.replace("release: 119", "release: 118")))),
        "not-matching"
    );
    assert_eq!(field(&run(Some("pack: 9\n"))), "refused");
    // Playing is the same with a pack as without.
    let played = |pack: Option<&str>| {
        let pack_file = dir.join("played.pack");
        let mut args = vec!["--seed", "1"];
        let name = path(&pack_file);
        if let Some(text) = pack {
            std::fs::write(&pack_file, text).unwrap();
            args.extend(["--pack", &name]);
        }
        let file = story("zork1/zork1.z3");
        args.push(&file);
        text(&zm(&args, "north\neast\nopen window\nin\n").stdout)
    };
    assert_eq!(played(Some(&right)), played(None));

    // A file that cannot be read is an error before the game starts.
    let missing = dir.join("no-such.pack");
    let out = zm(
        &["--pack", &path(&missing), &story("zork1/zork1.z3")],
        "north\n",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("no-such.pack"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(text(&out.stdout), "", "the game did not start");
    let out = zm(&["--pack"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("--pack needs a value"));
}

// I§12.3: on Zork I with its "open" attribute given, the window behind the house
// is a door, closed and then open, to the same room.
#[test]
fn a_door_is_reported_closed_and_then_open() {
    let dir = scratch_dir("door");
    let (out_file, pack_file) = (dir.join("inspect.jsonl"), dir.join("story.pack"));
    std::fs::write(&pack_file, zork1_pack("open-attribute: 11\n")).unwrap();
    let out = zm(
        &[
            "--seed",
            "1",
            "--pack",
            &path(&pack_file),
            "--inspect",
            &path(&out_file),
            &story("zork1/zork1.z3"),
        ],
        "north\neast\nopen window\n",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let written = std::fs::read_to_string(&out_file).unwrap();
    let lines: Vec<&str> = written.lines().collect();
    assert_eq!(lines.len(), 4);
    let door = |line: &str| {
        let at = line
            .find(r#""kind":"door""#)
            .unwrap_or_else(|| panic!("no door: {line}"));
        line[at..].split('}').next().unwrap().to_string()
    };
    assert!(
        !lines[0].contains(r#""kind":"door""#),
        "no door at the start"
    );
    let (closed, open) = (door(lines[2]), door(lines[3]));
    assert!(
        closed.ends_with(r#""state":"closed","dark_ahead":"unknown","source":"pack""#),
        "{closed}"
    );
    assert!(
        open.ends_with(r#""state":"open","dark_ahead":"unknown","source":"pack""#),
        "{open}"
    );
    assert_eq!(
        closed.replace("closed", "open"),
        open,
        "the same door and room"
    );
}
