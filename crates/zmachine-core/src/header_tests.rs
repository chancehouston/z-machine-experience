use super::*;
use crate::{HeaderField, LoadOptions, Machine};
use zmachine_testkit::{op, Story};

const OPTIONS: LoadOptions = LoadOptions { seed: Some(1), entropy: 0 };

fn image() -> Vec<u8> {
    let mut story = Story::new();
    story.release(88).serial("840726");
    story.word("north");
    story.main(&op::op0(op::QUIT));
    story.build()
}

fn put(image: &mut [u8], at: usize, value: u16) {
    image[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

fn word(image: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([image[at], image[at + 1]])
}

// §11.1: the fields of a well-formed header are read from their offsets.
#[test]
fn parse_reads_every_field_from_its_offset() {
    let image = image();
    let h = parse(&image).unwrap();
    assert_eq!(h.version, 3);
    assert_eq!(h.release, 88);
    assert_eq!(&h.serial, b"840726");
    assert_eq!(h.high_base, word(&image, 0x04));
    assert_eq!(h.initial_pc, word(&image, 0x06));
    assert_eq!(h.dictionary, word(&image, 0x08));
    assert_eq!(h.objects, word(&image, 0x0A));
    assert_eq!(h.globals, word(&image, 0x0C));
    assert_eq!(h.static_base, word(&image, 0x0E));
    assert_eq!(h.abbreviations, word(&image, 0x18));
    assert_eq!(h.checksum, word(&image, 0x1C));
    // §11.1.6: in versions 1 to 3 the stored length is the file length divided by 2.
    assert_eq!(h.file_length as usize, image.len());
}

// §11.1: byte 0 is the version. data-model.md: "Must be 3, otherwise load is refused".
#[test]
fn any_version_but_3_is_refused() {
    for version in [0u8, 1, 2, 4, 5, 6, 7, 8, 9, 255] {
        let mut image = image();
        image[0] = version;
        assert_eq!(parse(&image), Err(LoadError::UnsupportedVersion(version)));
        assert!(matches!(
            Machine::load(image, OPTIONS),
            Err(LoadError::UnsupportedVersion(v)) if v == version
        ));
    }
}

// §11: the header is 64 bytes; anything shorter cannot be a story file.
#[test]
fn a_file_shorter_than_the_header_is_too_short() {
    assert_eq!(parse(&[]), Err(LoadError::TooShort));
    assert_eq!(parse(&[3]), Err(LoadError::TooShort));
    assert_eq!(parse(&image()[..63]), Err(LoadError::TooShort));
}

// data-model.md: "Each must lie inside the file".
#[test]
fn a_table_address_outside_the_file_is_a_bad_header() {
    let cases = [
        (0x04, HeaderField::HighMemoryBase),
        (0x06, HeaderField::InitialProgramCounter),
        (0x08, HeaderField::Dictionary),
        (0x0A, HeaderField::ObjectTable),
        (0x0C, HeaderField::Globals),
        (0x0E, HeaderField::StaticMemoryBase),
        (0x18, HeaderField::Abbreviations),
    ];
    for (offset, field) in cases {
        let mut image = image();
        let len = image.len() as u16;
        put(&mut image, offset, len);
        assert_eq!(parse(&image), Err(LoadError::BadHeader(field)), "offset {offset:#x}");
    }
}

// §1.1: dynamic memory must contain at least the 64 header bytes.
// data-model.md: "static base must be at least 64".
#[test]
fn a_static_base_inside_the_header_is_a_bad_header() {
    let mut image = image();
    put(&mut image, 0x0E, 63);
    assert_eq!(parse(&image), Err(LoadError::BadHeader(HeaderField::StaticMemoryBase)));
    put(&mut image, 0x0E, 64);
    assert!(parse(&image).is_ok());
}

// §15 verify: length and checksum are for `verify`. data-model.md: "a mismatch with
// the file size is not a load error".
#[test]
fn a_wrong_length_or_checksum_does_not_refuse_the_load() {
    let mut image = image();
    put(&mut image, 0x1A, 0xFFFF);
    put(&mut image, 0x1C, 0x1234);
    assert!(Machine::load(image.clone(), OPTIONS).is_ok());
    put(&mut image, 0x1A, 0);
    assert!(Machine::load(image, OPTIONS).is_ok());
}

// §11.1 Flags 1, versions 1 to 3: the interpreter owns bit 4 (status line not
// available), bit 5 (screen splitting available) and bit 6 (variable-pitch font is
// the default). The game owns bit 1 (status line type) and the rest stay as stored.
#[test]
fn load_sets_the_interpreter_owned_flags_and_leaves_the_others() {
    let mut image = image();
    image[1] = 0b0111_0010;
    let machine = Machine::load(image, OPTIONS).unwrap();
    assert_eq!(machine.peek(1), 0b0000_0010);

    let mut image = self::image();
    image[1] = 0b1000_1101;
    let machine = Machine::load(image, OPTIONS).unwrap();
    assert_eq!(machine.peek(1), 0b1000_1101);
}

// §11.1 bytes 0x32-0x33: an interpreter conforming to Standard 1.1 writes 1 and 1.
#[test]
fn load_writes_the_standard_revision_number() {
    let machine = Machine::load(image(), OPTIONS).unwrap();
    assert_eq!((machine.peek(0x32), machine.peek(0x33)), (1, 1));
}

// Loading must not disturb anything else in the file.
#[test]
fn load_changes_nothing_but_the_interpreter_fields() {
    let image = image();
    let machine = Machine::load(image.clone(), OPTIONS).unwrap();
    for (at, &byte) in image.iter().enumerate() {
        if at == 0x01 || at == 0x32 || at == 0x33 {
            continue;
        }
        assert_eq!(machine.peek(at as u32), byte, "byte {at:#x}");
    }
}

// The committed Zork I file is a well-formed version 3 story (stories/zork1/SOURCE.md).
#[test]
fn zork1_header_is_accepted() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");
    let story = std::fs::read(path).unwrap();
    let h = parse(&story).unwrap();
    assert_eq!((h.version, h.release, &h.serial), (3, 119, b"880429"));
    assert_eq!(h.file_length, 86_838);
}
