//! The czech conformance program for versions 3, 5 and 8 (spec User Story 3,
//! SC-001, FR-022; 002 spec User Story 4, SC-002).
//!
//! czech runs 349 self-checking tests of the instruction set and prints 19 more
//! results that only a comparison with its expected output can check. Both are
//! verified here. The version 5 and 8 builds run 406 and print 19. See
//! `stories/czech/SOURCE.md`.

use sha2::{Digest, Sha256};
use std::process::{Command, Stdio};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/czech");
const LEDGER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/divergences.md");

fn read(name: &str) -> Vec<u8> {
    std::fs::read(format!("{DIR}/{name}")).unwrap_or_else(|e| panic!("stories/czech/{name}: {e}"))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Runs czech and returns what it printed, unwrapped.
fn run_czech() -> String {
    run_czech_version(3)
}

fn run_czech_version(version: u8) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_zm"))
        .args([
            "--seed",
            "1",
            "--width",
            "0",
            "--max-instructions",
            "10000000",
        ])
        .arg(format!("{DIR}/czech.z{version}"))
        .stdin(Stdio::null())
        .output()
        .expect("zm runs");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "czech for version {version} did not run to its end: {stderr}"
    );
    assert_eq!(stderr, "", "czech should cause no faults or diagnostics");
    String::from_utf8(out.stdout).expect("czech prints ASCII")
}

/// The lines czech prints about the interpreter's header, which its README says
/// "WILL be different for different interpreters" (stories/czech/SOURCE.md).
fn describes_the_interpreter(line: &str) -> bool {
    [
        "    standard ",
        "    interpreter ",
        "    Flags on:",
        "    Flags off:",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

// Principle VI: a committed story file is checked against its recorded digest, so
// the test is known to run the file the record describes.
#[test]
fn the_committed_files_match_their_record() {
    let record = String::from_utf8(read("SOURCE.md")).unwrap();
    for name in ["czech.z3", "czech.inf", "czech.out3", "README.txt"] {
        let digest = sha256(&read(name));
        assert!(
            record.contains(&digest),
            "stories/czech/{name} has digest {digest}, not in SOURCE.md"
        );
    }
}

// SC-001: "The czech conformance program for version 3 reports zero failed tests."
#[test]
fn czech_reports_no_failed_tests() {
    let output = run_czech();
    let failed: Vec<&str> = output.lines().filter(|l| l.contains("ERROR")).collect();
    assert!(failed.is_empty(), "czech reported:\n{}", failed.join("\n"));
    let summary = output
        .lines()
        .find(|l| l.starts_with("Passed:"))
        .unwrap_or_else(|| panic!("no summary line in:\n{output}"));
    assert_eq!(summary, "Passed: 349, Failed: 0, Print tests: 19");
    assert!(output.contains("Performed 368 tests."));
    assert!(
        output.trim_end().ends_with("Last test: quit!"),
        "czech ran to its last line"
    );
}

// FR-022: the printed-output tests must equal czech's published expected output.
// Every line is compared except the four that describe the interpreter.
#[test]
fn czech_output_matches_the_published_expected_output() {
    let expected = String::from_utf8(read("czech.out3"))
        .unwrap()
        .replace("\r\n", "\n");
    let output = run_czech();
    let expected: Vec<&str> = expected.lines().collect();
    let ours: Vec<&str> = output.lines().collect();
    assert_eq!(ours.len(), expected.len(), "line count; ours:\n{output}");
    let mut excluded = 0;
    for (number, (theirs, ours)) in expected.iter().zip(&ours).enumerate() {
        if describes_the_interpreter(theirs) {
            assert!(
                describes_the_interpreter(ours),
                "line {}: {ours:?}",
                number + 1
            );
            excluded += 1;
            continue;
        }
        assert_eq!(ours, theirs, "line {} of czech's output", number + 1);
    }
    assert_eq!(
        excluded, 4,
        "exactly the four header lines are left out of the comparison"
    );
}

// The header lines that are excluded above still have to say true things about
// this interpreter (§11.1, contracts/core-api.md "Loading").
#[test]
fn czech_header_report_describes_this_interpreter() {
    let output = run_czech();
    let line = |prefix: &str| {
        output
            .lines()
            .find(|l| l.starts_with(prefix))
            .unwrap_or_else(|| panic!("no {prefix:?} line"))
    };
    assert_eq!(line("    standard ").trim(), "standard 1.1");
    let on = line("    Flags on:");
    let off = line("    Flags off:");
    assert!(!on.contains("screen-splitting") && off.contains("screen-splitting"));
    assert!(!on.contains("variable-pitch-default") && off.contains("variable-pitch-default"));
    assert!(
        off.contains("NO status"),
        "the status line is available: {off}"
    );
}

// research.md R10: a czech test may be excused only by a `czech` rule in the
// ledger, and a rule that excuses nothing is an error. No czech test fails, so
// there must be no such rule.
#[test]
fn the_ledger_excuses_no_czech_test() {
    let ledger = std::fs::read_to_string(LEDGER).unwrap();
    let rules = zmachine_cli::ledger::parse(&ledger).unwrap_or_else(|e| panic!("ledger: {e}"));
    let czech: Vec<&str> = rules
        .iter()
        .filter(|rule| matches!(rule, zmachine_cli::ledger::Rule::Czech { .. }))
        .map(|rule| rule.id())
        .collect();
    assert!(
        czech.is_empty(),
        "docs/divergences.md has czech rules {czech:?}, but czech reports no failed test to excuse"
    );
}

// ---- Versions 5 and 8 (feature 002) ----

/// From version 5 czech prints two more lines about the interpreter, and prints
/// the line naming the Standard only when the header names one. Its published
/// output came from an interpreter that named none.
fn describes_the_interpreter_v5(line: &str) -> bool {
    describes_the_interpreter(line)
        || line.starts_with("    Screen size:")
        || line.starts_with("    Default color:")
        // The flags line is long enough to be wrapped where it was recorded.
        || line.starts_with("redraw pending,")
        || line.starts_with("sound, using menus,")
}

// Principle VI for the version 5 and 8 files.
#[test]
fn the_version_5_and_8_files_match_their_record() {
    let record = String::from_utf8(read("SOURCE.md")).unwrap();
    for name in ["czech.z5", "czech.z8", "czech.out5", "czech.out8"] {
        let digest = sha256(&read(name));
        assert!(record.contains(&digest), "stories/czech/{name} has digest {digest}, not in SOURCE.md");
    }
}

// 002 SC-002: czech for versions 5 and 8 reports zero failed tests.
#[test]
fn czech_reports_no_failed_tests_in_versions_5_and_8() {
    for version in [5, 8] {
        let output = run_czech_version(version);
        let failed: Vec<&str> = output.lines().filter(|l| l.contains("ERROR")).collect();
        assert!(failed.is_empty(), "version {version}: czech reported:\n{}", failed.join("\n"));
        let summary = output
            .lines()
            .find(|l| l.starts_with("Passed:"))
            .unwrap_or_else(|| panic!("version {version}: no summary line in:\n{output}"));
        assert_eq!(summary, "Passed: 406, Failed: 0, Print tests: 19", "version {version}");
        assert!(output.contains("Performed 425 tests."), "version {version}");
        assert!(output.trim_end().ends_with("Last test: quit!"), "version {version}");
    }
}

// 002 FR-022: every line of the published expected output is matched, except the
// lines that describe the interpreter.
#[test]
fn czech_output_matches_the_published_output_in_versions_5_and_8() {
    for version in [5, 8] {
        let expected = String::from_utf8(read(&format!("czech.out{version}")))
            .unwrap()
            .replace("\r\n", "\n");
        let output = run_czech_version(version);
        let keep = |text: &str| -> Vec<String> {
            text.lines()
                .filter(|l| !describes_the_interpreter_v5(l))
                .map(|l| l.trim_end().to_string())
                .collect()
        };
        let (ours, theirs) = (keep(&output), keep(&expected));
        for (number, (ours, theirs)) in ours.iter().zip(&theirs).enumerate() {
            assert_eq!(ours, theirs, "version {version}: compared line {}", number + 1);
        }
        assert_eq!(ours.len(), theirs.len(), "version {version}: line count; ours:\n{output}");
        let left_out = expected.lines().filter(|l| describes_the_interpreter_v5(l)).count();
        assert!(left_out <= 6, "version {version}: {left_out} lines left out of the comparison");
    }
}

// The lines left out still have to say true things about this interpreter (§11.1,
// 002 data-model.md "What the interpreter writes into the header").
#[test]
fn czech_header_report_describes_this_interpreter_in_versions_5_and_8() {
    for version in [5, 8] {
        let output = run_czech_version(version);
        let line = |prefix: &str| {
            output
                .lines()
                .find(|l| l.starts_with(prefix))
                .unwrap_or_else(|| panic!("version {version}: no {prefix:?} line"))
        };
        assert_eq!(line("    standard ").trim(), "standard 1.1");
        assert!(line("    interpreter ").starts_with("    interpreter 1 Z "), "version {version}");
        assert_eq!(line("    Flags on:").trim(), "Flags on: boldface, italic, fixed-space,");
        let off = line("    Flags off:");
        assert!(off.contains("color") && off.contains("timer") && off.contains("sound"));
        assert_eq!(line("    Screen size:").trim(), "Screen size: 80x24; in 1x1 units: 80x24");
    }
}
