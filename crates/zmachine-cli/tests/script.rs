//! Scripted runs and transcript files (contracts/cli.md, spec User Story 2).

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use zmachine_testkit::programs::{random_probe, transcript_probes};

const ZORK1: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");
const CORE_FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zmachine-core/tests/fixtures"
);

/// A fresh directory for one test's files.
fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zm-script-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, bytes: impl AsRef<[u8]>) -> PathBuf {
    std::fs::write(path, bytes).unwrap();
    path.to_path_buf()
}

fn zm(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zm"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("zm runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn s(path: &Path) -> &str {
    path.to_str().unwrap()
}

/// Runs `story` with `script` (the lines of the script file) in `dir`.
fn run_script(dir: &Path, story: &Path, script: &str, extra: &[&str]) -> Output {
    let script = write(&dir.join("script.txt"), script);
    let mut args = vec!["--script", s(&script), "--dir", s(dir)];
    args.extend_from_slice(extra);
    args.push(s(story));
    zm(&args)
}

fn fixture(relative: &str) -> String {
    std::fs::read_to_string(format!("{CORE_FIXTURES}/{relative}")).unwrap()
}

// FR-020, contracts/cli.md: lines of the script are fed in order; the line after
// the game starts a transcript is the file name; the file holds the transcript in
// the reference's format. Every probe must give the file dfrotz gave.
#[test]
fn probes_write_the_reference_transcripts() {
    let dir = scratch_dir("probes");
    for probe in transcript_probes() {
        let story = write(&dir.join(format!("{}.z3", probe.name)), &probe.image);
        let mut script = format!("{}.out\n", probe.name);
        for line in probe.input {
            script.push_str(line);
            script.push('\n');
        }
        let out = run_script(&dir, &story, &script, &["--seed", "1"]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}: {}",
            probe.name,
            text(&out.stderr)
        );
        let written = std::fs::read_to_string(dir.join(format!("{}.out", probe.name)))
            .unwrap_or_else(|e| panic!("{}: no transcript file: {e}", probe.name));
        assert_eq!(
            written,
            fixture(&format!("transcript/{}.scr", probe.name)),
            "{}",
            probe.name
        );
    }
}

// SC-003: the same story, script and seed give the same transcript, run after run;
// and it is the transcript the reference gives for that seed.
#[test]
fn ten_runs_with_one_seed_give_ten_identical_transcripts() {
    let dir = scratch_dir("determinism");
    let story = write(&dir.join("probe.z3"), random_probe());
    let expected = fixture("random/seed_123456.txt");
    for i in 0..10 {
        let run_dir = dir.join(format!("run{i}"));
        std::fs::create_dir_all(&run_dir).unwrap();
        let out = run_script(&run_dir, &story, "numbers\n", &["--seed", "123456"]);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert_eq!(
            std::fs::read_to_string(run_dir.join("numbers")).unwrap(),
            expected,
            "run {i}"
        );
    }
}

// FR-014: with no seed, random numbers differ from run to run.
#[test]
fn unseeded_runs_differ() {
    let dir = scratch_dir("unseeded");
    let story = write(&dir.join("probe.z3"), random_probe());
    let mut transcripts = std::collections::HashSet::new();
    for i in 0..4 {
        let run_dir = dir.join(format!("run{i}"));
        std::fs::create_dir_all(&run_dir).unwrap();
        run_script(&run_dir, &story, "numbers\n", &[]);
        transcripts.insert(std::fs::read_to_string(run_dir.join("numbers")).unwrap());
    }
    assert!(
        transcripts.len() > 1,
        "four unseeded runs all gave the same 260 numbers"
    );
}

// US2 scenario 4: the terminal width does not affect the transcript file, which
// holds one paragraph per line.
#[test]
fn the_transcript_does_not_depend_on_the_width() {
    let script = "script\nlog\nlook\nnorth\n";
    let read = |width: &str| {
        let dir = scratch_dir(&format!("width{width}"));
        let out = run_script(
            &dir,
            Path::new(ZORK1),
            script,
            &["--seed", "1", "--width", width],
        );
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        std::fs::read_to_string(dir.join("log")).unwrap()
    };
    let wide = read("0");
    assert_eq!(wide, read("40"));
    assert_eq!(wide, read("80"));
    assert!(wide.contains(
        "You are standing in an open field west of a white house, with a boarded front door.\n"
    ));
}

// US2 scenario 3 and contracts/transcript-format.md rule 6: when the script ends
// while the game is waiting, the run ends normally and the transcript ends with
// the prompt, with no line feed after it.
#[test]
fn a_script_that_ends_at_a_prompt_exits_zero() {
    let dir = scratch_dir("ends");
    let out = run_script(
        &dir,
        Path::new(ZORK1),
        "script\nlog\nopen mailbox\n",
        &["--seed", "1"],
    );
    assert_eq!(out.status.code(), Some(0));
    let log = std::fs::read_to_string(dir.join("log")).unwrap();
    assert!(
        log.starts_with("Here begins a transcript of interaction with\nZORK I"),
        "{log}"
    );
    assert!(
        log.ends_with(">open mailbox\nOpening the small mailbox reveals a leaflet.\n\n>"),
        "{log}"
    );
}

// contracts/transcript-format.md rules 9 and 10: after the game turns the
// transcript off and on again, writing continues in the same file and no second
// file name is asked for.
#[test]
fn a_restarted_transcript_continues_in_the_same_file() {
    let dir = scratch_dir("again");
    let script = "script\nlog\nunscript\ninventory\nscript\nlook\n";
    let out = run_script(&dir, Path::new(ZORK1), script, &["--seed", "1"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let log = std::fs::read_to_string(dir.join("log")).unwrap();
    assert_eq!(log.matches("Here begins a transcript").count(), 2, "{log}");
    assert_eq!(log.matches("Here ends a transcript").count(), 1, "{log}");
    assert!(
        !log.contains("inventory"),
        "typed while the transcript was off: {log}"
    );
    assert!(
        log.contains(">look\nWest of House\n"),
        "look went to the game, not to a prompt: {log}"
    );
    assert!(!dir.join("look").exists());
}

// contracts/cli.md: file names are resolved inside --dir; one with a path separator
// is refused and the game is told transcription is off.
#[test]
fn a_file_name_with_a_path_separator_is_refused() {
    let dir = scratch_dir("separator");
    let inner = dir.join("inner");
    std::fs::create_dir_all(&inner).unwrap();
    for name in ["../escaped", "sub/file", "/tmp/zm-absolute-test", ".."] {
        let script = format!("script\n{name}\nlook\n");
        let out = run_script(&inner, Path::new(ZORK1), &script, &["--seed", "1"]);
        assert_eq!(out.status.code(), Some(0), "{name}");
        assert!(
            text(&out.stderr).contains("file name"),
            "{name}: {}",
            text(&out.stderr)
        );
        // Zork I reports the failure itself when it finds the bit cleared.
        assert!(
            text(&out.stdout).contains("West of House"),
            "{name}: play continued"
        );
    }
    assert!(!dir.join("escaped").exists());
    assert!(!inner.join("sub").exists());
    assert!(!Path::new("/tmp/zm-absolute-test").exists());
    let left: Vec<_> = std::fs::read_dir(&inner)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, vec![std::ffi::OsString::from("script.txt")]);
}

// An empty line at the file-name prompt takes the default name, the story's name
// with ".scr", as the reference offers.
#[test]
fn an_empty_file_name_takes_the_default() {
    let dir = scratch_dir("default");
    let out = run_script(&dir, Path::new(ZORK1), "script\n\nlook\n", &["--seed", "1"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let log = std::fs::read_to_string(dir.join("zork1.scr")).unwrap();
    assert!(log.contains(">look\n"));
}

// If the script ends at the file-name prompt, the transcript is declined and the
// run still ends normally.
#[test]
fn a_script_that_ends_at_the_file_name_prompt_exits_zero() {
    let dir = scratch_dir("noname");
    let out = run_script(&dir, Path::new(ZORK1), "script\n", &["--seed", "1"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
}

// contracts/cli.md: `--script -` reads the script from standard input.
#[test]
fn script_dash_reads_standard_input() {
    use std::io::Write;
    let dir = scratch_dir("dash");
    let mut child = Command::new(env!("CARGO_BIN_EXE_zm"))
        .args(["--script", "-", "--dir", s(&dir), "--seed", "1", ZORK1])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"script\nlog\nlook\n")
        .unwrap();
    assert_eq!(child.wait_with_output().unwrap().status.code(), Some(0));
    assert!(
        std::fs::read_to_string(dir.join("log"))
            .unwrap()
            .contains(">look\n")
    );
}

// A script file that cannot be read is a usage error.
#[test]
fn a_missing_script_file_is_a_usage_error() {
    let out = zm(&["--script", "/nonexistent/zm-script", ZORK1]);
    assert_eq!(out.status.code(), Some(2));
    assert!(!out.stderr.is_empty());
}

// ---- Checks against the reference interpreter itself ----

fn dfrotz(dir: &Path, story: &Path, stdin: &str, seed: &str) {
    use std::io::Write;
    let version = Command::new("dfrotz")
        .arg("-v")
        .output()
        .expect("dfrotz is installed");
    assert!(
        text(&version.stdout).starts_with("FROTZ V2.55"),
        "these tests need Frotz 2.55"
    );
    let mut child = Command::new("dfrotz")
        .args(["-m", "-q", "-w", "80", "-S", "0", "-s", seed, s(story)])
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
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait().unwrap();
}

// FR-028 for the fixtures: dfrotz still writes the committed transcripts for the
// probe programs.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn probes_match_dfrotz() {
    let dir = scratch_dir("dfrotz-probes");
    for probe in transcript_probes() {
        let story = write(&dir.join(format!("{}.z3", probe.name)), &probe.image);
        let mut input = format!("{}\n", probe.name);
        for line in probe.input {
            input.push_str(line);
            input.push('\n');
        }
        dfrotz(&dir, &story, &input, "1");
        let reference = std::fs::read_to_string(dir.join(format!("{}.scr", probe.name))).unwrap();
        assert_eq!(
            reference,
            fixture(&format!("transcript/{}.scr", probe.name)),
            "{}",
            probe.name
        );
    }
}

// A foretaste of User Story 4: the same script, with combat, through both
// interpreters gives the same transcript file.
#[test]
#[ignore = "needs dfrotz 2.55"]
fn a_zork1_session_with_combat_matches_dfrotz() {
    let script = "script\nlog\nnorth\neast\nopen window\nenter\ntake all\nwest\ntake sword\n\
take lamp\nmove rug\nopen trap door\nturn on lamp\ndown\nnorth\nkill troll with sword\n\
kill troll with sword\nkill troll with sword\nkill troll with sword\ndiagnose\nxyzzy\n\n\
inventory\nscore\nquit\ny\n";
    for seed in ["4242", "7", "2147483647"] {
        let ours_dir = scratch_dir(&format!("session-ours-{seed}"));
        let theirs_dir = scratch_dir(&format!("session-theirs-{seed}"));
        let out = run_script(&ours_dir, Path::new(ZORK1), script, &["--seed", seed]);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        dfrotz(&theirs_dir, Path::new(ZORK1), script, seed);
        let ours = std::fs::read_to_string(ours_dir.join("log")).unwrap();
        let theirs = std::fs::read_to_string(theirs_dir.join("log.scr")).unwrap();
        assert!(
            theirs.contains("troll"),
            "the reference session reached the troll"
        );
        assert_eq!(ours, theirs, "seed {seed}");
    }
}
