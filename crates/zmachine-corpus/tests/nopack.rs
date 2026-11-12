//! With packs set aside, nothing has changed (specs/006-game-packs/spec.md
//! FR-013, FR-014, FR-021; SC-005, SC-006; docs/introspection.md I§12.2, I§12.6).

mod common;

use common::*;
use zmachine_corpus::corpus::{Mode, command};
use zmachine_corpus::directions::directions;
use zmachine_corpus::trial::{LIMIT, trial};
use zmachine_corpus::walk::walkthrough;
use zmachine_introspect::Introspector;

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|l| l.to_string()).collect()
}

fn toy() -> Toy {
    let mut toy = Toy::new(2);
    toy.way(1, SOUTH, 0).flag_way(0, EAST, 1, 30, false);
    toy.door = Some((0, NORTH, 1));
    toy
}

fn repository(name: &str, with_pack: bool) -> Scratch {
    let scratch = Scratch::new(name);
    let image = toy().build();
    scratch.write("stories/toy/walk.txt", b"open\nlook\n");
    scratch.add("toy", &image, "walkthrough: stories/toy/walk.txt\n");
    if with_pack {
        let entries = format!("open-attribute: {OPEN_ATTR}\nuntrusted-variable: 46\n");
        scratch.pack("toy", &image, &entries);
    }
    scratch
}

// FR-021: a story with a pack keeps a second report, of the run with packs set
// aside, and that report is the report of the same story where no pack exists.
#[test]
fn the_report_without_the_pack_is_the_report_of_a_story_with_none() {
    let (with, without) = (repository("with", true), repository("without", false));
    for scratch in [&with, &without] {
        let (said, status) = command(&scratch.0, Mode::Write, &[]);
        assert_eq!(status, 0, "{said:?}");
    }
    let plain = without.read("stories/corpus/reports/toy.txt");
    assert_eq!(with.read("stories/corpus/reports/nopack/toy.txt"), plain);
    assert!(!plain.contains("pack"), "{plain}");
    let marked = with.read("stories/corpus/reports/toy.txt");
    assert!(marked.contains("\npack in-force entries 2\n"), "{marked}");
    assert_ne!(marked, plain);
    // A story with no pack keeps one report.
    assert!(!without.path("stories/corpus/reports/nopack/toy.txt").exists());
}

// FR-021: the run compares both reports with those kept.
#[test]
fn a_run_compares_the_report_without_the_pack_too() {
    let scratch = repository("compare", true);
    let _ = command(&scratch.0, Mode::Write, &[]);
    let proofs = [
        "toy entry open-attribute 9: proven",
        "toy entry untrusted-variable 46: proven",
    ];
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (lines(&[proofs[0], proofs[1], "toy ok"]), 0)
    );
    let kept = scratch.read("stories/corpus/reports/nopack/toy.txt");
    let altered = kept.replace("wordless 0", "wordless 1");
    assert_ne!(altered, kept);
    scratch.write("stories/corpus/reports/nopack/toy.txt", altered.as_bytes());
    assert_eq!(
        command(&scratch.0, Mode::Run, &[]),
        (
            lines(&[proofs[0], proofs[1], "toy differs without its pack"]),
            1
        )
    );
    std::fs::remove_file(scratch.path("stories/corpus/reports/nopack/toy.txt")).unwrap();
    assert_eq!(command(&scratch.0, Mode::Run, &[]).1, 1);
}

// SC-005: for each story that has a pack, the kept report without it has the
// verdict counts of feature 005's report: the same trials, the same confirmed,
// the same wrong (known and unexplained together, since entries a pack made
// needless have left the ledger), the same conditional.
#[test]
fn the_kept_reports_without_packs_have_feature_005s_figures() {
    // As `stories/corpus/reports/<id>.txt` stood when feature 005 was merged.
    let was = [(
        "zork1",
        "trials 4628 wordless 0",
        801,
        35,
        3591,
        66,
        "conditional 113 moved 89 stayed 24 question 0 ended 0 fault 0",
    )];
    let mut checked = 0;
    for (id, trials, fixed_confirmed, fixed_wrong, none_confirmed, none_wrong, conditional) in was
    {
        let Ok(text) = std::fs::read_to_string(repo(&format!(
            "stories/corpus/reports/nopack/{id}.txt"
        ))) else {
            continue;
        };
        let report = zmachine_corpus::report::Report::read(&text).unwrap();
        assert!(text.contains(&format!("\n{trials}\n")), "{id}");
        assert!(text.contains(&format!("\n{conditional}\n")), "{id}");
        assert_eq!(report.fixed_confirmed, fixed_confirmed, "{id}");
        assert_eq!(
            report.fixed_wrong_known + report.fixed_wrong_unexplained,
            fixed_wrong,
            "{id}"
        );
        assert_eq!(report.none_confirmed, none_confirmed, "{id}");
        assert_eq!(
            report.none_wrong_known + report.none_wrong_unexplained,
            none_wrong,
            "{id}"
        );
        assert_eq!(report.pack, zmachine_introspect::PackState::None, "{id}");
        checked += 1;
    }
    let packs = std::fs::read_dir(repo("packs"))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".pack"))
        .count();
    assert!(checked >= packs.min(1), "{checked} of {packs}");
}

// SC-006; FR-014: each walkthrough gives its reference transcript, byte for
// byte, with a trial of every direction run at every command prompt by an
// interface that holds the story's pack, where it has one.
#[test]
fn a_walkthrough_with_a_pack_gives_the_reference_transcript() {
    for game in GAMES {
        let pack = std::fs::read_to_string(repo(&format!("packs/{}.pack", game.name))).ok();
        let mut trials = 0;
        let mut tools = None;
        let replay = walkthrough(
            read(game.story),
            &read_text(game.walkthrough),
            game.seed,
            |driver, _| {
                let (introspector, found) = tools.get_or_insert_with(|| {
                    let view = driver.machine.view();
                    let introspector = match &pack {
                        Some(text) => Introspector::with_pack(&view, text),
                        None => Introspector::new(&view),
                    };
                    let found = directions(&introspector, &view);
                    (introspector, found)
                });
                let address = driver.command_address.unwrap();
                for direction in found.iter() {
                    if let Some(word) = &direction.word {
                        trial(&driver.machine, introspector, address, word, LIMIT);
                        trials += 1;
                    }
                }
            },
        );
        assert_eq!(replay.transcript, read_text(game.reference), "{}", game.name);
        assert!(trials > 1000, "{}: {trials}", game.name);
    }
}
