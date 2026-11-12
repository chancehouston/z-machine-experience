//! The corpus program's commands, as functions the tests can call
//! (specs/005-corpus-testing/contracts/commands.md).

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::ledger::Ledger;
use crate::manifest::{self, Story};
use crate::report::Report;
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
    for story in chosen {
        let Some(fresh) = report(root, story, &ledger) else {
            lines.push(format!("{} skipped", story.id));
            continue;
        };
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
    (lines, failed as i32)
}
