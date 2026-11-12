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

// F§ "The summary"; FR-025, FR-027, FR-028; US5 scenarios 1 to 3.
#[test]
fn the_summary_is_kept_and_checked_with_the_reports() {
    const SUMMARY: &str = "stories/corpus/reports/summary.txt";
    let scratch = good();
    let image = Toy::new(4).build();
    fetched(&scratch, &image, 3, "zil");
    scratch.write("stories/local/corpus/far.z3", &image);
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]).0,
        ["toy written", "far written"]
    );
    let kept = scratch.read(SUMMARY);
    let lines: Vec<&str> = kept.lines().collect();
    assert!(
        lines[0].starts_with("story toy zil places 2 rooms 2 trials 12 fixed 2/2 "),
        "{kept}"
    );
    assert!(
        lines[1].starts_with("story far zil places 1 rooms 1 "),
        "{kept}"
    );
    assert!(
        lines[2].starts_with("family zil stories 2 places 3 rooms 3 "),
        "{kept}"
    );
    assert!(
        lines[5].starts_with("all stories 2 places 3 rooms 3 "),
        "{kept}"
    );
    assert_eq!(lines.len(), 6);
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (vec!["toy ok".into(), "far ok".into()], 0)
    );

    // With every story present, the whole summary must be the one kept.
    scratch.write(
        SUMMARY,
        kept.replace("all stories 2", "all stories 3").as_bytes(),
    );
    let (said, status) = command(&scratch.0, Mode::Run, &[]);
    assert_eq!(status, 1);
    assert_eq!(
        said.last().unwrap(),
        "stories/corpus/reports/summary.txt differs"
    );

    // With a story absent, each present story is checked against its own line.
    let mut altered = zmachine_corpus::report::Summary::read(&kept).unwrap();
    altered.stories[0].figures.places += 1;
    scratch.write(SUMMARY, altered.write().as_bytes());
    std::fs::rename(
        scratch.path("stories/local/corpus/far.z3"),
        scratch.path("stories/local/corpus/far.aside"),
    )
    .unwrap();
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (
            vec![
                "toy ok".into(),
                "toy: its line of the summary differs".into(),
                "far skipped".into()
            ],
            1
        )
    );
    std::fs::rename(
        scratch.path("stories/local/corpus/far.aside"),
        scratch.path("stories/local/corpus/far.z3"),
    )
    .unwrap();

    // With a story absent, its kept line stays, the present stories are checked
    // against theirs, and writing leaves the absent story's report and line alone.
    scratch.write(SUMMARY, kept.as_bytes());
    std::fs::remove_file(scratch.path("stories/local/corpus/far.z3")).unwrap();
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (vec!["toy ok".into(), "far skipped".into()], 0)
    );
    assert_eq!(
        command(&scratch.0, Mode::Write, &[]).0,
        ["toy written", "far skipped"]
    );
    assert_eq!(scratch.read(SUMMARY), kept);
    assert!(scratch.path("stories/corpus/reports/far.txt").exists());
}

// F§ "The summary": a story that has never run is listed as skipped and is in
// no total.
#[test]
fn a_story_that_has_never_run_is_listed_as_skipped() {
    let scratch = good();
    fetched(&scratch, &Toy::new(4).build(), 3, "zil");
    command(&scratch.0, Mode::Write, &[]);
    let kept = scratch.read("stories/corpus/reports/summary.txt");
    assert!(kept.contains("\nall stories 1 places 2 "), "{kept}");
    assert!(kept.ends_with("skipped far\n"), "{kept}");
}

// ---------------------------------------------------------------------------
// Game packs (specs/006-game-packs/research.md R9; FR-005, FR-023)
// ---------------------------------------------------------------------------

/// The failures of the pack check, which must be exactly one, naming `what`.
fn pack_fails_with(scratch: &Scratch, what: &str) {
    let (stories, ledger) = zmachine_corpus::corpus::load(&scratch.0).unwrap();
    let (failures, _) = zmachine_corpus::packs::check(&scratch.0, &stories, &ledger);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].contains(what), "{what:?} in {failures:?}");
    // Nothing is run on a pack that does not pass.
    assert_eq!(command(&scratch.0, Mode::Run, &[]), (failures, 1));
}

fn pack_passes(scratch: &Scratch) {
    let (stories, ledger) = zmachine_corpus::corpus::load(&scratch.0).unwrap();
    let (failures, _) = zmachine_corpus::packs::check(&scratch.0, &stories, &ledger);
    assert_eq!(failures, Vec::<String>::new());
}

// FR-023: a repository with no pack passes, and so does a well-formed pack for
// a story of the manifest, named by the story's id.
#[test]
fn a_well_formed_pack_for_a_story_of_the_manifest_passes() {
    let scratch = good();
    pack_passes(&scratch);
    scratch.pack("toy", &toy(), "open-attribute: 9\n");
    pack_passes(&scratch);
    // A file that is not a pack is not read as one.
    scratch.write("packs/README.md", b"Packs.\n");
    pack_passes(&scratch);
}

// FR-005: a malformed pack is refused as a whole, with the file and the line.
#[test]
fn a_malformed_pack_is_refused_with_its_file_and_line() {
    let scratch = good();
    scratch.pack("toy", &toy(), "open-attribute: 9\nlit-attribute: 3\n");
    pack_fails_with(
        &scratch,
        "packs/toy.pack:6: a key this format does not have",
    );
    scratch.pack("toy", &toy(), "open-attribute: 32\n");
    pack_fails_with(
        &scratch,
        "packs/toy.pack:5: open-attribute is a number from 0 to 31",
    );
}

// FR-023: an entry with no story to test it on may not exist; nor two packs for
// one story; and a pack is named by its story's id.
#[test]
fn a_pack_must_be_the_one_pack_of_a_story_the_corpus_holds() {
    let scratch = good();
    let mut other = Toy::new(3);
    other.way(0, NORTH, 1);
    scratch.pack("toy", &other.build(), "");
    pack_fails_with(
        &scratch,
        "packs/toy.pack: its digest is no story's in the manifest",
    );

    let scratch = good();
    scratch.pack("toy", &toy(), "");
    scratch.pack("toy-again", &toy(), "");
    let (stories, ledger) = zmachine_corpus::corpus::load(&scratch.0).unwrap();
    let (failures, _) = zmachine_corpus::packs::check(&scratch.0, &stories, &ledger);
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert!(
        failures[0].contains("packs/toy-again.pack: another pack names the same story"),
        "{failures:?}"
    );
    assert!(
        failures[1].contains("packs/toy-again.pack: its story is `toy`"),
        "{failures:?}"
    );
}

// FR-023: both kinds of entry are for the ZIL family. A pack with none is no
// fault for any family.
#[test]
fn an_entry_for_a_story_of_another_family_is_refused() {
    let scratch = good();
    edit(&scratch, MANIFEST, "family: zil", "family: inform");
    scratch.pack("toy", &toy(), "");
    pack_passes(&scratch);
    scratch.pack("toy", &toy(), "untrusted-variable: 40\n");
    pack_fails_with(
        &scratch,
        "packs/toy.pack: an entry, for a story of family inform",
    );
}

// I§12.2: the release and the serial are the story file's own; where the file
// is present a pack that states others is refused.
#[test]
fn a_pack_states_its_storys_own_release_and_serial() {
    let scratch = good();
    scratch.pack("toy", &toy(), "");
    edit(&scratch, "packs/toy.pack", "release: 1\n", "release: 2\n");
    pack_fails_with(&scratch, "packs/toy.pack: the story's release is 1");
    scratch.pack("toy", &toy(), "");
    edit(
        &scratch,
        "packs/toy.pack",
        "serial: 000000\n",
        "serial: 000001\n",
    );
    pack_fails_with(&scratch, "packs/toy.pack: the story's serial is 000000");
    // Where the file is absent these cannot be checked, and are not.
    std::fs::remove_file(scratch.path("stories/toy/toy.z3")).unwrap();
    pack_passes(&scratch);
}

// FR-019: the check shows each ledger entry on an answer that rests on a pack,
// for a person to read its cause. It is a note, not a failure.
#[test]
fn the_check_shows_ledger_entries_on_marked_answers() {
    let scratch = good();
    scratch.write(
        "stories/corpus/ledger.txt",
        b"story: toy\nroom: 2\ndirection: 31\nreported: fixed 3\noutcome: stayed\ncause: One.\n\n\
          story: toy\nroom: 3\ndirection: 29\nreported: open 2\noutcome: stayed\ncause: The room is dark.\n",
    );
    let (stories, ledger) = zmachine_corpus::corpus::load(&scratch.0).unwrap();
    let (failures, notes) = zmachine_corpus::packs::check(&scratch.0, &stories, &ledger);
    assert_eq!(failures, Vec::<String>::new());
    assert_eq!(
        notes,
        [
            "stories/corpus/ledger.txt:8: on an answer that rests on a pack: toy 3 29 open 2 stayed: The room is dark."
        ]
    );
}

// The packs the repository holds pass the check.
#[test]
fn the_repositorys_packs_pass() {
    let root = std::path::Path::new(ROOT);
    let (stories, ledger) = zmachine_corpus::corpus::load(root).unwrap();
    let (failures, _) = zmachine_corpus::packs::check(root, &stories, &ledger);
    assert_eq!(failures, Vec::<String>::new());
}
