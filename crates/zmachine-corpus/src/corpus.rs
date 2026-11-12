//! The corpus program's commands, as functions the tests can call
//! (specs/005-corpus-testing/contracts/commands.md).

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::ledger::Ledger;
use crate::manifest::{self, Story};
use crate::report::{Case, Report};
use crate::verdict::{Verdict, judge};
use crate::walk::run_story;

pub const MANIFEST: &str = "stories/corpus/manifest.txt";
pub const LEDGER: &str = "stories/corpus/ledger.txt";
pub const REPORTS: &str = "stories/corpus/reports";

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

/// Runs a present story and gives its report; `None` for an absent one.
pub fn report(root: &Path, story: &Story, ledger: &Ledger) -> Option<Report> {
    let bytes = present(root, story)?;
    let script = story
        .walkthrough
        .as_ref()
        .and_then(|path| std::fs::read_to_string(root.join(path)).ok());
    Some(run_story(story, bytes, script.as_deref(), ledger))
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
    let mut ran: Vec<&str> = Vec::new();
    for story in chosen {
        let Some(fresh) = report(root, story, &ledger) else {
            lines.push(format!("{} skipped", story.id));
            continue;
        };
        ran.push(&story.id);
        let wrong: Vec<&Case> = fresh
            .detail
            .iter()
            .map(|(case, _)| case)
            .filter(|case| judge(case) == Verdict::Wrong)
            .collect();
        for (entry, matched) in ledger.entries.iter().zip(&mut matched) {
            *matched |= entry.story == story.id && wrong.contains(&&entry.case);
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
        let path = root.join(report_path(&story.id));
        match mode {
            Mode::Write => match std::fs::write(&path, fresh.write()) {
                Ok(()) => lines.push(format!("{} written", story.id)),
                Err(e) => {
                    lines.push(format!("{}: {e}", report_path(&story.id)));
                    failed = true;
                }
            },
            Mode::Run => {
                let kept = std::fs::read_to_string(&path).unwrap_or_default();
                if kept == fresh.write() {
                    lines.push(format!("{} ok", story.id));
                } else {
                    lines.push(format!("{} differs", story.id));
                    failed = true;
                }
            }
        }
    }
    // An entry that no trial matched, for a story that ran, has outlived its case.
    if mode == Mode::Run {
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
    let script = story
        .walkthrough
        .as_ref()
        .and_then(|path| std::fs::read_to_string(root.join(path)).ok());
    let (_, _, shown) = crate::walk::run_story_watching(
        story,
        bytes,
        script.as_deref(),
        &ledger,
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
