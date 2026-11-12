use super::*;
use crate::FaultKind;
use crate::header;
use crate::memory::Memory;
use zmachine_testkit::{Story, op, zencode, zencode_dict};

/// Packs raw Z-characters three to a word with the end bit on the last (§3.2).
fn pack(zchars: &[u8]) -> Vec<u8> {
    let mut z = zchars.to_vec();
    while !z.len().is_multiple_of(3) || z.is_empty() {
        z.push(5);
    }
    let words = z.len() / 3;
    let mut out = Vec::new();
    for (i, c) in z.chunks(3).enumerate() {
        let mut w = (c[0] as u16) << 10 | (c[1] as u16) << 5 | c[2] as u16;
        if i == words - 1 {
            w |= 0x8000;
        }
        out.extend(w.to_be_bytes());
    }
    out
}

/// A story holding the given strings (already encoded) and these abbreviations:
/// 0 = "the ", 33 = "xyzzy", 95 = "Zork".
struct Fixture {
    memory: Memory,
    abbreviations: u16,
    at: Vec<u32>,
}

fn fixture(strings: &[Vec<u8>]) -> Fixture {
    let mut story = Story::new();
    story
        .abbreviation(0, "the ")
        .abbreviation(33, "xyzzy")
        .abbreviation(95, "Zork");
    let at = strings
        .iter()
        .map(|s| story.raw_string(s) as u32 * 2)
        .collect();
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let header = header::parse(&image).unwrap();
    Fixture {
        memory: Memory::new(image, header.static_base as usize),
        abbreviations: header.abbreviations,
        at,
    }
}

fn decode_one(bytes: Vec<u8>) -> String {
    let f = fixture(&[bytes]);
    decode_string(&f.memory, f.at[0], f.abbreviations)
        .unwrap()
        .0
}

// §3.2, §3.5.3: alphabet A0 holds a to z at Z-characters 6 to 31; 0 is a space.
#[test]
fn lower_case_and_spaces() {
    assert_eq!(decode_one(zencode("hello world")), "hello world");
    assert_eq!(decode_one(pack(&[13, 10, 17, 17, 20])), "hello");
}

// §3.2: decoding stops at the word with the top bit set and reports where it ended.
#[test]
fn decoding_stops_at_the_end_bit_and_reports_the_end_address() {
    let mut bytes = zencode("hello");
    bytes.extend(zencode("ignored"));
    let f = fixture(&[bytes]);
    let (text, end) = decode_string(&f.memory, f.at[0], f.abbreviations).unwrap();
    assert_eq!(text, "hello");
    assert_eq!(end, f.at[0] + 4);
}

// §3.2.3: in version 3, Z-characters 4 and 5 shift to A1 and A2 for one character.
#[test]
fn shifts_apply_to_one_character() {
    assert_eq!(decode_one(pack(&[4, 13, 13])), "Hh");
    assert_eq!(decode_one(pack(&[5, 8, 13])), "0h");
    assert_eq!(decode_one(zencode("Hello, World!")), "Hello, World!");
}

// §3.5.3: the whole of alphabet A2 for versions 2 and later. Character 7 is a
// new-line.
#[test]
fn alphabet_a2() {
    assert_eq!(
        decode_one(zencode("0123456789.,!?_#'\"/\\-:()")),
        "0123456789.,!?_#'\"/\\-:()"
    );
    assert_eq!(decode_one(pack(&[5, 7])), "\n");
    assert_eq!(decode_one(zencode("a\nb")), "a\nb");
}

// §3.5.1: Z-character 0 is a space in every alphabet, and uses up a pending shift.
#[test]
fn space_is_the_same_in_every_alphabet() {
    assert_eq!(decode_one(pack(&[4, 0, 13])), " h");
    assert_eq!(decode_one(pack(&[5, 0, 13])), " h");
}

// §3.3: Z-characters 1, 2, 3 followed by x print abbreviation 32(z-1)+x.
#[test]
fn abbreviations_from_all_three_tables() {
    assert_eq!(decode_one(pack(&[1, 0, 6])), "the a");
    assert_eq!(decode_one(pack(&[2, 1])), "xyzzy");
    assert_eq!(decode_one(pack(&[3, 31])), "Zork");
    assert_eq!(decode_one(pack(&[13, 1, 0, 3, 31, 13])), "hthe Zorkh");
}

// §3.4: A2 character 6 is followed by a ten-bit ZSCII code in two Z-characters.
#[test]
fn ten_bit_zscii_escape() {
    assert_eq!(decode_one(zencode("@")), "@");
    assert_eq!(decode_one(pack(&[5, 6, 2, 0, 13])), "@h");
    // 155 is the first of the default extra characters, a-diaeresis (§3.8.7).
    assert_eq!(decode_one(pack(&[5, 6, 4, 27])), "\u{e4}");
}

// §3.4, §3.6.1: a string may end in the middle of a multi-character construction,
// which then prints nothing.
#[test]
fn incomplete_constructions_at_the_end_print_nothing() {
    assert_eq!(decode_one(pack(&[13, 4])), "h");
    assert_eq!(decode_one(pack(&[13, 5, 6])), "h");
    // Six Z-characters fill two words exactly, so no padding follows the first half
    // of the ZSCII code to be taken for its second half.
    assert_eq!(decode_one(pack(&[13, 14, 13, 5, 6, 2])), "hih");
    assert_eq!(decode_one(pack(&[13, 14, 1])), "hi");
    // Padding 5s after the text (§3.6) are shifts with nothing after them.
    assert_eq!(decode_one(pack(&[13, 5, 5])), "h");
    assert_eq!(decode_one(pack(&[5, 5, 5])), "");
}

// A string with no end bit before the end of the file is a fault, not a panic.
#[test]
fn text_running_past_the_end_of_the_file_is_a_fault() {
    let mut bytes = vec![0u8; 0x40];
    bytes.extend([0x35, 0x51, 0x46, 0x85]);
    let memory = Memory::new(bytes, 0x40);
    assert_eq!(decode_string(&memory, 0x40, 0), Err(FaultKind::TextOverrun));
    assert_eq!(
        decode_string(&memory, 0x1000, 0),
        Err(FaultKind::TextOverrun)
    );
}

// §3.8: ZSCII codes on output.
#[test]
fn zscii_to_unicode() {
    assert_eq!(zscii_to_char(0), None, "§3.8.2.1: 0 prints nothing");
    assert_eq!(zscii_to_char(13), Some('\n'), "§3.8.2.2: 13 is new-line");
    assert_eq!(zscii_to_char(32), Some(' '));
    assert_eq!(zscii_to_char(65), Some('A'));
    assert_eq!(zscii_to_char(126), Some('~'));
    // §3.8.7: the default extra characters run from 155 to 223.
    assert_eq!(zscii_to_char(155), Some('\u{e4}'));
    assert_eq!(zscii_to_char(161), Some('\u{df}'));
    assert_eq!(zscii_to_char(219), Some('\u{a3}'));
    assert_eq!(zscii_to_char(223), Some('\u{bf}'));
    // Codes with no meaning on output print a question mark.
    for undefined in [1u16, 9, 12, 31, 127, 154, 224, 255, 300, 1023] {
        assert_eq!(zscii_to_char(undefined), Some('?'), "code {undefined}");
    }
}

// §3.8 on input: a character typed by the player becomes its ZSCII code, in lower
// case; one with no code becomes a question mark.
#[test]
fn unicode_to_zscii_for_input() {
    assert_eq!(char_to_zscii('a'), b'a');
    assert_eq!(char_to_zscii('A'), b'a');
    assert_eq!(char_to_zscii(' '), b' ');
    assert_eq!(char_to_zscii('~'), b'~');
    assert_eq!(char_to_zscii('\u{e4}'), 155);
    assert_eq!(char_to_zscii('\u{4e2d}'), b'?');
    assert_eq!(char_to_zscii('\t'), b'?');
    assert_eq!(char_to_zscii('\u{1F600}'), b'?');
}

// §13.3, §13.4: a word is encoded to six Z-characters in four bytes, truncated or
// padded with 5s. The testkit encoder is itself checked against Zork I's dictionary.
#[test]
fn encode_for_dictionary_lookup() {
    for word in [
        "a",
        "mailbox",
        "north",
        "xyzzy",
        "1",
        "1.5",
        "can't",
        "x@y",
        "zzzzzzzzzz",
        "",
    ] {
        assert_eq!(encode_dict(word.as_bytes()), zencode_dict(word), "{word:?}");
    }
    assert_eq!(encode_dict(b"mailbox"), encode_dict(b"mailbo"));
    assert_eq!(encode_dict(b"hello"), [0x35, 0x51, 0xC6, 0x85]);
    // A multi-character construction cut by the six-character limit is kept as
    // far as it fits: "aaaaa@" is five letters and the shift.
    assert_eq!(encode_dict(b"aaaaa@"), zencode_dict("aaaaa@"));
}
