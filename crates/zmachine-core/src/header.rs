//! The story file header (§11).

use crate::version::Version;
use crate::{HeaderField, LoadError, Settings};

/// Size of the header, and so the smallest possible dynamic memory (§1.1, §11).
pub(crate) const HEADER_LEN: usize = 64;

const FLAGS1: usize = 0x01;
const FLAGS2_LOW: usize = 0x11;
const INTERPRETER_NUMBER: usize = 0x1E;
const SCREEN_ROWS: usize = 0x20;
const STANDARD_REVISION: usize = 0x32;

/// The header fields this interpreter uses, as read from the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Header {
    pub version: u8,
    /// The same, as the type that answers what differs between versions.
    pub v: Version,
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
    /// Versions 5 and 8: the table of keys that end a line of input, or 0 (§10.5.2.1).
    pub terminating_characters: u16,
    /// Versions 5 and 8: the game's own alphabet table, or 0 (§3.5.5).
    pub alphabet_table: u16,
    /// Versions 5 and 8: the game's own table of extra characters, or 0 (§3.8.5.2).
    pub unicode_table: u16,
}

/// Reads and validates the header of a story file of version 3, 5 or 8.
///
/// The stated length and checksum are not checked here: they belong to the game's
/// `verify` instruction (§15), and a file padded or cut by a transfer program is
/// still loadable.
pub(crate) fn parse(story: &[u8]) -> Result<Header, LoadError> {
    if story.len() < HEADER_LEN {
        return Err(LoadError::TooShort);
    }
    let v = Version::from_byte(story[0]).ok_or(LoadError::UnsupportedVersion(story[0]))?;
    // §1.1.4. Version 3 files were never checked for size and still are not.
    if !v.is_v3() && story.len() > v.max_story_size() {
        return Err(LoadError::TooLarge);
    }
    let word = |at: usize| u16::from_be_bytes([story[at], story[at + 1]]);
    // The tables that exist only from version 5 are read only from version 5: a
    // version 3 file may hold anything at those offsets.
    let later = |at: usize| if v.is_v3() { 0 } else { word(at) };
    let extension = later(0x36);
    // §11.1.7: the header extension table's first word is how many words follow;
    // the Unicode translation table is its third (§3.8.5.2).
    let unicode_table = if extension != 0
        && (extension as usize + 7) < story.len()
        && word(extension as usize) >= 3
    {
        word(extension as usize + 6)
    } else {
        0
    };
    let mut serial = [0u8; 6];
    serial.copy_from_slice(&story[0x12..0x18]);
    let header = Header {
        version: story[0],
        v,
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
        // §11.1.6: the stored length is divided by 2, 4 or 8 according to version.
        file_length: word(0x1A) as u32 * v.length_multiplier(),
        terminating_characters: later(0x2E),
        alphabet_table: later(0x34),
        unicode_table,
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
    // Each optional table is absent (0) or inside the file.
    inside(
        header.terminating_characters,
        HeaderField::TerminatingCharacters,
    )?;
    inside(header.alphabet_table, HeaderField::AlphabetTable)?;
    inside(extension, HeaderField::HeaderExtension)?;
    inside(header.unicode_table, HeaderField::UnicodeTable)?;
    Ok(header)
}

/// Writes the header fields that belong to the interpreter (§11.1). Called at load
/// and again after every restart, restore and undo (§6.1.2.2, §6.1.3).
///
/// Version 3, Flags 1: the interpreter owns three bits.
/// Bit 4, "status line not available", is cleared because the host is given one;
/// bit 5, "screen splitting available", is cleared;
/// bit 6, "variable-pitch font is the default", is cleared.
///
/// Versions 5 and 8, Flags 1: colours (bit 0) and timed input (bit 7) are reported
/// unavailable; bold (bit 2), italic (bit 3) and fixed pitch (bit 4) available.
/// Flags 2: pictures (bit 3), mouse (bit 5) and sound (bit 7) are cleared, and undo
/// (bit 4) is cleared if the host allows none (§6.1.4). The interpreter's number and
/// version letter and the screen size come from the host's settings; the screen is
/// reported in characters and again in units, with a font of 1 by 1 (§8.4.3).
///
/// Bytes 0x32 and 0x33 hold the revision of the Standard followed: 1.1.
pub(crate) fn apply_interpreter_fields(dynamic: &mut [u8], header: &Header, settings: &Settings) {
    if header.v.is_v3() {
        dynamic[FLAGS1] &= !0b0111_0000;
    } else {
        dynamic[FLAGS1] = dynamic[FLAGS1] & !0b1000_0001 | 0b0001_1100;
        dynamic[FLAGS2_LOW] &= !0b1010_1000;
        if settings.undo_slots == 0 {
            dynamic[FLAGS2_LOW] &= !0b0001_0000;
        }
        dynamic[INTERPRETER_NUMBER] = settings.interpreter_number;
        dynamic[INTERPRETER_NUMBER + 1] = settings.interpreter_version;
        let (rows, columns) = (settings.screen_rows, settings.screen_columns);
        dynamic[SCREEN_ROWS] = rows.min(255) as u8;
        dynamic[SCREEN_ROWS + 1] = columns.min(255) as u8;
        dynamic[0x22..0x24].copy_from_slice(&columns.to_be_bytes());
        dynamic[0x24..0x26].copy_from_slice(&rows.to_be_bytes());
        dynamic[0x26] = 1;
        dynamic[0x27] = 1;
    }
    dynamic[STANDARD_REVISION] = 1;
    dynamic[STANDARD_REVISION + 1] = 1;
}

#[cfg(test)]
#[path = "header_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "header5_tests.rs"]
mod tests5;
