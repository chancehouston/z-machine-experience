//! The story file header (§11).

use crate::{HeaderField, LoadError};

/// Size of the header, and so the smallest possible dynamic memory (§1.1, §11).
pub(crate) const HEADER_LEN: usize = 64;

const FLAGS1: usize = 0x01;
const STANDARD_REVISION: usize = 0x32;

/// The header fields this interpreter uses, as read from the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Header {
    pub version: u8,
    pub release: u16,
    pub serial: [u8; 6],
    pub checksum: u16,
    pub high_base: u16,
    pub initial_pc: u16,
    pub dictionary: u16,
    pub objects: u16,
    pub globals: u16,
    pub static_base: u16,
    pub abbreviations: u16,
    /// Length of the file as the header states it, in bytes. Used by `verify` only.
    pub file_length: u32,
}

/// Reads and validates the header of a version 3 story file.
///
/// The stated length and checksum are not checked here: they belong to the game's
/// `verify` instruction (§15), and a file padded or cut by a transfer program is
/// still loadable.
pub(crate) fn parse(story: &[u8]) -> Result<Header, LoadError> {
    if story.len() < HEADER_LEN {
        return Err(LoadError::TooShort);
    }
    if story[0] != 3 {
        return Err(LoadError::UnsupportedVersion(story[0]));
    }
    let word = |at: usize| u16::from_be_bytes([story[at], story[at + 1]]);
    let mut serial = [0u8; 6];
    serial.copy_from_slice(&story[0x12..0x18]);
    let header = Header {
        version: story[0],
        release: word(0x02),
        serial,
        checksum: word(0x1C),
        high_base: word(0x04),
        initial_pc: word(0x06),
        dictionary: word(0x08),
        objects: word(0x0A),
        globals: word(0x0C),
        static_base: word(0x0E),
        abbreviations: word(0x18),
        // §11.1.6: versions 1 to 3 store the length divided by 2.
        file_length: word(0x1A) as u32 * 2,
    };

    let inside = |value: u16, field: HeaderField| {
        if (value as usize) < story.len() {
            Ok(())
        } else {
            Err(LoadError::BadHeader(field))
        }
    };
    inside(header.high_base, HeaderField::HighMemoryBase)?;
    inside(header.initial_pc, HeaderField::InitialProgramCounter)?;
    inside(header.dictionary, HeaderField::Dictionary)?;
    inside(header.objects, HeaderField::ObjectTable)?;
    inside(header.globals, HeaderField::Globals)?;
    inside(header.static_base, HeaderField::StaticMemoryBase)?;
    inside(header.abbreviations, HeaderField::Abbreviations)?;
    if (header.static_base as usize) < HEADER_LEN {
        return Err(LoadError::BadHeader(HeaderField::StaticMemoryBase));
    }
    Ok(header)
}

/// Writes the header fields that belong to the interpreter (§11.1). Called at load
/// and again after every restart and restore (§6.1.2, §6.1.3).
///
/// In Flags 1 for versions 1 to 3 the interpreter owns three bits:
/// bit 4, "status line not available", cleared because the host is given one;
/// bit 5, "screen splitting available", cleared;
/// bit 6, "variable-pitch font is the default", cleared.
/// Bytes 0x32 and 0x33 hold the revision of the Standard followed: 1.1.
pub(crate) fn apply_interpreter_fields(dynamic: &mut [u8]) {
    dynamic[FLAGS1] &= !0b0111_0000;
    dynamic[STANDARD_REVISION] = 1;
    dynamic[STANDARD_REVISION + 1] = 1;
}

#[cfg(test)]
#[path = "header_tests.rs"]
mod tests;
