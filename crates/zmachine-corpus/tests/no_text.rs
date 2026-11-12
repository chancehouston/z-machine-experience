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
const KEYWORDS: [&str; 52] = [
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
    // Feature 006: a run that was given a game pack.
    "pack",
    "in-force",
    "not-matching",
    "refused",
    "entries",
    "open",
    "closed",
    "distrusted",
    // Feature 007: the comparison of the inventory with the game's listing.
    "inventory",
    "compared",
    "agreed",
    "differed-known",
    "differed-unexplained",
    "unjudged",
    "no-listing",
    "empty",
    "through-closed",
    "seen",
    "kept",
    "listed",
];

fn report_files() -> Vec<std::path::PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(repo(REPORTS))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .chain(
            // The reports of runs with packs set aside (feature 006).
            std::fs::read_dir(repo(zmachine_corpus::corpus::NOPACK))
                .into_iter()
                .flatten()
                .map(|entry| entry.unwrap().path()),
        )
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
                || matches!(word, "detail" | "lost" | "unlisted")
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

// ---------------------------------------------------------------------------
// Game packs (specs/006-game-packs/research.md R13; FR-003; SC-008)
// ---------------------------------------------------------------------------

/// The words a pack's comment may use, besides numbers.
const COMMENT_WORDS: [&str; 6] = ["a", "game", "pack", "docs", "introspection", "section"];

/// Why a pack's text is not of the format's grammar, read by this test's own
/// reader and not the crate's: the line, from 1, and the fault. A line is blank,
/// a comment, or a known key and a value of digits, of hexadecimal digits, or
/// for the serial of six letters or digits. A comment may hold no word of
/// `names`, and no word at all but the format's own.
fn pack_fault(text: &str, names: &[String]) -> Option<(usize, String)> {
    for (number, line) in text.lines().enumerate().map(|(i, l)| (i + 1, l)) {
        let fault = |why: &str| Some((number, why.to_string()));
        if line.trim().is_empty() {
            continue;
        }
        if let Some(comment) = line.strip_prefix('#') {
            for word in comment
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|w| !w.is_empty())
            {
                let word = word.to_lowercase();
                if names.contains(&word) {
                    return fault(&format!("the comment holds {word:?}, a word of a game"));
                }
                let plain = word.bytes().all(|b| b.is_ascii_digit())
                    || COMMENT_WORDS.contains(&word.as_str())
                    || word == "md";
                if !plain {
                    return fault(&format!("the comment holds {word:?}"));
                }
            }
            continue;
        }
        let Some((key, value)) = line.split_once(": ") else {
            return fault("not a key and a value");
        };
        let digits = !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit());
        let good = match key {
            "pack"
            | "release"
            | "open-attribute"
            | "untrusted-variable"
            | "transparent-attribute" => digits,
            "sha256" => {
                value.len() == 64
                    && value
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }
            "serial" => value.len() == 6 && value.bytes().all(|b| b.is_ascii_alphanumeric()),
            _ => return fault("a key the format does not have"),
        };
        if !good {
            return fault("a value that is not a number, a digest or a serial");
        }
    }
    None
}

/// Every word of four letters or more in a committed story's dictionary, and
/// every word of its objects' names.
fn game_words() -> Vec<String> {
    let root = std::path::Path::new(ROOT);
    let stories = manifest::read(&read_text(MANIFEST)).unwrap();
    let mut names = Vec::new();
    for story in stories.iter().filter(|s| s.tier == Tier::Committed) {
        let machine = load_bytes(present(root, story).unwrap(), story.seed);
        let view = machine.view();
        let introspector = Introspector::new(&view);
        for object in introspector.object_tree(&view).value.unwrap_or_default() {
            if let Ok(name) = object.printed_name {
                names.extend(name.to_lowercase().split_whitespace().map(str::to_string));
            }
        }
        for word in introspector.dictionary(&view).value.unwrap_or_default() {
            if word.word.chars().count() >= 4 {
                names.push(word.word.to_lowercase());
            }
        }
    }
    names.retain(|name| !COMMENT_WORDS.contains(&name.as_str()) && name.len() >= 4);
    assert!(names.len() > 1000, "{}", names.len());
    names
}

// I§12.1: "A pack has nowhere to hold a condition, a script, or any text of a
// game." SC-008, FR-003: every pack the repository holds is of the grammar and
// nothing else, and its comments hold no word of any committed story.
#[test]
fn no_pack_holds_a_word_of_a_game() {
    let names = game_words();
    let mut read = 0;
    for entry in std::fs::read_dir(repo("packs")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "pack") {
            let text = std::fs::read_to_string(&path).unwrap();
            assert_eq!(pack_fault(&text, &names), None, "{}", path.display());
            read += 1;
        }
    }
    assert!(read >= 1, "the repository holds a pack");

    // The reader refuses what it is there to refuse.
    let good = "# A game pack (docs/introspection.md, section 12).\npack: 1\nopen-attribute: 11\n";
    assert_eq!(pack_fault(good, &names), None);
    for (bad, line) in [
        ("# The trap door of the kitchen.\npack: 1\n", 1),
        ("pack: 1\n# Zork\n", 2),
        ("pack: 1\nopen-attribute: window\n", 2),
        ("pack: 1\nname: zork\n", 2),
        ("pack: 1\nopen-attribute: 11 if the lamp is lit\n", 2),
        ("pack: 1\nserial: 88 429\n", 2),
        ("pack: 2\ntransparent-attribute: glass\n", 2),
    ] {
        assert_eq!(pack_fault(bad, &names).map(|f| f.0), Some(line), "{bad}");
    }
    assert!(names.contains(&"zork".to_string()) && names.contains(&"window".to_string()));
}
