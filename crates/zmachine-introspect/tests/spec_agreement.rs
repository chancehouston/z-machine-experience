//! The published specification and this implementation agree (SC-011, FR-029 to
//! FR-031): the same queries, the same version, the same reasons and field
//! names, and every section a test cites exists.

mod common;

use common::*;
use std::collections::BTreeSet;
use zmachine_introspect::json::{stop_line, tables};
use zmachine_introspect::{Introspector, QUERIES, SPEC_VERSION, Unknown};

fn specification() -> String {
    read_text("docs/introspection.md")
}

/// The numbered headings of the document: "4", "3.1", and so on, with titles.
fn sections(document: &str) -> Vec<(String, String)> {
    document
        .lines()
        .filter(|l| l.starts_with("## ") || l.starts_with("### "))
        .filter_map(|l| {
            let rest = l.trim_start_matches('#').trim();
            let (number, title) = rest.split_once(' ')?;
            let number = number.trim_end_matches('.');
            number.chars().all(|c| c.is_ascii_digit() || c == '.').then(|| (number.to_string(), title.to_string()))
        })
        .collect()
}

// SC-011, FR-030: the queries the document describes are the queries the
// implementation offers, in the same order.
#[test]
fn the_document_and_the_implementation_list_the_same_queries() {
    let document = specification();
    let described: Vec<String> = sections(&document)
        .into_iter()
        .filter(|(number, title)| !number.contains('.') && title.starts_with('`') && title.ends_with('`'))
        .map(|(_, title)| title.trim_matches('`').to_string())
        .collect();
    assert_eq!(described, QUERIES);
}

// FR-031: the document carries a version, and it is the one the implementation
// reports and `zm` prints.
#[test]
fn the_versions_agree() {
    let document = specification();
    let line = document.lines().find(|l| l.starts_with("**Version**:")).expect("a version line");
    assert!(line.contains(&format!("**Version**: {SPEC_VERSION} ")), "{line}");
    let machine = load(ZORK1.story, 1);
    let view = machine.view();
    let i = Introspector::new(&view);
    assert_eq!(i.spec_version(), SPEC_VERSION);
    assert!(stop_line(&i, &view, 1, "line").starts_with(&format!(r#"{{"spec":"{SPEC_VERSION}""#)));
    // Section 11 says what moves the version.
    let versioning = document.split("## 11. Versioning").nth(1).expect("section 11");
    assert!(versioning.contains("major") && versioning.contains("minor"), "section 11 says what changes which part");
}

/// Every field name in the JSON the implementation writes for both games, at
/// several stops, and for a story file of neither family.
fn written_fields() -> BTreeSet<String> {
    let mut fields = BTreeSet::new();
    let mut collect = |json: &str| {
        let mut rest = json;
        while let Some(at) = rest.find("\":") {
            let before = &rest[..at];
            if let Some(start) = before.rfind('"') {
                let key = &before[start + 1..];
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                    fields.insert(key.to_string());
                }
            }
            rest = &rest[at + 2..];
        }
    };
    for game in [ZORK1, ADVENT5] {
        walk(game, |machine, stop| {
            if stop.number % 40 == 1 {
                let view = machine.view();
                let i = Introspector::new(&view);
                collect(&stop_line(&i, &view, stop.number, "line"));
                collect(&tables(&i, &view, true, true));
            }
        });
    }
    let czech = load("stories/czech/czech.z5", 1);
    let view = czech.view();
    collect(&stop_line(&Introspector::new(&view), &view, 1, "line"));
    fields
}

// SC-011: every field the implementation writes is named in the document, and
// every reason for an unknown is listed there and nowhere invented.
#[test]
fn fields_and_reasons_agree() {
    let document = specification();
    let fields = written_fields();
    assert!(fields.len() >= 35, "{fields:?}");
    for field in &fields {
        assert!(document.contains(&format!("`{field}`")), "the document does not name the field `{field}`");
    }
    // The reasons, from the table in section 2.
    let section = document.split("## 2. Answers").nth(1).unwrap().split("\n## 3.").next().unwrap();
    let listed: BTreeSet<String> = section
        .lines()
        .filter(|l| l.starts_with("| `"))
        .filter_map(|l| l.split('`').nth(1))
        .filter(|word| word.contains('-') || *word == "inconsistent" || *word == "damaged")
        .map(str::to_string)
        .collect();
    let offered: BTreeSet<String> = Unknown::ALL.iter().map(|u| u.as_str().to_string()).collect();
    assert_eq!(listed, offered, "the reasons in section 2 against the implementation's");
}

// FR-040: every test that cites a section of the document ("I§") cites one that
// exists.
#[test]
fn every_cited_section_exists() {
    let numbers: BTreeSet<String> = sections(&specification()).into_iter().map(|(n, _)| n).collect();
    let mut cited = BTreeSet::new();
    let mut files = Vec::new();
    for directory in ["crates/zmachine-introspect/src", "crates/zmachine-introspect/tests", "crates/zmachine-introspect/tests/common", "crates/zmachine-cli/tests"] {
        for entry in std::fs::read_dir(format!("{ROOT}/{directory}")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
    }
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        for (at, _) in text.match_indices("I\u{a7}") {
            let number: String = text[at + 3..].chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            let number = number.trim_end_matches('.').to_string();
            assert!(numbers.contains(&number), "{}: I\u{a7}{number} is not a section of the document", path.display());
            cited.insert(number);
        }
    }
    // The query sections and the rules are all cited by some test.
    for needed in ["1.1", "1.6", "1.7", "2", "3.1", "3.2", "4", "5", "6", "7", "8", "9"] {
        assert!(cited.contains(needed), "no test cites I\u{a7}{needed}");
    }
}

// FR-026, FR-029: the document is for any implementer. It names no type of this
// implementation's language and points at no file of its source.
#[test]
fn the_document_does_not_depend_on_this_implementation() {
    let document = specification();
    for foreign in ["crates/", "Vec<", "Result<", "Option<", "&View", "::", "fn ", "impl ", "cargo "] {
        assert!(!document.contains(foreign), "the document contains {foreign:?}");
    }
    // Every numbered section has something in it.
    let all = sections(&document);
    assert!(all.len() >= 25, "{}", all.len());
    for pair in document.split("\n## ").skip(1) {
        let body: String = pair.lines().skip(1).collect::<Vec<_>>().join("\n");
        assert!(body.trim().len() > 80, "a section is nearly empty: {:?}", pair.lines().next());
    }
    assert!(!document.contains("This is a skeleton"), "the skeleton's note is gone");
}
