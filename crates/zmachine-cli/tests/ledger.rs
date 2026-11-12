//! The divergence ledger: parsing its rules, applying them to a reference
//! transcript, and reporting where two transcripts differ (research.md R10, FR-024,
//! SC-008). All inputs here are made up, because the real ledger may stay empty.

use zmachine_cli::ledger::{Rule, apply, first_difference, parse};

const STORY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn ledger(entries: &str) -> String {
    format!(
        "# Divergence ledger\n\nIntroductory text.\n\n## Entry format\n\n\
         ````text\n### D-000 Example\n\nStandard: §1\n\n```rule\nkind: czech\ntest: 1\n```\n````\n\n\
         ## Entries\n\n{entries}"
    )
}

fn entry(id: &str, standard: &str, rule: &str) -> String {
    format!("### {id} A title\n\nWhat differs and why.\n\n{standard}\n\n```rule\n{rule}\n```\n\n")
}

fn transcript_rule(after: usize, reference: &str, ours: &str) -> String {
    format!(
        "kind: transcript\nstory: {STORY}\nafter-command: {after}\nreference: {reference}\nours: {ours}"
    )
}

// A short session: an opening, three commands, and a question answered mid-line.
const COMMANDS: [&str; 4] = ["look", "north", "quit", "y"];
const REFERENCE: &str = "Opening text.\n\n>look\nA room.\nWarning: something.\n\n>north\nA hall.\n\n>quit\nReally? >y\n\n";

// research.md R10: an empty ledger is valid and has no rules. The example in the
// "Entry format" section is documentation, not a rule.
#[test]
fn an_empty_ledger_has_no_rules() {
    assert_eq!(parse(&ledger("None.\n")), Ok(vec![]));
    assert_eq!(parse(&ledger("")), Ok(vec![]));
}

// The ledger in the repository parses.
#[test]
fn the_real_ledger_parses() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/divergences.md");
    let text = std::fs::read_to_string(path).unwrap();
    parse(&text).unwrap_or_else(|e| panic!("docs/divergences.md: {e}"));
}

// research.md R10: both rule kinds, each with its identifier.
#[test]
fn both_rule_kinds_are_parsed() {
    let text = ledger(&format!(
        "{}{}",
        entry(
            "D-001",
            "Standard: §7.1",
            &transcript_rule(1, "Warning: something.\\n", "")
        ),
        entry("D-002", "Standard: §15 (mod)", "kind: czech\ntest: 137"),
    ));
    assert_eq!(
        parse(&text),
        Ok(vec![
            Rule::Transcript {
                id: "D-001".into(),
                story: STORY.into(),
                after_command: 1,
                reference: "Warning: something.\n".into(),
                ours: String::new(),
            },
            Rule::Czech {
                id: "D-002".into(),
                test: "137".into()
            },
        ])
    );
}

// docs/divergences.md: in `reference` and `ours`, `\n` stands for a line feed and
// `\\` for a backslash.
#[test]
fn escapes_in_rule_text() {
    let text = ledger(&entry(
        "D-001",
        "Standard: §3",
        &transcript_rule(0, "a\\nb\\\\n", "c\\\\"),
    ));
    let Ok(rules) = parse(&text) else {
        panic!("should parse")
    };
    let Rule::Transcript {
        reference, ours, ..
    } = &rules[0]
    else {
        panic!("transcript rule")
    };
    assert_eq!(reference, "a\nb\\n");
    assert_eq!(ours, "c\\");
}

// FR-024: each entry must cite the section of the Standard that justifies it, and
// be specific. A rule that cannot be understood is an error, never ignored.
#[test]
fn malformed_entries_are_errors() {
    let good = transcript_rule(1, "x", "y");
    let cases = [
        ("no Standard line", entry("D-001", "", &good)),
        (
            "Standard line with no section",
            entry("D-001", "Standard: because", &good),
        ),
        (
            "unknown kind",
            entry("D-001", "Standard: §1", "kind: mystery\ntest: 1"),
        ),
        ("no kind", entry("D-001", "Standard: §1", "test: 1")),
        (
            "missing key",
            entry(
                "D-001",
                "Standard: §1",
                &format!("kind: transcript\nstory: {STORY}\nreference: x\nours: y"),
            ),
        ),
        (
            "command is not a number",
            entry(
                "D-001",
                "Standard: §1",
                &good.replace("after-command: 1", "after-command: one"),
            ),
        ),
        (
            "story is not a digest",
            entry("D-001", "Standard: §1", &good.replace(STORY, "zork1")),
        ),
        (
            "unknown key",
            entry("D-001", "Standard: §1", &format!("{good}\ncolour: red")),
        ),
        (
            "line without a colon",
            entry("D-001", "Standard: §1", &format!("{good}\njust words")),
        ),
        (
            "czech rule with no test",
            entry("D-001", "Standard: §1", "kind: czech"),
        ),
        (
            "empty reference",
            entry("D-001", "Standard: §1", &transcript_rule(1, "", "y")),
        ),
        (
            "rule before any entry heading",
            "Standard: §1\n\n```rule\nkind: czech\ntest: 1\n```\n".to_string(),
        ),
        (
            "unclosed rule block",
            "### D-001 T\n\nStandard: §1\n\n```rule\nkind: czech\ntest: 1\n".to_string(),
        ),
        (
            "entry with no rule",
            "### D-001 T\n\nStandard: §1\n\nJust prose.\n".to_string(),
        ),
        (
            "two rules in one entry",
            format!(
                "{}```rule\nkind: czech\ntest: 2\n```\n",
                entry("D-001", "Standard: §1", "kind: czech\ntest: 1")
            ),
        ),
        (
            "duplicate identifier",
            format!(
                "{0}{0}",
                entry("D-001", "Standard: §1", "kind: czech\ntest: 1")
            ),
        ),
    ];
    for (what, entries) in cases {
        let result = parse(&ledger(&entries));
        assert!(result.is_err(), "{what} should be an error, got {result:?}");
    }
}

fn rule(after: usize, reference: &str, ours: &str) -> Rule {
    Rule::Transcript {
        id: "D-001".into(),
        story: STORY.into(),
        after_command: after,
        reference: reference.into(),
        ours: ours.into(),
    }
}

// research.md R10: the comparison applies "exactly those substitutions to the
// reference transcript": the named text, after the named command, and nothing else.
#[test]
fn a_transcript_rule_substitutes_exactly_the_named_text() {
    let rules = [rule(1, "Warning: something.\n", "")];
    assert_eq!(
        apply(REFERENCE, &rules, STORY, &COMMANDS).unwrap(),
        "Opening text.\n\n>look\nA room.\n\n>north\nA hall.\n\n>quit\nReally? >y\n\n"
    );
    let rules = [rule(2, "A hall.", "A corridor.")];
    assert!(
        apply(REFERENCE, &rules, STORY, &COMMANDS)
            .unwrap()
            .contains(">north\nA corridor.\n\n>quit")
    );
    // Command 0 is the text before the first command.
    let rules = [rule(0, "Opening", "Closing")];
    assert!(
        apply(REFERENCE, &rules, STORY, &COMMANDS)
            .unwrap()
            .starts_with("Closing text.\n\n>look")
    );
    // The last command's text runs to the end of the transcript: here, one line feed.
    let rules = [rule(4, "\n", "")];
    let applied = apply(REFERENCE, &rules, STORY, &COMMANDS).unwrap();
    assert!(
        applied.ends_with("Really? >y\n") && !applied.ends_with("\n\n"),
        "{applied:?}"
    );
}

// The substitution is confined to the text after the named command: the same words
// elsewhere are left alone, and are not a match.
#[test]
fn a_rule_applies_only_after_its_command() {
    let reference = ">a\nsame\n\n>b\nsame\n\n>c\nsame\n";
    let commands = ["a", "b", "c"];
    assert_eq!(
        apply(reference, &[rule(2, "same", "changed")], STORY, &commands).unwrap(),
        ">a\nsame\n\n>b\nchanged\n\n>c\nsame\n"
    );
}

// SC-008: "an entry with no corresponding difference is flagged".
#[test]
fn a_rule_that_matches_nothing_is_an_error() {
    let wrong_text = apply(REFERENCE, &[rule(1, "No such text", "x")], STORY, &COMMANDS);
    assert!(wrong_text.unwrap_err().contains("D-001"));
    let wrong_command = apply(
        REFERENCE,
        &[rule(2, "Warning: something.\n", "")],
        STORY,
        &COMMANDS,
    );
    assert!(wrong_command.is_err(), "the text is after command 1, not 2");
    let no_such_command = apply(REFERENCE, &[rule(9, "A hall.", "x")], STORY, &COMMANDS);
    assert!(no_such_command.is_err());
}

// A rule must pin down one place. Text that occurs twice after its command is not
// specific enough (FR-024: "specific enough that the comparison excludes only that
// difference").
#[test]
fn a_rule_that_matches_twice_is_an_error() {
    let reference = ">a\nword word\n";
    assert!(apply(reference, &[rule(1, "word", "x")], STORY, &["a"]).is_err());
}

// A rule names the story file it is about. Rules for other story files are not
// applied and are not errors here; czech rules are not transcript rules.
#[test]
fn rules_for_other_stories_and_czech_rules_are_left_alone() {
    let other = Rule::Transcript {
        id: "D-002".into(),
        story: OTHER.into(),
        after_command: 1,
        reference: "No such text".into(),
        ours: "x".into(),
    };
    let czech = Rule::Czech {
        id: "D-003".into(),
        test: "5".into(),
    };
    assert_eq!(
        apply(REFERENCE, &[other, czech], STORY, &COMMANDS).unwrap(),
        REFERENCE
    );
}

// A transcript that does not contain the commands in order cannot be divided up.
#[test]
fn a_reference_that_lacks_a_command_is_an_error() {
    assert!(
        apply(
            REFERENCE,
            &[rule(1, "A room.", "x")],
            STORY,
            &["look", "south"]
        )
        .is_err()
    );
}

// Spec User Story 4 scenario 2: on a difference, report the position of the first
// difference and the command that preceded it, with context from both sides.
#[test]
fn the_first_difference_is_located_and_described() {
    assert_eq!(first_difference(REFERENCE, REFERENCE, &COMMANDS), None);

    let ours = REFERENCE.replace("A hall.", "A hull.");
    let d = first_difference(&ours, REFERENCE, &COMMANDS).expect("they differ");
    assert_eq!(d.offset, REFERENCE.find("A hall.").unwrap() + 3);
    assert_eq!(d.after_command, 2);
    assert_eq!(d.command, "north");
    assert!(
        d.reference_context.contains("A hall."),
        "{}",
        d.reference_context
    );
    assert!(d.ours_context.contains("A hull."), "{}", d.ours_context);
    assert!(
        d.reference_context.contains(">north"),
        "context includes the command"
    );

    // A difference in the opening is before any command.
    let ours = REFERENCE.replace("Opening", "Opened ");
    let d = first_difference(&ours, REFERENCE, &COMMANDS).unwrap();
    assert_eq!((d.after_command, d.command.as_str()), (0, ""));
    assert_eq!(d.offset, 4);
}

// One transcript being a prefix of the other is a difference at the shorter length.
#[test]
fn a_truncated_transcript_differs_at_its_end() {
    let ours = &REFERENCE[..REFERENCE.len() - 1];
    let d = first_difference(ours, REFERENCE, &COMMANDS).unwrap();
    assert_eq!(d.offset, REFERENCE.len() - 1);
    assert_eq!((d.after_command, d.command.as_str()), (4, "y"));

    let d = first_difference(REFERENCE, ours, &COMMANDS).unwrap();
    assert_eq!(d.offset, REFERENCE.len() - 1);
}

// A difference inside a multi-byte character is reported without splitting it.
#[test]
fn context_never_splits_a_character() {
    let reference = ">look\nna\u{ef}ve caf\u{e9}\n";
    let ours = ">look\nna\u{ef}ve caf\u{e8}\n";
    let d = first_difference(ours, reference, &["look"]).unwrap();
    assert!(d.reference_context.contains("caf\u{e9}"));
    assert!(d.ours_context.contains("caf\u{e8}"));
}
