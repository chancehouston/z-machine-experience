//! Text for versions 5 and 8 (feature 002, T010).

use super::*;
use crate::header;
use crate::memory::Memory;
use crate::objects::Objects;
use zmachine_testkit::{Story5, op, zencode, zencode_dict9};

fn advent(version: u8) -> (Memory, header::Header) {
    let path = format!(
        "{}/../../stories/advent/advent.z{version}",
        env!("CARGO_MANIFEST_DIR")
    );
    let story = std::fs::read(path).unwrap();
    let h = header::parse(&story).unwrap();
    (Memory::new(story, h.static_base as usize), h)
}

// §13.4: version 5 dictionary words are nine Z-characters in six bytes. The testkit
// encoder is checked against Advent's own dictionary.
#[test]
fn nine_z_character_encoding() {
    let c = Charset::default();
    for word in [
        "a",
        "lantern",
        "inventory",
        "building",
        "xyzzy",
        "1",
        "can't",
        "x@y",
        "",
    ] {
        assert_eq!(
            c.encode(word.as_bytes(), 9),
            zencode_dict9(word),
            "{word:?}"
        );
    }
    assert_eq!(c.encode(b"lanternfish", 9), c.encode(b"lanternfi", 9));
    assert_eq!(
        c.encode(b"hello", 9),
        vec![0x35, 0x51, 0x46, 0x85, 0x94, 0xA5]
    );
    // Six Z-characters, as version 3 does, is the same function with another count.
    assert_eq!(c.encode(b"hello", 6), vec![0x35, 0x51, 0xC6, 0x85]);
}

// §3.3: abbreviation entries are word addresses in every version; they do not take
// the packed-address multiplier of §1.2.3.
#[test]
fn abbreviations_are_word_addresses_in_versions_5_and_8() {
    for version in [5u8, 8] {
        let mut story = Story5::new(version);
        story.abbreviation(0, "the ").abbreviation(33, "xyzzy");
        // Z-characters 1,0 (abbreviation 0), 2,1 (abbreviation 33), then padding.
        let words = [1u16 << 10 | 2, 0x8000 | 1 << 10 | 5 << 5 | 5];
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_be_bytes()).collect();
        let at = story.raw_string(&bytes) as u32 * story.packed_multiplier() as u32;
        story.main(&op::op0(op::QUIT));
        let image = story.build();
        let h = header::parse(&image).unwrap();
        let m = Memory::new(image, h.static_base as usize);
        let (text, _) = Charset::default().decode(&m, at, h.abbreviations).unwrap();
        assert_eq!(text, "the xyzzy", "version {version}");
    }
}

/// A story with an alphabet table in its scratch area: A0 is upper case, A1 lower,
/// and A2 has `*` where the default has `0`.
fn custom_alphabet() -> (Memory, header::Header, Vec<u32>) {
    let mut story = Story5::new(5);
    let mut table: Vec<u8> = (b'A'..=b'Z').chain(b'a'..=b'z').collect();
    table.extend(b"  *123456789.,!?_#'\"/\\-:()");
    assert_eq!(table.len(), 78);
    let at = story.scratch(&table);
    story.header_word(0x34, at);
    let pack = |z: &[u16]| -> Vec<u8> {
        z.chunks(3)
            .enumerate()
            .flat_map(|(i, c)| {
                let end = if (i + 1) * 3 >= z.len() { 0x8000 } else { 0 };
                (end | c[0] << 10 | c[1] << 5 | c[2]).to_be_bytes()
            })
            .collect()
    };
    let strings = [
        pack(&[13, 4, 13]),        // H, shift, h
        pack(&[5, 8, 5, 7, 5, 5]), // A2: '*', then new-line
        pack(&[5, 6, 2, 0, 5, 5]), // A2 escape: ZSCII 64, '@'
    ];
    let addresses = strings
        .iter()
        .map(|s| story.raw_string(s) as u32 * 4)
        .collect();
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let h = header::parse(&image).unwrap();
    (Memory::new(image, h.static_base as usize), h, addresses)
}

// §3.5.5: a game's alphabet table replaces all three alphabets. §3.5.5.1: Z-characters
// 6 and 7 of A2 remain the escape and new-line whatever the table holds.
#[test]
fn a_games_alphabet_table_is_used_for_decoding() {
    let (m, h, at) = custom_alphabet();
    let c = Charset::read(&m, h.alphabet_table, h.unicode_table);
    let text = |i: usize| c.decode(&m, at[i], h.abbreviations).unwrap().0;
    assert_eq!(text(0), "Hh");
    assert_eq!(text(1), "*\n");
    assert_eq!(text(2), "@");
}

// §3.7: encoding uses the same table, so that a typed word finds its dictionary
// entry.
#[test]
fn a_games_alphabet_table_is_used_for_encoding() {
    let (m, h, _) = custom_alphabet();
    let c = Charset::read(&m, h.alphabet_table, h.unicode_table);
    // With this table lower-case letters are in A1 (shift 4) and '*' in A2.
    // "a*": 4,6 / 5,8 / then padding to nine.
    let encoded = c.encode(b"a*", 9);
    let z: Vec<u16> = encoded
        .chunks(2)
        .flat_map(|w| {
            let w = u16::from_be_bytes([w[0], w[1]]);
            [w >> 10 & 31, w >> 5 & 31, w & 31]
        })
        .collect();
    assert_eq!(z, vec![4, 6, 5, 8, 5, 5, 5, 5, 5]);
    // '0' is not in this table, so it needs the ten-bit escape (§3.7.1).
    let z0 = c.encode(b"0", 9);
    assert_eq!(u16::from_be_bytes([z0[0], z0[1]]), 5 << 10 | 6 << 5 | 1);
}

// §3.8.5.2: a game's Unicode translation table replaces the default extra
// characters, for output and for input.
#[test]
fn a_games_unicode_table_replaces_the_default_extras() {
    let mut story = Story5::new(5);
    // Three characters: alpha, beta, euro sign.
    let table = story.scratch(&[3, 0x03, 0xB1, 0x03, 0xB2, 0x20, 0xAC]);
    let extension = story.scratch(&[0, 3, 0, 0, 0, 0, (table >> 8) as u8, table as u8]);
    story.header_word(0x36, extension);
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let h = header::parse(&image).unwrap();
    assert_eq!(h.unicode_table, table);
    let m = Memory::new(image, h.static_base as usize);
    let c = Charset::read(&m, h.alphabet_table, h.unicode_table);
    assert_eq!(c.zscii_to_char(155), Some('\u{3b1}'));
    assert_eq!(c.zscii_to_char(157), Some('\u{20ac}'));
    assert_eq!(
        c.zscii_to_char(158),
        Some('?'),
        "beyond the table's three entries"
    );
    assert_eq!(c.zscii_for('\u{3b2}'), Some(156));
    assert_eq!(
        c.zscii_for('\u{e4}'),
        None,
        "a-diaeresis is not in this game's table"
    );
    assert_eq!(c.input_zscii('\u{e4}'), b'?');
}

// The default tables, through the same type: version 3's rules are unchanged.
#[test]
fn the_default_charset_is_version_3s() {
    let c = Charset::default();
    assert_eq!(c.zscii_to_char(155), Some('\u{e4}'));
    assert_eq!(c.zscii_to_char(224), Some('?'));
    assert_eq!(c.zscii_for('~'), Some(126));
    assert_eq!(c.zscii_for('\u{4e2d}'), None);
    assert_eq!(c.input_zscii('A'), b'a');
}

// The real thing: Advent's object names decode from both builds, using the version
// 4+ object table and the packed-address rules of each.
#[test]
fn advents_object_names_decode_in_both_versions() {
    for version in [5u8, 8] {
        let (m, h) = advent(version);
        let c = Charset::read(&m, h.alphabet_table, h.unicode_table);
        let o = Objects::for_version(h.objects, h.v);
        let mut names = std::collections::HashSet::new();
        for object in 1..=300u16 {
            if let Ok(Some(at)) = o.name_address(&m, object)
                && let Ok((name, _)) = c.decode(&m, at, h.abbreviations)
            {
                names.insert(name);
            }
        }
        for expected in [
            "brass lantern",
            "Inside Building",
            "At End Of Road",
            "set of keys",
        ] {
            assert!(
                names.contains(expected),
                "version {version}: {expected} missing"
            );
        }
    }
}

// zencode and the decoder agree in version 5 as in version 3.
#[test]
fn round_trip_in_version_5() {
    let mut story = Story5::new(5);
    let text = "Hello, World! 0123456789 \"quoted\" (x-y)";
    let at = story.raw_string(&zencode(text)) as u32 * 4;
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let h = header::parse(&image).unwrap();
    let m = Memory::new(image, h.static_base as usize);
    assert_eq!(
        Charset::default()
            .decode(&m, at, h.abbreviations)
            .unwrap()
            .0,
        text
    );
}
