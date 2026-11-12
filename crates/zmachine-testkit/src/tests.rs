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

// ---------------------------------------------------------------------------
// Versions 5 and 8 (feature 002, T008)
// ---------------------------------------------------------------------------

use crate::{zencode_dict9, Object5, Story5};

fn advent(version: u8) -> Vec<u8> {
    let path = format!("{}/../../stories/advent/advent.z{version}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(path).expect("stories/advent is committed")
}

/// The six-byte encoded form of every word in a version 5 or 8 dictionary (§13.2,
/// §13.4).
fn dictionary_keys9(story: &[u8], dictionary: usize) -> Vec<[u8; 6]> {
    let separators = story[dictionary] as usize;
    let entry_len = story[dictionary + 1 + separators] as usize;
    let count = (word(story, dictionary + 2 + separators) as i16).unsigned_abs() as usize;
    let first = dictionary + 4 + separators;
    (0..count)
        .map(|i| story[first + i * entry_len..][..6].try_into().unwrap())
        .collect()
}

/// A simple checksum of an image, to notice any change at all.
fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

// Guarantee 6 of the core contract depends on this: adding versions 5 and 8 must
// not change a single byte of the version 3 images the first feature's tests and
// fixtures were made from.
#[test]
fn version_3_images_are_byte_for_byte_what_they_were() {
    assert_eq!(fingerprint(&crate::programs::random_probe()), V3_RANDOM_PROBE);
    let probes: Vec<u64> =
        crate::programs::transcript_probes().iter().take(15).map(|p| fingerprint(&p.image)).collect();
    assert_eq!(probes, V3_TRANSCRIPT_PROBES);
}

// §13.4: version 5 dictionary words are 9 Z-characters in 6 bytes, padded with 5s,
// with the end bit on the third word only.
#[test]
fn zencode_dict9_pads_and_truncates_to_nine_zchars() {
    // "hello": 13,10,17 / 17,20,5 / 5,5,5 with the end bit.
    assert_eq!(zencode_dict9("hello"), [0x35, 0x51, 0x46, 0x85, 0x94, 0xA5]);
    assert_eq!(zencode_dict9("a"), [0x18, 0xA5, 0x14, 0xA5, 0x94, 0xA5]);
    assert_eq!(zencode_dict9("lanternfish"), zencode_dict9("lanternfi"));
    assert_ne!(zencode_dict9("lanternf"), zencode_dict9("lanternfi"));
}

// The same encoder must produce what the Inform compiler produced: these words are
// in Advent's vocabulary, in both builds, and a made-up one is not.
#[test]
fn zencode_dict9_agrees_with_advents_dictionary() {
    for version in [5u8, 8] {
        let story = advent(version);
        let keys = dictionary_keys9(&story, word(&story, 0x08));
        for w in ["lantern", "xyzzy", "plugh", "inventory", "building", "north", "take", "i"] {
            assert!(keys.contains(&zencode_dict9(w)), "{w} should be in Advent z{version}");
        }
        assert!(!keys.contains(&zencode_dict9("qqqqqqqqq")));
        assert!(keys.windows(2).all(|w| w[0] < w[1]), "Inform sorts its dictionary");
    }
}

// §12.3.2: Advent's own object table has 14-byte entries whose links are object
// numbers and whose property addresses lie in dynamic memory. This checks the
// layout the builder writes against a real compiler's.
#[test]
fn advents_object_table_has_the_layout_the_builder_writes() {
    let story = advent(5);
    let table = word(&story, 0x0A);
    let static_base = word(&story, 0x0E);
    let first = table + 63 * 2;
    // §12.3.2: the first object's property table follows the last object entry.
    let count = (word(&story, first + 12) - first) / 14;
    assert!(count > 200, "Advent has a few hundred objects, found {count}");
    for n in 0..count {
        let e = first + n * 14;
        for link in [6, 8, 10] {
            assert!(word(&story, e + link) <= count, "object {} link at +{link}", n + 1);
        }
        let props = word(&story, e + 12);
        assert!(props > e && props < static_base, "object {} property table", n + 1);
    }
}

// §11.1, §1.2.3: header fields of a built version 5 image; packed addresses are
// byte addresses divided by 4.
#[test]
fn built_version_5_image_has_a_coherent_header() {
    let mut story = Story5::new(5);
    story.release(7).serial("261111");
    story.word("north");
    assert_eq!(story.object(Object5::new("lamp")), 1);
    let routine = story.routine(2, &op::op0(op::RTRUE));
    let main_at = story.next_code_address();
    story.main(&op::op0(op::QUIT));
    let image = story.build();

    assert_eq!(image[0], 5);
    assert_eq!(word(&image, 0x02), 7);
    assert_eq!(&image[0x12..0x18], b"261111");
    assert_eq!(word(&image, 0x06), main_at as usize);
    assert_eq!(word(&image, 0x0C), GLOBALS as usize);
    assert_eq!(image.len() % 4, 0);
    // §11.1.6: the length is stored divided by 4.
    assert_eq!(word(&image, 0x1A) * 4, image.len());
    let sum = image[0x40..].iter().fold(0u32, |a, &b| a + b as u32) & 0xFFFF;
    assert_eq!(word(&image, 0x1C), sum as usize);

    // §5.2.1: a version 5 routine is its local count and then code.
    let at = routine as usize * 4;
    assert_eq!(&image[at..at + 2], &[2, 0xB0]);
    assert_eq!(dictionary_keys9(&image, word(&image, 0x08)), vec![zencode_dict9("north")]);
}

// §1.2.3, §11.1.6: version 8 divides packed addresses and the length by 8.
#[test]
fn built_version_8_image_uses_multiples_of_8() {
    let mut story = Story5::new(8);
    let s = story.string("hello");
    let routine = story.routine(0, &op::op0(op::RTRUE));
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    assert_eq!(image[0], 8);
    assert_eq!(image.len() % 8, 0);
    assert_eq!(word(&image, 0x1A) * 8, image.len());
    assert_eq!(&image[s as usize * 8..][..4], &[0x35, 0x51, 0xC6, 0x85]);
    assert_eq!(&image[routine as usize * 8..][..2], &[0, 0xB0]);
}

// §12.3.2, §12.4.2: 14-byte entries after 63 default words; 48 attribute bits;
// word links; property size in one byte for lengths 1 and 2, two for longer.
#[test]
fn built_version_5_objects_follow_the_layout() {
    let mut story = Story5::new(5);
    story.prop_default(63, 0xBEEF);
    let mut a = Object5::new("ab");
    a.attributes = 1 << 47 | 1; // attributes 0 and 47
    a.child = 0x0102;
    a.props = vec![
        (40, vec![0x12, 0x34]),
        (33, vec![9, 8, 7]),
        (5, vec![0x7F]),
        (2, vec![0xAA; 64]),
    ];
    story.object(a);
    story.main(&op::op0(op::QUIT));
    let image = story.build();

    let table = word(&image, 0x0A);
    assert_eq!(word(&image, table + 2 * 62), 0xBEEF);
    let e = table + 126;
    assert_eq!(&image[e..e + 6], &[0x80, 0, 0, 0, 0, 0x01]);
    assert_eq!(&image[e + 6..e + 12], &[0, 0, 0, 0, 1, 2], "parent, sibling, child as words");
    let p = word(&image, e + 12);
    assert_eq!(image[p], 1, "name is one word long");
    // Descending order. Length 2: bit 6 set. Length 3: two bytes, second has the length.
    assert_eq!(&image[p + 3..p + 6], &[0x40 | 40, 0x12, 0x34]);
    assert_eq!(&image[p + 6..p + 11], &[0x80 | 33, 0x80 | 3, 9, 8, 7]);
    assert_eq!(&image[p + 11..p + 13], &[5, 0x7F]);
    // §12.4.2.1.1: a length of 64 is stored as 0.
    assert_eq!(&image[p + 13..p + 15], &[0x80 | 2, 0x80]);
    assert_eq!(image[p + 15 + 64], 0, "the terminator follows 64 bytes of data");
}

// §4.3.4: extended form is 0xBE, the opcode number, a types byte, the operands.
#[test]
fn extended_form_bytes() {
    assert_eq!(op::ext(op::SAVE_UNDO, &[]), vec![0xBE, 9, 0xFF]);
    assert_eq!(
        op::ext(op::LOG_SHIFT, &[Arg::L(0x1234), Arg::S(3)]),
        vec![0xBE, 2, 0b00_01_11_11, 0x12, 0x34, 3]
    );
}

// §4.4.3.1: call_vs2 and call_vn2 have two bytes of operand types.
#[test]
fn double_variable_form_bytes() {
    let args: Vec<Arg> = (1..=8).map(Arg::S).collect();
    assert_eq!(
        op::var2(op::CALL_VS2, &args),
        vec![0xEC, 0x55, 0x55, 1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(
        op::var2(op::CALL_VN2, &[Arg::L(0x0400), Arg::V(1), Arg::S(2), Arg::S(3), Arg::S(4)]),
        vec![0xFA, 0b00_10_01_01, 0b01_11_11_11, 4, 0, 1, 2, 3, 4]
    );
}

// §15 read: the version 5 text buffer has its capacity in byte 0 and the number of
// characters already present in byte 1.
#[test]
fn version_5_buffers() {
    let mut story = Story5::new(5);
    let text = story.text_buffer(20, "abc");
    let parse = story.parse_buffer(4);
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    assert_eq!(&image[text as usize..][..5], &[20, 3, b'a', b'b', b'c']);
    assert_eq!(image[parse as usize], 4);
}

// §13.2: an unsorted dictionary has a negative entry count.
#[test]
fn dictionary_bytes_sorted_and_unsorted() {
    let sorted = Story5::dictionary_bytes(".", &["zebra", "apple"], true);
    assert_eq!(&sorted[..5], &[1, b'.', 9, 0, 2]);
    assert_eq!(&sorted[5..11], &zencode_dict9("apple"));
    let unsorted = Story5::dictionary_bytes(".", &["zebra", "apple"], false);
    assert_eq!(&unsorted[3..5], &(-2i16).to_be_bytes());
    assert_eq!(&unsorted[5..11], &zencode_dict9("zebra"));
}

// A version 8 image can exceed version 5's limit, with code beyond 256K.
#[test]
fn a_version_8_image_can_be_larger_than_256k() {
    let mut story = Story5::new(8);
    story.main(&op::op0(op::QUIT));
    story.filler(300 * 1024);
    let far = story.routine(0, &op::op0(op::RTRUE));
    let image = story.build();
    assert!(image.len() > 256 * 1024);
    assert!(far as usize * 8 > 256 * 1024);
    assert_eq!(&image[far as usize * 8..][..2], &[0, 0xB0]);
}

// Recorded before any version 5 code was added to the testkit.
const V3_RANDOM_PROBE: u64 = 5417669845303728571;
const V3_TRANSCRIPT_PROBES: [u64; 15] = [
    454429368906337069,
    420175888543861311,
    14197230477765710539,
    1667820991696585850,
    2577020743345998557,
    982986369477493244,
    688989483018303118,
    13572019307251096689,
    7685274411442232617,
    2311778466523503765,
    7388862359471212685,
    11988781680383080542,
    14388547041347996110,
    643607776771933347,
    6712874335486275676,
];
