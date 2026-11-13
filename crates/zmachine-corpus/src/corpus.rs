//! The corpus program's commands, as functions the tests can call
//! (specs/005-corpus-testing/contracts/commands.md).

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::ledger::Ledger;
use crate::manifest::{self, Story};
use crate::packs::{self, Proof};
use crate::report::{Case, Report, StoryLine, Summary};
use crate::verdict::{Verdict, judge};
use crate::walk::run_story_with;

pub const MANIFEST: &str = "stories/corpus/manifest.txt";
pub const LEDGER: &str = "stories/corpus/ledger.txt";
pub const REPORTS: &str = "stories/corpus/reports";
pub const SUMMARY: &str = "stories/corpus/reports/summary.txt";
/// Where the report of a run with packs set aside is kept, for a story that has
/// a pack (specs/006-game-packs/research.md R11).
pub const NOPACK: &str = "stories/corpus/reports/nopack";

/// What `corpus` was asked to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Run,
    Write,
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The story's file, if it is present: there, and with the manifest's digest
/// (data-model.md "Story").
pub fn present(root: &Path, story: &Story) -> Option<Vec<u8>> {
    let bytes = std::fs::read(root.join(&story.path)).ok()?;
    (sha256(&bytes) == story.sha256).then_some(bytes)
}

/// The manifest and the ledger, or the first place either departs from its
/// format, as `file:line: why`.
pub fn load(root: &Path) -> Result<(Vec<Story>, Ledger), String> {
    let read =
        |file: &str| std::fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"));
    let stories = manifest::read(&read(MANIFEST)?)
        .map_err(|(line, why)| format!("{MANIFEST}:{line}: {why}"))?;
    let ledger = crate::ledger::read(&read(LEDGER)?)
        .map_err(|(line, why)| format!("{LEDGER}:{line}: {why}"))?;
    Ok((stories, ledger))
}

/// Runs a present story with its pack, if it has one, and gives its report;
/// `None` for an absent one.
pub fn report(root: &Path, story: &Story, ledger: &Ledger) -> Option<Report> {
    let pack = packs::of(root, story);
    report_with(root, story, ledger, pack.as_ref().map(|p| p.text.as_str()))
}

/// Runs a present story with this pack's text, or with none.
pub fn report_with(
    root: &Path,
    story: &Story,
    ledger: &Ledger,
    pack: Option<&str>,
) -> Option<Report> {
    let bytes = present(root, story)?;
    let script = script(root, story);
    Some(run_story_with(story, bytes, script.as_deref(), ledger, pack, None).0)
}

/// What a story is walked by before the crawl: its walkthrough, or the
/// manifest's opening answers and first commands, one to a line (research R12).
pub fn script(root: &Path, story: &Story) -> Option<String> {
    if let Some(path) = &story.walkthrough {
        return std::fs::read_to_string(root.join(path)).ok();
    }
    if story.first.is_empty() {
        return None;
    }
    let lines: Vec<&str> = story
        .opening
        .iter()
        .chain(&story.first)
        .map(String::as_str)
        .collect();
    Some(lines.join("\n") + "\n")
}

pub fn nopack_path(id: &str) -> String {
    format!("{NOPACK}/{id}.txt")
}

/// The proof of each entry of a story's pack, as `(key and value, outcome)`:
/// `with` is the run with the whole pack (research R11).
pub fn proofs(
    root: &Path,
    story: &Story,
    ledger: &Ledger,
    found: &packs::Found,
    with: &Report,
) -> Vec<(String, Proof)> {
    let mut out = Vec::new();
    if let Some(attribute) = found.pack.open_attribute {
        out.push((
            format!("open-attribute {attribute}"),
            packs::prove_open(with),
        ));
    }
    if let Some(attribute) = found.pack.transparent_attribute {
        out.push((
            format!("transparent-attribute {attribute}"),
            packs::prove_transparent(with),
        ));
    }
    for &variable in &found.pack.untrusted_variables {
        let rest = packs::text(&found.pack, Some(variable));
        let proof = match report_with(root, story, ledger, Some(&rest)) {
            Some(without) => packs::prove_variable(with, &without),
            None => Proof::NotProven,
        };
        out.push((format!("untrusted-variable {variable}"), proof));
    }
    out
}

pub fn report_path(id: &str) -> String {
    format!("{REPORTS}/{id}.txt")
}

/// Runs a command from the repository's root. Returns the lines to print and
/// the exit status.
pub fn command(root: &Path, mode: Mode, ids: &[String]) -> (Vec<String>, i32) {
    let (stories, ledger) = match load(root) {
        Ok(loaded) => loaded,
        Err(why) => return (vec![why], 1),
    };
    if let Some(line) = ledger
        .inventory
        .iter()
        .find(|e| !stories.iter().any(|s| s.id == e.story))
        .map(|e| e.line)
    {
        let why = format!("{LEDGER}:{line}: the entry's story is not in the manifest");
        return (vec![why], 1);
    }
    if let Some(entry) = ledger
        .entries
        .iter()
        .find(|e| !stories.iter().any(|s| s.id == e.story))
    {
        let why = format!(
            "{LEDGER}:{}: `{}` is not in the manifest",
            entry.line, entry.story
        );
        return (vec![why], 1);
    }
    let (bad_packs, _) = packs::check(root, &stories, &ledger);
    if !bad_packs.is_empty() {
        return (bad_packs, 1);
    }
    let unknown: Vec<String> = ids
        .iter()
        .filter(|id| !stories.iter().any(|s| &s.id == *id))
        .map(|id| format!("{id} is not in the manifest"))
        .collect();
    if !unknown.is_empty() {
        return (unknown, 2);
    }
    let chosen = stories
        .iter()
        .filter(|s| ids.is_empty() || ids.contains(&s.id));
    let mut lines = Vec::new();
    let mut failed = false;
    // The ledger's entries that a trial of this run matched.
    let mut matched = vec![false; ledger.entries.len()];
    let mut matched_inventory = vec![false; ledger.inventory.len()];
    let mut ran: Vec<&str> = Vec::new();
    // The summary kept, and the lines of this run's stories.
    let kept_read = std::fs::read_to_string(root.join(SUMMARY))
        .ok()
        .and_then(|text| Summary::read(&text).ok());
    // A kept summary that does not read is caught whole, below.
    let kept_reads = kept_read.is_some();
    let kept = kept_read.unwrap_or_default();
    let mut fresh_lines: Vec<StoryLine> = Vec::new();
    for story in chosen {
        let Some(fresh) = report(root, story, &ledger) else {
            lines.push(format!("{} skipped", story.id));
            continue;
        };
        ran.push(&story.id);
        fresh_lines.push(fresh.line());
        // A story with a pack: each entry is proven or the run fails, and the
        // story is run again with packs set aside.
        let found = packs::of(root, story);
        let bare = found
            .as_ref()
            .and_then(|_| report_with(root, story, &ledger, None));
        let mut unproven = false;
        if let Some(found) = &found {
            for (entry, proof) in proofs(root, story, &ledger, found, &fresh) {
                lines.push(format!("{} entry {entry}: {}", story.id, proof.as_str()));
                unproven |= mode == Mode::Run && proof != Proof::Proven;
            }
        }
        failed |= unproven;
        let wrong: Vec<&Case> = fresh
            .detail
            .iter()
            .map(|(case, _)| case)
            .filter(|case| judge(case) == Verdict::Wrong)
            .collect();
        for (entry, matched) in ledger.entries.iter().zip(&mut matched) {
            *matched |= entry.story == story.id && wrong.contains(&&entry.case);
        }
        for (entry, matched) in ledger.inventory.iter().zip(&mut matched_inventory) {
            *matched |= entry.story == story.id
                && fresh
                    .inventory_detail
                    .iter()
                    .any(|(difference, _)| *difference == entry.difference);
        }
        // A difference between what is seen of the inventory and what the game
        // lists, with no cause written, fails as a wrong exit does.
        let differing: Vec<String> = fresh
            .inventory_detail
            .iter()
            .filter(|(difference, _)| !ledger.knows_inventory(&story.id, difference))
            .map(|(d, _)| {
                format!(
                    "{} inventory wrong {} {} {} {}",
                    story.id,
                    d.room,
                    d.object,
                    if d.seen { "seen" } else { "kept" },
                    if d.listed { "listed" } else { "unlisted" }
                )
            })
            .collect();
        if mode == Mode::Run && !differing.is_empty() {
            lines.extend(differing);
            failed = true;
            continue;
        }
        let unexplained: Vec<&&Case> = wrong
            .iter()
            .filter(|case| !ledger.knows(&story.id, case))
            .collect();
        if mode == Mode::Run && !unexplained.is_empty() {
            for case in unexplained {
                lines.push(format!("{} wrong {}", story.id, case.text()));
            }
            failed = true;
            continue;
        }
        if mode == Mode::Run && unproven {
            continue;
        }
        let path = root.join(report_path(&story.id));
        let bare_path = root.join(nopack_path(&story.id));
        match mode {
            Mode::Write => {
                let mut wrote = std::fs::write(&path, fresh.write());
                if let (Ok(()), Some(bare)) = (&wrote, &bare) {
                    wrote = std::fs::create_dir_all(root.join(NOPACK))
                        .and_then(|()| std::fs::write(&bare_path, bare.write()));
                }
                match wrote {
                    Ok(()) => lines.push(format!("{} written", story.id)),
                    Err(e) => {
                        lines.push(format!("{}: {e}", report_path(&story.id)));
                        failed = true;
                    }
                }
            }
            Mode::Run => {
                let summary_has = |line: &StoryLine| !kept_reads || kept.stories.contains(line);
                let kept = std::fs::read_to_string(&path).unwrap_or_default();
                let line = fresh.line();
                let bare_kept = bare.as_ref().is_none_or(|bare| {
                    std::fs::read_to_string(&bare_path).ok() == Some(bare.write())
                });
                if kept == fresh.write() && !bare_kept {
                    lines.push(format!("{} differs without its pack", story.id));
                    failed = true;
                } else if kept == fresh.write() {
                    lines.push(format!("{} ok", story.id));
                    if !summary_has(&line) {
                        lines.push(format!("{}: its line of the summary differs", story.id));
                        failed = true;
                    }
                } else {
                    lines.push(format!("{} differs", story.id));
                    failed = true;
                }
            }
        }
    }
    // The summary: this run's stories by their fresh lines, the others by the
    // lines kept; a story with neither is listed as skipped.
    let summary = Summary {
        stories: stories
            .iter()
            .filter_map(|story| {
                fresh_lines
                    .iter()
                    .chain(&kept.stories)
                    .find(|line| line.id == story.id)
                    .cloned()
            })
            .collect(),
        skipped: stories
            .iter()
            .filter(|story| {
                !fresh_lines
                    .iter()
                    .chain(&kept.stories)
                    .any(|line| line.id == story.id)
            })
            .map(|story| story.id.clone())
            .collect(),
    };
    let all_ran = ran.len() == stories.len();
    match mode {
        Mode::Write => {
            if let Err(e) = std::fs::write(root.join(SUMMARY), summary.write()) {
                lines.push(format!("{SUMMARY}: {e}"));
                failed = true;
            }
        }
        // With every story run, the whole summary must be the one kept.
        Mode::Run if all_ran => {
            let text = std::fs::read_to_string(root.join(SUMMARY)).unwrap_or_default();
            if text != summary.write() {
                lines.push(format!("{SUMMARY} differs"));
                failed = true;
            }
        }
        Mode::Run => {}
    }
    // An entry that no trial matched, for a story that ran, has outlived its case.
    if mode == Mode::Run {
        for (entry, matched) in ledger.inventory.iter().zip(matched_inventory) {
            if !matched && ran.contains(&entry.story.as_str()) {
                lines.push(format!(
                    "{LEDGER}:{}: the entry matches no comparison",
                    entry.line
                ));
                failed = true;
            }
        }
        for (entry, matched) in ledger.entries.iter().zip(matched) {
            if !matched && ran.contains(&entry.story.as_str()) {
                lines.push(format!(
                    "{LEDGER}:{}: the entry matches no trial",
                    entry.line
                ));
                failed = true;
            }
        }
    }
    (lines, failed as i32)
}

/// `corpus explain ID ROOM DIRECTION`: every trial of the story from that room in
/// that direction, with what the game printed. For finding a wrong case's cause.
/// What it prints is a game's text: it is for the terminal, never for a file the
/// repository tracks.
pub fn explain(root: &Path, id: &str, room: u16, direction: u8) -> (Vec<String>, i32) {
    let (stories, ledger) = match load(root) {
        Ok(loaded) => loaded,
        Err(why) => return (vec![why], 1),
    };
    let Some(story) = stories.iter().find(|s| s.id == id) else {
        return (vec![format!("{id} is not in the manifest")], 2);
    };
    let Some(bytes) = present(root, story) else {
        return (vec![format!("{id} skipped")], 0);
    };
    let script = script(root, story);
    let pack = packs::of(root, story);
    let (_, _, shown) = run_story_with(
        story,
        bytes,
        script.as_deref(),
        &ledger,
        pack.as_ref().map(|p| p.text.as_str()),
        Some((room, direction)),
    );
    let mut lines = Vec::new();
    for trial in shown {
        let case = Case {
            room,
            direction,
            reported: trial.reported,
            outcome: trial.outcome,
        };
        lines.push(format!(
            "{}: > {}    [{}]",
            trial.origin,
            trial.word,
            case.text()
        ));
        for line in trial.text.trim().lines() {
            lines.push(format!("    {line}"));
        }
    }
    if lines.is_empty() {
        lines.push(format!(
            "no trial of {id} started in room {room} in direction {direction}"
        ));
    }
    (lines, 0)
}

/// The corpus as a test run: one line for each story of the manifest, in the
/// form the standard test runner uses, then the line of totals. Returns the
/// lines and whether every story that ran passed (research R16). An absent
/// story is ignored, which is not passed.
pub fn corpus_lines(root: &Path) -> (Vec<String>, bool) {
    let stories = match load(root) {
        Ok((stories, _)) => stories,
        Err(why) => return (vec![why, "test result: FAILED.".to_string()], false),
    };
    let mut lines = Vec::new();
    let (mut passed, mut failed, mut ignored) = (0, 0, 0);
    // Each pack is a test too: of its entries' proofs, which its story's run
    // makes. A pack whose story is absent is ignored, which is not passed.
    let mut pack_lines = Vec::new();
    for story in &stories {
        let (said, status) = command(root, Mode::Run, std::slice::from_ref(&story.id));
        let has_pack = packs::of(root, story).is_some();
        let outcome = if said == [format!("{} skipped", story.id)] {
            ignored += 1 + has_pack as usize;
            "ignored, story absent"
        } else if status == 0 {
            passed += 1 + has_pack as usize;
            "ok"
        } else {
            failed += 1;
            "FAILED"
        };
        lines.push(format!("test {} ... {outcome}", story.id));
        if outcome == "FAILED" {
            lines.extend(said.iter().map(|line| format!("    {line}")));
        }
        if has_pack {
            let proven = !said
                .iter()
                .any(|l| l.contains(" entry ") && !l.ends_with(": proven"));
            let outcome = match outcome {
                "FAILED" if proven => "ok",
                other => other,
            };
            if outcome == "FAILED" {
                failed += 1;
            } else if status != 0 {
                passed += 1;
            }
            pack_lines.push(format!("test pack {} ... {outcome}", story.id));
        }
    }
    lines.extend(pack_lines);
    let result = if failed == 0 { "ok" } else { "FAILED" };
    lines.push(String::new());
    lines.push(format!(
        "test result: {result}. {passed} passed; {failed} failed; {ignored} ignored"
    ));
    (lines, failed == 0)
}
