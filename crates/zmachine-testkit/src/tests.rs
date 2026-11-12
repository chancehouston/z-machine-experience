//! Known-vector tests for the encoders, so that a mistake here cannot be mirrored
//! unseen by the interpreter that is tested with these images.

use crate::op::{self, Arg};
use crate::{GLOBALS, Object, SCRATCH, Story, zencode, zencode_dict};

fn zork1() -> Vec<u8> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");
    std::fs::read(path).expect("stories/zork1/zork1.z3 is committed")
}

fn word(bytes: &[u8], at: usize) -> usize {
    (bytes[at] as usize) << 8 | bytes[at + 1] as usize
}

/// The four-byte encoded form of every word in a version 3 dictionary (§13.2, §13.3).
fn dictionary_keys(story: &[u8]) -> Vec<[u8; 4]> {
    let dict = word(story, 0x08);
    let separators = story[dict] as usize;
    let entry_len = story[dict + 1 + separators] as usize;
    let count = word(story, dict + 2 + separators);
    let first = dict + 4 + separators;
    (0..count)
        .map(|i| {
            let at = first + i * entry_len;
            [story[at], story[at + 1], story[at + 2], story[at + 3]]
        })
        .collect()
}

// §3.2, §3.5.3: three 5-bit Z-characters per word, alphabet A0 puts a..z at 6..31.
// "hello" is 13,10,17 / 17,20,pad(5), with the end bit on the last word.
#[test]
fn zencode_hello_matches_hand_computed_words() {
    assert_eq!(zencode("hello"), vec![0x35, 0x51, 0xC6, 0x85]);
}

// §3.5.1: space is Z-character 0. "a b" is 6,0,7 in one word with the end bit.
#[test]
fn zencode_space_is_zchar_zero() {
    assert_eq!(zencode("a b"), vec![0x98, 0x07]);
}

// §3.2.3, §3.5.3: Z-character 4 shifts to A1 (upper case) for one character.
// "Hi" is 4,13,14 with the end bit.
#[test]
fn zencode_upper_case_uses_shift_4() {
    assert_eq!(zencode("Hi"), vec![0x91, 0xAE]);
}

// §3.5.3: Z-character 5 shifts to A2, where '0' is 8 and '.' is 18.
// "0." is 5,8,5 / 18,5,5.
#[test]
fn zencode_digits_and_punctuation_use_shift_5() {
    assert_eq!(zencode("0."), vec![0x15, 0x05, 0xC8, 0xA5]);
}

// §3.4: A2 character 6 introduces a ten-bit ZSCII code as two Z-characters.
// '@' is ZSCII 64 = 0b00010_00000: 5,6,2 / 0,pad,pad.
#[test]
fn zencode_other_characters_use_the_ten_bit_escape() {
    assert_eq!(zencode("@"), vec![0x14, 0xC2, 0x80, 0xA5]);
}

// §3.2: an empty string is still one word, all padding.
#[test]
fn zencode_empty_string_is_one_padded_word() {
    assert_eq!(zencode(""), vec![0x94, 0xA5]);
}

// §13.3, §13.4: version 3 dictionary words are 6 Z-characters in 4 bytes, truncated
// and padded with 5s, end bit on the second word only.
#[test]
fn zencode_dict_pads_and_truncates_to_six_zchars() {
    assert_eq!(zencode_dict("hello"), [0x35, 0x51, 0xC6, 0x85]);
    assert_eq!(zencode_dict("a"), [0x18, 0xA5, 0x94, 0xA5]);
    assert_eq!(zencode_dict("mailbox"), zencode_dict("mailbo"));
}

// The same encoder must produce what Infocom's compiler produced. These words are
// in Zork I's vocabulary; a made-up one is not.
#[test]
fn zencode_dict_agrees_with_the_zork1_dictionary() {
    let keys = dictionary_keys(&zork1());
    for w in [
        "mailbox", "troll", "xyzzy", "lantern", "north", "leaflet", "sword", "all",
    ] {
        assert!(
            keys.contains(&zencode_dict(w)),
            "{w} should be in Zork I's dictionary"
        );
    }
    assert!(!keys.contains(&zencode_dict("qqqqqq")));
}

// §13.2: entries are in numerical order of the encoded text.
#[test]
fn zork1_dictionary_is_sorted_as_our_reader_assumes() {
    let keys = dictionary_keys(&zork1());
    assert!(keys.windows(2).all(|w| w[0] < w[1]));
}

// §4.3.2, §4.4.2: long form. Bit 6 and bit 5 give the operand types (0 small
// constant, 1 variable); the bottom five bits are the opcode number.
#[test]
fn op2_long_form_bytes() {
    assert_eq!(op::op2(op::ADD, Arg::S(2), Arg::S(3)), vec![0x14, 2, 3]);
    assert_eq!(op::op2(op::ADD, Arg::V(1), Arg::S(3)), vec![0x54, 1, 3]);
    assert_eq!(
        op::op2(op::ADD, Arg::S(2), Arg::V(0x10)),
        vec![0x34, 2, 0x10]
    );
    assert_eq!(op::op2(op::ADD, Arg::V(1), Arg::V(2)), vec![0x74, 1, 2]);
}

// §4.3.3, §4.4.3: a 2OP with a large constant must use variable form, top bits 110,
// followed by a byte of four 2-bit operand types (00 large, 01 small, 10 variable,
// 11 omitted).
#[test]
fn op2_with_a_large_constant_uses_variable_form() {
    assert_eq!(
        op::op2(op::ADD, Arg::L(0x1234), Arg::S(3)),
        vec![0xD4, 0b00_01_11_11, 0x12, 0x34, 3]
    );
}

// §4.3.3: `je` may take up to four operands in variable form.
#[test]
fn op2_var_form_with_four_operands() {
    assert_eq!(
        op::op2_var(op::JE, &[Arg::V(1), Arg::S(2), Arg::S(3), Arg::S(4)]),
        vec![0xC1, 0b10_01_01_01, 1, 2, 3, 4]
    );
}

// §4.3.1: short form, top bits 10; bits 4-5 give the operand type, 11 meaning 0OP.
#[test]
fn op1_and_op0_short_form_bytes() {
    assert_eq!(op::op1(op::JZ, Arg::L(0x0102)), vec![0x80, 1, 2]);
    assert_eq!(op::op1(op::JZ, Arg::S(7)), vec![0x90, 7]);
    assert_eq!(op::op1(op::JZ, Arg::V(0)), vec![0xA0, 0]);
    assert_eq!(op::op0(op::RTRUE), vec![0xB0]);
    assert_eq!(op::op0(op::QUIT), vec![0xBA]);
}

// §4.3.3: VAR form, top bits 111.
#[test]
fn var_form_bytes() {
    assert_eq!(
        op::var(op::CALL, &[Arg::L(0x0400)]),
        vec![0xE0, 0b00_11_11_11, 4, 0]
    );
    assert_eq!(
        op::var(op::STOREW, &[Arg::L(0x0300), Arg::S(1), Arg::V(5)]),
        vec![0xE1, 0b00_01_10_11, 3, 0, 1, 5]
    );
    assert_eq!(op::var(op::PRINT_NUM, &[]), vec![0xE6, 0xFF]);
}

// §4.7: branch data. Bit 7 is the sense; bit 6 set means a 6-bit offset in one byte,
// otherwise a signed 14-bit offset in two.
#[test]
fn branch_bytes() {
    assert_eq!(op::branch(true, 0), vec![0xC0]);
    assert_eq!(op::branch(true, 1), vec![0xC1]);
    assert_eq!(op::branch(false, 63), vec![0x7F]);
    assert_eq!(op::branch(true, 64), vec![0x80, 0x40]);
    assert_eq!(op::branch(false, -1), vec![0x3F, 0xFF]);
    assert_eq!(op::branch_long(true, 5), vec![0x80, 0x05]);
}

// §11.1: header fields of a built image read back at their offsets.
#[test]
fn built_image_has_a_coherent_header() {
    let mut story = Story::new();
    story.release(7).serial("261111").flags1(0x02);
    story.word("north");
    let obj = story.object(Object::new("lamp"));
    assert_eq!(obj, 1);
    let routine = story.routine(&[1, 2], &op::op0(op::RTRUE));
    let main_at = story.next_code_address();
    story.main(&op::op0(op::QUIT));
    let image = story.build();

    assert_eq!(image[0], 3);
    assert_eq!(image[1], 0x02);
    assert_eq!(word(&image, 0x02), 7);
    assert_eq!(&image[0x12..0x18], b"261111");
    assert_eq!(word(&image, 0x06), main_at as usize);
    assert_eq!(word(&image, 0x0C), GLOBALS as usize);
    assert_eq!(word(&image, 0x18), 0x40);
    let static_base = word(&image, 0x0E);
    let high_base = word(&image, 0x04);
    let dict = word(&image, 0x08);
    let objects = word(&image, 0x0A);
    assert!(SCRATCH as usize + 0x200 <= objects);
    assert!(objects < static_base);
    assert!(static_base <= dict && dict < high_base);
    assert!(high_base <= main_at as usize);
    assert_eq!(image.len() % 2, 0);
    assert_eq!(word(&image, 0x1A) * 2, image.len());

    // §15 verify: the checksum is the sum of all bytes from 0x40, modulo 0x10000.
    let sum = image[0x40..].iter().fold(0u32, |a, &b| a + b as u32) & 0xFFFF;
    assert_eq!(word(&image, 0x1C), sum as usize);

    // §5.2: a version 3 routine starts with its local count, then initial values.
    let at = routine as usize * 2;
    assert_eq!(&image[at..at + 6], &[2, 0, 1, 0, 2, 0xB0]);

    // The dictionary holds the one word, in the layout of §13.2.
    assert_eq!(dictionary_keys(&image), vec![zencode_dict("north")]);
}

// §12.3.1, §12.4: version 3 objects are 9 bytes after 31 default words; the
// property table starts with the name's length in words, then the name.
#[test]
fn built_objects_follow_the_version_3_layout() {
    let mut story = Story::new();
    story.prop_default(5, 0xBEEF);
    let mut a = Object::new("ab");
    a.attributes = 0x8000_0001;
    a.child = 2;
    a.props = vec![(5, vec![0x12, 0x34]), (2, vec![0x7F])];
    story.object(a);
    let mut b = Object::new("");
    b.parent = 1;
    story.object(b);
    story.main(&op::op0(op::QUIT));
    let image = story.build();

    let table = word(&image, 0x0A);
    assert_eq!(word(&image, table + 2 * 4), 0xBEEF);
    let first = table + 62;
    assert_eq!(&image[first..first + 4], &[0x80, 0, 0, 0x01]);
    assert_eq!(&image[first + 4..first + 7], &[0, 0, 2]);
    let props = word(&image, first + 7);
    assert_eq!(image[props], 1, "name is one word long");
    assert_eq!(&image[props + 1..props + 3], &zencode("ab")[..]);
    // §12.4.1: size byte is 32 * (length - 1) + number, in descending number order.
    assert_eq!(&image[props + 3..props + 6], &[32 + 5, 0x12, 0x34]);
    assert_eq!(&image[props + 6..props + 8], &[2, 0x7F]);
    assert_eq!(image[props + 8], 0);

    let second = first + 9;
    assert_eq!(image[second + 4], 1, "parent");
    let props2 = word(&image, second + 7);
    assert_eq!(
        &image[props2..props2 + 2],
        &[0, 0],
        "no name, no properties"
    );
}

// §3.3: abbreviation strings are reached through a table of word addresses.
#[test]
fn built_abbreviations_are_word_addresses_of_zstrings() {
    let mut story = Story::new();
    story.abbreviation(0, "the ");
    story.abbreviation(33, "xyzzy");
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let table = word(&image, 0x18);
    let first = word(&image, table) * 2;
    let enc = zencode("the ");
    assert_eq!(&image[first..first + enc.len()], &enc[..]);
    let other = word(&image, table + 33 * 2) * 2;
    let enc = zencode("xyzzy");
    assert_eq!(&image[other..other + enc.len()], &enc[..]);
}

// Strings in high memory are word-aligned so they have packed addresses (§1.2.3).
#[test]
fn strings_and_scratch_data_are_placed_where_reported() {
    let mut story = Story::new();
    let buf = story.scratch(&[1, 2, 3]);
    let s = story.string("hello");
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    assert_eq!(buf, SCRATCH);
    assert_eq!(&image[buf as usize..buf as usize + 3], &[1, 2, 3]);
    let at = s as usize * 2;
    assert_eq!(&image[at..at + 4], &[0x35, 0x51, 0xC6, 0x85]);
}
