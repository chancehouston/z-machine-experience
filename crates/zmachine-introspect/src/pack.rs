//! Game packs: the file, and a story file's identity (docs/introspection.md,
//! section 12).

use zmachine_core::View;

/// A game pack as its file states it (specification section 12.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pack {
    /// The story file's SHA-256, as 64 lowercase hexadecimal digits.
    pub sha256: String,
    pub release: u16,
    pub serial: [u8; 6],
    /// The attribute set on a door while it is open.
    pub open_attribute: Option<u8>,
    /// Variables whose stored value is not to be relied on, in the file's order.
    pub untrusted_variables: Vec<u8>,
    /// The format's version the file states: 1 or 2.
    pub version: u8,
    /// The attribute set on a thing whose contents can be seen while it is
    /// closed (specification section 12.8).
    pub transparent_attribute: Option<u8>,
    /// The attribute set on a room that is lit of itself and on a thing that
    /// gives light (specification section 12.9).
    pub lit_attribute: Option<u8>,
}

/// Why a pack was refused: the line, counted from 1, and the fault.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackError {
    pub line: usize,
    pub what: &'static str,
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.what)
    }
}

/// What became of the pack a reader was given (specification section 12.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PackState {
    #[default]
    None,
    InForce,
    NotMatching,
    Refused,
}

impl PackState {
    pub fn as_str(self) -> &'static str {
        match self {
            PackState::None => "none",
            PackState::InForce => "in-force",
            PackState::NotMatching => "not-matching",
            PackState::Refused => "refused",
        }
    }
}

/// The facts of a pack in force, as the adapters use them. Empty when no pack is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Entries {
    pub open_attribute: Option<u8>,
    pub untrusted_variables: Vec<u8>,
    pub transparent_attribute: Option<u8>,
    pub lit_attribute: Option<u8>,
}

/// A decimal number as the format writes one: digits only, no sign, and no
/// leading zero but `0` itself.
fn decimal(value: &str, least: u32, most: u32) -> Option<u32> {
    let plain = !value.is_empty()
        && value.len() <= 5
        && value.bytes().all(|b| b.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'));
    let number: u32 = plain.then(|| value.parse().ok()).flatten()?;
    (least..=most).contains(&number).then_some(number)
}

impl Pack {
    /// Reads a pack's text (specification section 12.1). Any fault refuses the
    /// whole pack.
    pub fn parse(text: &str) -> Result<Pack, PackError> {
        const IDENTITY: [&str; 4] = ["pack", "sha256", "release", "serial"];
        let mut pack = Pack {
            sha256: String::new(),
            release: 0,
            serial: [0; 6],
            open_attribute: None,
            untrusted_variables: Vec::new(),
            version: 1,
            transparent_attribute: None,
            lit_attribute: None,
        };
        // How many lines that are not skipped have been read.
        let mut read = 0;
        for (index, line) in text.split('\n').enumerate() {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.bytes().all(|b| b == b' ') || line.starts_with('#') {
                continue;
            }
            let fault = |what| PackError {
                line: index + 1,
                what,
            };
            let (key, value) = line
                .split_once(": ")
                .ok_or_else(|| fault("not a key, a colon, a space and a value"))?;
            if IDENTITY.get(read).is_some_and(|expected| key != *expected) {
                return Err(fault(
                    "the first four lines are pack, sha256, release and serial, in that order",
                ));
            }
            read += 1;
            match key {
                "pack" if read == 1 => {
                    pack.version = match value {
                        "1" => 1,
                        "2" => 2,
                        "3" => 3,
                        _ => {
                            return Err(fault("a version of the format this reader does not know"));
                        }
                    };
                }
                "sha256" if read == 2 => {
                    let hexadecimal = value.len() == 64
                        && value
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
                    if !hexadecimal {
                        return Err(fault("sha256 is 64 of the characters 0 to 9 and a to f"));
                    }
                    pack.sha256 = value.to_string();
                }
                "release" if read == 3 => {
                    pack.release = decimal(value, 0, 65535)
                        .ok_or_else(|| fault("release is a number from 0 to 65535"))?
                        as u16;
                }
                "serial" if read == 4 => {
                    let bytes = value.as_bytes();
                    if bytes.len() != 6 || !bytes.iter().all(u8::is_ascii_alphanumeric) {
                        return Err(fault("serial is six letters or digits"));
                    }
                    pack.serial.copy_from_slice(bytes);
                }
                "open-attribute" => {
                    let attribute = decimal(value, 0, 31)
                        .ok_or_else(|| fault("open-attribute is a number from 0 to 31"))?;
                    if pack.open_attribute.replace(attribute as u8).is_some() {
                        return Err(fault("open-attribute may be given once"));
                    }
                }
                "untrusted-variable" => {
                    let variable = decimal(value, 16, 255)
                        .ok_or_else(|| fault("untrusted-variable is a number from 16 to 255"))?
                        as u8;
                    if pack.untrusted_variables.contains(&variable) {
                        return Err(fault("the same variable twice"));
                    }
                    pack.untrusted_variables.push(variable);
                }
                "transparent-attribute" if pack.version >= 2 => {
                    let attribute = decimal(value, 0, 31)
                        .ok_or_else(|| fault("transparent-attribute is a number from 0 to 31"))?;
                    if pack
                        .transparent_attribute
                        .replace(attribute as u8)
                        .is_some()
                    {
                        return Err(fault("transparent-attribute may be given once"));
                    }
                }
                "transparent-attribute" => {
                    return Err(fault("transparent-attribute needs a pack of version 2"));
                }
                "lit-attribute" if pack.version >= 3 => {
                    let attribute = decimal(value, 0, 31)
                        .ok_or_else(|| fault("lit-attribute is a number from 0 to 31"))?;
                    if pack.lit_attribute.replace(attribute as u8).is_some() {
                        return Err(fault("lit-attribute may be given once"));
                    }
                }
                "lit-attribute" => {
                    return Err(fault("lit-attribute needs a pack of version 3"));
                }
                _ => return Err(fault("a key this format does not have")),
            }
        }
        if read < IDENTITY.len() {
            return Err(PackError {
                line: text.lines().count() + 1,
                what: "the first four lines are pack, sha256, release and serial, in that order",
            });
        }
        Ok(pack)
    }

    /// Whether the pack states any fact.
    pub fn has_entries(&self) -> bool {
        self.open_attribute.is_some()
            || !self.untrusted_variables.is_empty()
            || self.transparent_attribute.is_some()
            || self.lit_attribute.is_some()
    }

    pub(crate) fn entries(&self) -> Entries {
        Entries {
            open_attribute: self.open_attribute,
            untrusted_variables: self.untrusted_variables.clone(),
            transparent_attribute: self.transparent_attribute,
            lit_attribute: self.lit_attribute,
        }
    }
}

/// Whether a pack's identity is that of the story file as shipped: its SHA-256,
/// its release and its serial, all three (specification section 12.2).
pub(crate) fn matches(pack: &Pack, view: &View<'_>) -> bool {
    use sha2::{Digest, Sha256};
    let serial: Option<Vec<u8>> = (0x12..0x18).map(|at| view.shipped_byte(at)).collect();
    if view.shipped_word(2) != Some(pack.release) || serial.as_deref() != Some(&pack.serial[..]) {
        return false;
    }
    let shipped: Vec<u8> = (0..view.len())
        .filter_map(|at| view.shipped_byte(at))
        .collect();
    let digest: String = Sha256::digest(&shipped)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    shipped.len() == view.len() && digest == pack.sha256
}
