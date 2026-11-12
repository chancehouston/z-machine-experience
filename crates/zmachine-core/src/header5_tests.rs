//! Header rules for versions 5 and 8 (feature 002, T007).

use super::*;
use crate::version::Version;
use crate::{HeaderField, LoadError, LoadOptions, Machine, Settings};
use zmachine_testkit::{Story, op};

const OPTIONS: LoadOptions = LoadOptions {
    seed: Some(1),
    entropy: 0,
};

/// A version 3 image relabelled as `version`. Its tables are where a version 3
/// file has them, which is all these header tests need.
fn image(version: u8) -> Vec<u8> {
    let mut story = Story::new();
    story.main(&op::op0(op::QUIT));
    let mut image = story.build();
    image[0] = version;
    // Clear the bytes a version 3 testkit image leaves undefined for later versions.
    for byte in &mut image[0x1E..0x40] {
        *byte = 0;
    }
    image
}

fn put(image: &mut [u8], at: usize, value: u16) {
    image[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

// 002 FR-001, FR-002: versions 3, 5 and 8 load; any other is refused, and the
// message names the three that are supported.
#[test]
fn versions_5_and_8_load_and_others_are_refused_by_name() {
    assert_eq!(parse(&image(5)).unwrap().v, Version::V5);
    assert_eq!(parse(&image(8)).unwrap().v, Version::V8);
    for version in [1u8, 2, 4, 6, 7, 9] {
        let error = Machine::load(image(version), OPTIONS).unwrap_err();
        assert_eq!(error, LoadError::UnsupportedVersion(version));
        let message = error.to_string();
        assert!(message.contains(&format!("version {version}")), "{message}");
        assert!(message.contains("3, 5 and 8"), "{message}");
    }
}

// §11.1.6: the length field is in units of 4 bytes in version 5 and 8 in version 8.
#[test]
fn the_length_field_is_scaled_by_version() {
    for (version, times) in [(5u8, 4u32), (8, 8)] {
        let mut image = image(version);
        put(&mut image, 0x1A, 100);
        assert_eq!(parse(&image).unwrap().file_length, 100 * times);
    }
}

// §1.1.4: a version 5 file may be up to 256K and a version 8 file up to 512K.
// Version 3 files are not checked, as they never were.
#[test]
fn an_oversize_version_5_or_8_file_is_refused() {
    for (version, limit) in [(5u8, 256 * 1024usize), (8, 512 * 1024)] {
        let mut exact = image(version);
        exact.resize(limit, 0);
        assert!(parse(&exact).is_ok(), "version {version} at its limit");
        exact.push(0);
        assert_eq!(
            parse(&exact),
            Err(LoadError::TooLarge),
            "version {version} one byte over"
        );
    }
    let mut v3 = image(3);
    v3.resize(200 * 1024, 0);
    assert!(parse(&v3).is_ok(), "version 3 is not checked for size");
}

// §10.5.2.1, §3.5.5, §11.1.7: the terminating characters table (0x2E), the
// alphabet table (0x34) and the header extension table (0x36) are optional; one
// that is present must lie inside the file.
#[test]
fn optional_tables_are_read_and_must_lie_inside_the_file() {
    let mut image5 = image(5);
    let h = parse(&image5).unwrap();
    assert_eq!(
        (h.terminating_characters, h.alphabet_table, h.unicode_table),
        (0, 0, 0)
    );

    put(&mut image5, 0x2E, 0x0100);
    put(&mut image5, 0x34, 0x0120);
    let h = parse(&image5).unwrap();
    assert_eq!(
        (h.terminating_characters, h.alphabet_table),
        (0x0100, 0x0120)
    );

    let len = image5.len() as u16;
    for (offset, field) in [
        (0x2E, HeaderField::TerminatingCharacters),
        (0x34, HeaderField::AlphabetTable),
        (0x36, HeaderField::HeaderExtension),
    ] {
        let mut bad = image(5);
        put(&mut bad, offset, len);
        assert_eq!(
            parse(&bad),
            Err(LoadError::BadHeader(field)),
            "offset {offset:#x}"
        );
    }
}

// §3.8.5.2: the Unicode translation table is word 3 of the header extension table,
// when that table has at least three words.
#[test]
fn the_unicode_table_is_found_through_the_header_extension() {
    let mut image5 = image(5);
    let at = 0x02E0; // the testkit's scratch area
    put(&mut image5, 0x36, at as u16);
    put(&mut image5, at, 3);
    put(&mut image5, at + 6, 0x0300);
    assert_eq!(parse(&image5).unwrap().unicode_table, 0x0300);
    // With only two words there is no third to read.
    put(&mut image5, at, 2);
    assert_eq!(parse(&image5).unwrap().unicode_table, 0);
    // A table address outside the file is refused.
    put(&mut image5, at, 3);
    put(&mut image5, at + 6, 0xFFFE);
    assert_eq!(
        parse(&image5),
        Err(LoadError::BadHeader(HeaderField::UnicodeTable))
    );
}

// A version 3 file may hold anything at the offsets later versions use for tables.
#[test]
fn version_3_ignores_the_offsets_of_later_tables() {
    let mut v3 = image(3);
    for offset in [0x2E, 0x34, 0x36] {
        put(&mut v3, offset, 0xFFFF);
    }
    let h = parse(&v3).unwrap();
    assert_eq!(
        (h.terminating_characters, h.alphabet_table, h.unicode_table),
        (0, 0, 0)
    );
}

// data-model.md "What the interpreter writes into the header" (§11.1, §8.4.3).
#[test]
fn the_interpreter_writes_its_fields_for_versions_5_and_8() {
    for version in [5u8, 8] {
        let mut image = image(version);
        image[0x01] = 0b1110_0011; // everything the interpreter owns, wrong
        image[0x11] = 0b1011_1011; // Flags 2 low byte: all bits the game might set
        let settings = Settings {
            screen_columns: 60,
            screen_rows: 30,
            interpreter_number: 6,
            interpreter_version: b'Q',
            undo_slots: 25,
        };
        let m = Machine::load_with(image, OPTIONS, settings).unwrap();
        // Flags 1: colours (bit 0) and timed input (bit 7) clear; bold, italic and
        // fixed pitch (bits 2, 3, 4) set; bits the interpreter does not own kept.
        assert_eq!(m.peek(0x01), 0b0111_1110, "version {version} Flags 1");
        // Flags 2: pictures (3), mouse (5) and sound (7) cleared; transcript (0),
        // fixed pitch (1) and undo (4) left as the game had them.
        assert_eq!(m.peek(0x11), 0b0001_0011, "version {version} Flags 2");
        assert_eq!((m.peek(0x1E), m.peek(0x1F)), (6, b'Q'));
        assert_eq!((m.peek(0x20), m.peek(0x21)), (30, 60), "rows, then columns");
        assert_eq!((m.peek(0x22), m.peek(0x23)), (0, 60), "width in units");
        assert_eq!((m.peek(0x24), m.peek(0x25)), (0, 30), "height in units");
        assert_eq!((m.peek(0x26), m.peek(0x27)), (1, 1), "font size in units");
        assert_eq!((m.peek(0x32), m.peek(0x33)), (1, 1), "Standard 1.1");
    }
}

// contracts/core-api.md: the default settings.
#[test]
fn default_settings() {
    let m = Machine::load(image(5), OPTIONS).unwrap();
    assert_eq!((m.peek(0x1E), m.peek(0x1F)), (1, b'Z'));
    assert_eq!((m.peek(0x20), m.peek(0x21)), (24, 80));
    assert_eq!(Settings::default().undo_slots, 25);
}

// §6.1.4: an interpreter that cannot offer undo clears Flags 2 bit 4.
#[test]
fn undo_is_reported_unavailable_when_the_host_allows_none() {
    let mut image = image(5);
    image[0x11] = 0b0001_0000;
    let settings = Settings {
        undo_slots: 0,
        ..Settings::default()
    };
    assert_eq!(
        Machine::load_with(image, OPTIONS, settings)
            .unwrap()
            .peek(0x11),
        0
    );
}

// Guarantee 6: settings have no effect on a version 3 story file. Every header
// byte is what the default load writes.
#[test]
fn settings_do_not_touch_a_version_3_header() {
    let settings = Settings {
        screen_columns: 60,
        screen_rows: 30,
        interpreter_number: 6,
        interpreter_version: b'Q',
        undo_slots: 0,
    };
    let plain = Machine::load(image(3), OPTIONS).unwrap();
    let with = Machine::load_with(image(3), OPTIONS, settings).unwrap();
    for at in 0..64 {
        assert_eq!(plain.peek(at), with.peek(at), "header byte {at:#x}");
    }
}
