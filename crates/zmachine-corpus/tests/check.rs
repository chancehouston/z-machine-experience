//! The manifest check (specs/005-corpus-testing/contracts/formats.md, "The
//! manifest check"; spec.md FR-016 to FR-019; User Story 4).

mod common;

use common::*;
use zmachine_corpus::corpus::{Mode, command, corpus_lines};
use zmachine_corpus::manifest::check;

fn toy() -> Vec<u8> {
    let mut toy = Toy::new(2);
    toy.way(0, NORTH, 1).way(1, SOUTH, 0);
    toy.build()
}

/// A scratch repository with one committed story, well recorded.
fn good() -> Scratch {
    let scratch = Scratch::new("check");
    scratch.add("toy", &toy(), "");
    scratch
}

/// The failures of the check, which must be exactly one, naming `what`.
fn fails_with(scratch: &Scratch, what: &str) {
    let failures = check(&scratch.0);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].contains(what), "{what:?} in {failures:?}");
    assert!(
        failures[0].starts_with("stories/corpus/manifest.txt:"),
        "{failures:?}"
    );
}

fn edit(scratch: &Scratch, file: &str, from: &str, to: &str) {
    let text = scratch.read(file);
    assert!(text.contains(from), "{from:?} in {file}");
    scratch.write(file, text.replace(from, to).as_bytes());
}

const MANIFEST: &str = "stories/corpus/manifest.txt";

#[test]
fn a_well_recorded_story_passes() {
    assert_eq!(check(&good().0), Vec::<String>::new());
}

// FR-017: a story is known by its digest; an id or a digest may not be shared.
#[test]
fn two_entries_may_not_share_an_id_or_a_digest() {
    let scratch = good();
    let entry = scratch.read(MANIFEST);
    scratch.write(MANIFEST, format!("{entry}{entry}").as_bytes());
    let failures = check(&scratch.0);
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert!(
        failures[0].contains("`toy` is the id of another entry"),
        "{failures:?}"
    );
    assert!(
        failures[1].contains("digest of another entry"),
        "{failures:?}"
    );
    assert!(
        failures[0].starts_with("stories/corpus/manifest.txt:12: "),
        "{failures:?}"
    );
}

// FR-018: a committed story is there, is the file the manifest describes, and
// has its source, digest and license recorded beside it.
#[test]
fn a_committed_story_must_be_there_and_be_what_the_manifest_says() {
    let scratch = good();
    std::fs::remove_file(scratch.path("stories/toy/toy.z3")).unwrap();
    fails_with(&scratch, "the file is absent");

    let scratch = good();
    scratch.write("stories/toy/toy.z3", &Toy::new(3).build());
    fails_with(&scratch, "the file's digest is");

    let scratch = good();
    edit(&scratch, MANIFEST, "size: ", "size: 1");
    fails_with(&scratch, "the file's size is");

    let scratch = good();
    edit(&scratch, MANIFEST, "version: 3", "version: 5");
    fails_with(&scratch, "the file's version is 3");

    let scratch = good();
    edit(&scratch, MANIFEST, "family: zil", "family: inform");
    fails_with(&scratch, "introspection finds the family zil");
}

#[test]
fn a_committed_story_must_have_its_record_and_license_beside_it() {
    let scratch = good();
    std::fs::remove_file(scratch.path("stories/toy/SOURCE.md")).unwrap();
    fails_with(&scratch, "no SOURCE.md beside the file");

    let scratch = good();
    scratch.write("stories/toy/SOURCE.md", b"The digest is not here.\n");
    fails_with(&scratch, "SOURCE.md does not hold the digest");

    let scratch = good();
    std::fs::remove_file(scratch.path("stories/toy/LICENSE")).unwrap();
    fails_with(&scratch, "no license text beside the file");
    // A file whose name starts COPYING is a license text too.
    scratch.write("stories/toy/COPYING-toy.txt", b"A test's own.\n");
    assert_eq!(check(&scratch.0), Vec::<String>::new());
}

/// A fetched story's entry, for a file under `stories/local/corpus/`.
fn fetched(scratch: &Scratch, image: &[u8], version: u8, family: &str) {
    let digest = zmachine_corpus::corpus::sha256(image);
    let entry = format!(
        "id: far\nsha256: {digest}\nsize: {}\nversion: {version}\nfamily: {family}\n\
         source: https://example.org/far.z3\nstanding: personal use only\ntier: fetched\n\
         path: stories/local/corpus/far.z3\nseed: 1\n\n",
        image.len()
    );
    let manifest = scratch.read(MANIFEST) + &entry;
    scratch.write(MANIFEST, manifest.as_bytes());
}

// FR-021; US4 scenario 1: an absent fetched story is no failure of the check,
// and its test is ignored, not passed.
#[test]
fn an_absent_fetched_story_is_skipped_everywhere() {
    let scratch = good();
    fetched(&scratch, &Toy::new(4).build(), 3, "zil");
    assert_eq!(check(&scratch.0), Vec::<String>::new());
    command(&scratch.0, Mode::Write, &[]);
    assert!(!scratch.path("stories/corpus/reports/far.txt").exists());
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]).0,
        ["toy ok", "far skipped"]
    );
    let (lines, passed) = corpus_lines(&scratch.0);
    assert_eq!(
        lines,
        [
            "test toy ... ok",
            "test far ... ignored, story absent",
            "",
            "test result: ok. 1 passed; 0 failed; 1 ignored",
        ]
    );
    assert!(passed);
}

// US4 scenarios 2 and 3: a present fetched story runs; a file whose digest the
// manifest does not have is not used.
#[test]
fn a_present_fetched_story_runs_and_a_stray_file_is_not_used() {
    let scratch = good();
    let image = Toy::new(4).build();
    fetched(&scratch, &image, 3, "zil");
    // Some other file where the story should be: not the story.
    scratch.write("stories/local/corpus/far.z3", &Toy::new(5).build());
    scratch.write("stories/local/corpus/stray.z3", &Toy::new(6).build());
    assert_eq!(check(&scratch.0), Vec::<String>::new());
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]).0,
        ["toy written", "far skipped"]
    );
    // The story itself.
    scratch.write("stories/local/corpus/far.z3", &image);
    assert_eq!(check(&scratch.0), Vec::<String>::new());
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]).0,
        ["toy written", "far written"]
    );
    assert_eq!(
        corpus_lines(&scratch.0).0[..2],
        ["test toy ... ok", "test far ... ok"]
    );
}

// F§ "The manifest check": a present fetched story is what the manifest says.
#[test]
fn a_present_fetched_story_must_be_what_the_manifest_says() {
    let scratch = good();
    let image = Toy::new(4).build();
    fetched(&scratch, &image, 5, "zil");
    scratch.write("stories/local/corpus/far.z3", &image);
    fails_with(&scratch, "the file's version is 3");

    let scratch = good();
    fetched(&scratch, &image, 3, "none");
    scratch.write("stories/local/corpus/far.z3", &image);
    fails_with(&scratch, "introspection finds the family zil");
}

// The corpus as a test run fails when a story does.
#[test]
fn a_story_that_differs_fails_the_corpus_run() {
    let scratch = good();
    command(&scratch.0, Mode::Write, &[]);
    edit(
        &scratch,
        "stories/corpus/reports/toy.txt",
        "places 2 ",
        "places 3 ",
    );
    let (lines, passed) = corpus_lines(&scratch.0);
    assert!(!passed);
    assert_eq!(lines[0], "test toy ... FAILED");
    assert_eq!(lines[1], "    toy differs");
    assert_eq!(
        lines.last().unwrap(),
        "test result: FAILED. 0 passed; 1 failed; 0 ignored"
    );
}

// The repository's own manifest and ledger are well formed, and every committed
// story is recorded as the constitution requires.
#[test]
fn the_repositorys_manifest_passes() {
    assert_eq!(check(std::path::Path::new(ROOT)), Vec::<String>::new());
}
