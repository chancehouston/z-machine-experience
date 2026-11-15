//! A presentation file: how one story's map is drawn, authored by hand
//! (the constitution's Principle VII;
//! specs/017-authored-map/contracts/presentation.md, part F).
//!
//! The file says where a room sits on which sheet, and may name a sheet. It
//! holds nothing else: no word of a game, no way, no condition. Reading it
//! here proves only its form; whether it is for the story in hand, and
//! whether each place is a room of that story, is asked when a game is
//! opened with it.

use crate::map_place::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// The form of file this build reads.
const FORM: &str = "1";

/// The longest name a sheet may have, in characters.
const NAME: usize = 40;

/// The sheets a file may declare.
const SHEETS: std::ops::RangeInclusive<u8> = 1..=99;

/// How far from a sheet's middle a place may be put, either way.
const REACH: i32 = 99;

/// A presentation file, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Presentation {
    /// The story's SHA-256 digest, in lowercase hex.
    pub sha256: String,
    pub release: u16,
    pub serial: [u8; 6],
    /// Each sheet declared, with its name where it has one.
    pub sheets: BTreeMap<u8, Option<String>>,
    /// Each room placed: its sheet and its position there.
    pub places: BTreeMap<u16, (u8, Cell)>,
}

/// Why a file was refused, and the line that did it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Refused {
    pub line: usize,
    pub what: String,
}

impl Presentation {
    /// Reads a file, or refuses it whole (F§5).
    pub(crate) fn parse(text: &str) -> Result<Presentation, Refused> {
        let mut lines = text
            .lines()
            .enumerate()
            .map(|(n, line)| (n + 1, line.trim()))
            .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'));
        let mut last = 0;
        // F§1: the identity, in its order.
        let mut head = |key: &str| -> Result<(usize, String), Refused> {
            let Some((n, line)) = lines.next() else {
                return Err(refused(last + 1, format!("`{key}:` is missing")));
            };
            last = n;
            match line.split_once(": ") {
                Some((found, value)) if found == key => Ok((n, value.trim().to_string())),
                _ => Err(refused(n, format!("`{key}:` belongs here"))),
            }
        };
        let (n, form) = head("presentation")?;
        if form != FORM {
            return Err(refused(
                n,
                format!("form {form} is not read; this reads {FORM}"),
            ));
        }
        let (n, sha256) = head("sha256")?;
        if sha256.len() != 64
            || !sha256
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(refused(n, "a digest is 64 lowercase hex digits".into()));
        }
        let (n, release) = head("release")?;
        let release: u16 = release
            .parse()
            .map_err(|_| refused(n, "a release is a number".into()))?;
        let (n, serial) = head("serial")?;
        let serial: [u8; 6] = serial
            .as_bytes()
            .try_into()
            .ok()
            .filter(|s: &[u8; 6]| s.iter().all(u8::is_ascii_alphanumeric))
            .ok_or_else(|| refused(n, "a serial is six letters or digits".into()))?;

        let mut file = Presentation {
            sha256,
            release,
            serial,
            sheets: BTreeMap::new(),
            places: BTreeMap::new(),
        };
        let mut taken: BTreeSet<(u8, Cell)> = BTreeSet::new();
        for (n, line) in lines {
            match line
                .split_once(": ")
                .or_else(|| line.strip_suffix(':').map(|key| (key, "")))
            {
                Some(("sheet", rest)) => {
                    // F§2.
                    let rest = rest.trim();
                    let (number, name) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
                    let number =
                        sheet(number).ok_or_else(|| refused(n, "a sheet is 1 to 99".into()))?;
                    let name = name.trim();
                    if name.chars().count() > NAME {
                        return Err(refused(n, format!("a name is at most {NAME} characters")));
                    }
                    let name = (!name.is_empty()).then(|| name.to_string());
                    if file.sheets.insert(number, name).is_some() {
                        return Err(refused(n, format!("sheet {number} is given twice")));
                    }
                }
                Some(("place", rest)) => {
                    // F§3, F§4.
                    let fields: Vec<&str> = rest.split_whitespace().collect();
                    let [room, on, x, y] = fields[..] else {
                        return Err(refused(n, "a place is ROOM SHEET X Y".into()));
                    };
                    let room = room
                        .parse::<u16>()
                        .ok()
                        .filter(|room| *room > 0)
                        .ok_or_else(|| refused(n, "a room is a number from 1".into()))?;
                    let on = sheet(on)
                        .filter(|on| file.sheets.contains_key(on))
                        .ok_or_else(|| refused(n, "the sheet is not declared above".into()))?;
                    let along = |field: &str| {
                        field
                            .parse::<i32>()
                            .ok()
                            .filter(|v| (-REACH..=REACH).contains(v))
                            .ok_or_else(|| refused(n, format!("a position is -{REACH} to {REACH}")))
                    };
                    let cell = (along(x)?, along(y)?);
                    if !taken.insert((on, cell)) {
                        return Err(refused(n, "another place is in this position".into()));
                    }
                    if file.places.insert(room, (on, cell)).is_some() {
                        return Err(refused(n, format!("room {room} is given twice")));
                    }
                }
                _ => return Err(refused(n, "neither `sheet:` nor `place:`".into())),
            }
        }
        Ok(file)
    }
}

fn sheet(field: &str) -> Option<u8> {
    field.parse::<u8>().ok().filter(|n| SHEETS.contains(n))
}

fn refused(line: usize, what: String) -> Refused {
    Refused { line, what }
}
