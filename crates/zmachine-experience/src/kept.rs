//! The kept game: what a host stores so that a game can be brought back
//! (specs/011-resume-game/contracts/kept-game.md, part K).
//!
//! It holds which story the game is of, the chance it was started with, and
//! every input the player gave that was answered. It holds no text of a game:
//! the transcript is there only as a digest.

use crate::{Seed, Source};
use zmachine_introspect::pack::sha256_hex;

/// The form's first line, less its number.
const NAME: &str = "lamplight-kept ";
/// The form this code writes and reads.
const FORM: &str = "1";
/// The largest fixed seed (K§3).
const MAX_FIXED: u32 = 2_147_483_647;

/// A kept game, read or about to be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Kept {
    /// The SHA-256 of the story file, lower-case hex.
    pub story: String,
    pub chance: Seed,
    /// The inputs answered, in order.
    pub inputs: Vec<(Source, String)>,
    /// The digest of the turns those inputs gave (K§5).
    pub transcript: String,
}

/// Why bytes are not a kept game this code reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unread {
    /// Cut short, altered, or never a kept game.
    Damaged,
    /// A kept game of a form this code does not know.
    OtherForm,
}

/// The SHA-256 of `bytes`, lower-case hex.
pub(crate) fn digest(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// The digest of a transcript: each turn's text followed by one zero byte, in
/// order (K§5).
pub(crate) fn transcript<'a>(texts: impl Iterator<Item = &'a str>) -> String {
    let mut all = Vec::new();
    for text in texts {
        all.extend_from_slice(text.as_bytes());
        all.push(0);
    }
    sha256_hex(&all)
}

fn is_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// A number written as this code writes one: digits only, no leading zero.
fn number(text: &str) -> Option<u32> {
    let plain = !text.is_empty()
        && text.bytes().all(|b| b.is_ascii_digit())
        && (text == "0" || !text.starts_with('0'));
    if plain { text.parse().ok() } else { None }
}

impl Kept {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = format!("{NAME}{FORM}\nstory {}\n", self.story);
        match self.chance {
            Seed::Fixed(n) => out.push_str(&format!("chance fixed {n}\n")),
            Seed::Fresh(n) => out.push_str(&format!("chance fresh {n}\n")),
        }
        out.push_str(&format!("inputs {}\n", self.inputs.len()));
        out.push_str(&format!("transcript {}\n", self.transcript));
        for (source, text) in &self.inputs {
            out.push_str(source.as_str());
            out.push(' ');
            out.push_str(text);
            out.push('\n');
        }
        let check = digest(out.as_bytes());
        out.push_str(&format!("check {check}\n"));
        out.into_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Kept, Unread> {
        let text = std::str::from_utf8(bytes).map_err(|_| Unread::Damaged)?;
        if text.contains('\r') {
            return Err(Unread::Damaged);
        }
        // The last line is the check of all before it (K§7); nothing follows.
        let at = text.rfind("\ncheck ").ok_or(Unread::Damaged)? + 1;
        let (body, last) = text.split_at(at);
        let given = last
            .strip_prefix("check ")
            .and_then(|rest| rest.strip_suffix('\n'))
            .ok_or(Unread::Damaged)?;
        if !is_digest(given) || given != digest(body.as_bytes()) {
            return Err(Unread::Damaged);
        }
        let mut lines = body.strip_suffix('\n').ok_or(Unread::Damaged)?.split('\n');
        let form = lines
            .next()
            .and_then(|line| line.strip_prefix(NAME))
            .ok_or(Unread::Damaged)?;
        if form != FORM {
            return Err(Unread::OtherForm);
        }
        let mut field = |name: &str| {
            lines
                .next()
                .and_then(|line| line.strip_prefix(name))
                .ok_or(Unread::Damaged)
        };
        let story = field("story ")?;
        let chance = field("chance ")?;
        let count = field("inputs ")?;
        let transcript = field("transcript ")?;
        if !is_digest(story) || !is_digest(transcript) {
            return Err(Unread::Damaged);
        }
        let chance = match chance.split_once(' ') {
            Some(("fixed", n)) => number(n).filter(|n| *n <= MAX_FIXED).map(Seed::Fixed),
            Some(("fresh", n)) => number(n).map(Seed::Fresh),
            _ => None,
        }
        .ok_or(Unread::Damaged)?;
        let count = number(count).ok_or(Unread::Damaged)? as usize;
        let mut inputs = Vec::new();
        for line in lines {
            let input = if let Some(text) = line.strip_prefix("typed ") {
                (Source::Typed, text.to_string())
            } else if let Some(text) = line.strip_prefix("tap ") {
                (Source::Tap, text.to_string())
            } else {
                return Err(Unread::Damaged);
            };
            inputs.push(input);
        }
        if inputs.len() != count {
            return Err(Unread::Damaged);
        }
        Ok(Kept {
            story: story.to_string(),
            chance,
            inputs,
            transcript: transcript.to_string(),
        })
    }
}
