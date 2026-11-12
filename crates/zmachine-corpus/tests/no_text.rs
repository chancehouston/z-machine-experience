//! No game's text in what the repository keeps of the corpus
//! (specs/005-corpus-testing/research.md R9; spec.md FR-026, SC-006).

mod common;

use common::*;
use zmachine_corpus::corpus::{LEDGER, MANIFEST, REPORTS, present};
use zmachine_corpus::manifest::{self, Tier};
use zmachine_corpus::report::{Report, Summary};
use zmachine_introspect::Introspector;

/// The words of the report format. A game's word that is one of these cannot be
/// told from the format's own.
const KEYWORDS: [&str; 32] = [
    "story",
    "sha256",
    "version",
    "family",
    "zil",
    "inform",
    "none",
    "seed",
    "places",
    "walkthrough",
    "crawl",
    "rooms",
    "visited",
    "named-not-reached",
    "budget",
    "trials",
    "reached",
    "no",
    "wordless",
    "fixed",
    "confirmed",
    "wrong-known",
    "wrong-unexplained",
    "other",
    "conditional",
    "moved",
    "stayed",
    "question",
    "ended",
    "fault",
    "unknown",
    "no-verdict",
];

fn report_files() -> Vec<std::path::PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(repo(REPORTS))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "txt"))
        .collect();
    files.sort();
    files
}

// R9, by grammar: every report reads as the format gives it, which has no free
// text; and what is left of it once the format's own words, numbers, digests
// and ids are taken away is nothing.
#[test]
fn every_report_is_of_the_format_and_nothing_else() {
    let files = report_files();
    assert!(!files.is_empty());
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        if path.file_name().unwrap() == "summary.txt" {
            // F§ "The summary": its lines are ids, families and figures.
            let summary = Summary::read(&text).unwrap_or_else(|e| panic!("summary: {e:?}"));
            assert_eq!(summary.write(), text);
            continue;
        }
        let report = Report::read(&text).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()));
        assert_eq!(report.write(), text, "{}", path.display());
        for word in text.split_whitespace() {
            let plain = KEYWORDS.contains(&word)
                || matches!(word, "detail" | "lost")
                || word == report.id
                || word == report.sha256
                || word.bytes().all(|b| b.is_ascii_digit());
            assert!(plain, "{}: {word:?}", path.display());
        }
    }
}

// SC-006, by search: for each committed story, none of its objects' names and
// none of its dictionary's words of five letters or more is a word of its
// report; and none of its objects' names of two words or more is anywhere in
// the ledger.
#[test]
fn no_committed_storys_names_are_in_its_report_or_the_ledger() {
    let root = std::path::Path::new(ROOT);
    let stories = manifest::read(&read_text(MANIFEST)).unwrap();
    let ledger = read_text(LEDGER).to_lowercase();
    let mut searched = 0;
    for story in stories.iter().filter(|s| s.tier == Tier::Committed) {
        let bytes = present(root, story).expect("a committed story is present");
        let machine = load_bytes(bytes, story.seed);
        let view = machine.view();
        let introspector = Introspector::new(&view);
        let mut names: Vec<String> = Vec::new();
        let mut phrases: Vec<String> = Vec::new();
        for object in introspector.object_tree(&view).value.unwrap_or_default() {
            let Ok(name) = object.printed_name else {
                continue;
            };
            let name = name.to_lowercase();
            if name.split_whitespace().count() >= 2 {
                phrases.push(name.clone());
            }
            names.extend(name.split_whitespace().map(str::to_string));
        }
        for word in introspector.dictionary(&view).value.unwrap_or_default() {
            if word.word.chars().count() >= 5 {
                names.push(word.word.to_lowercase());
            }
        }
        names.retain(|name| !KEYWORDS.contains(&name.as_str()) && name.len() >= 3);
        assert!(names.len() > 100, "{}: {} names", story.id, names.len());
        assert!(
            phrases.len() > 20,
            "{}: {} phrases",
            story.id,
            phrases.len()
        );

        // The story's own report, and the summary, whose words besides the
        // format's are the stories' ids and figures written `8/11`.
        let report = read_text(&format!("{REPORTS}/{}.txt", story.id)).to_lowercase()
            + &read_text(&format!("{REPORTS}/summary.txt")).to_lowercase();
        for word in report.split_whitespace() {
            if stories.iter().any(|s| s.id == word) || matches!(word, "all" | "stories" | "skipped")
            {
                continue;
            }
            assert!(
                !names.iter().any(|name| name == word),
                "{}: the report holds {word:?}, a word of the game",
                story.id
            );
        }
        for phrase in &phrases {
            assert!(
                !ledger.contains(phrase.as_str()),
                "the ledger holds {phrase:?}, a name from {}",
                story.id
            );
        }
        searched += 1;
    }
    assert!(searched >= 3);
}
