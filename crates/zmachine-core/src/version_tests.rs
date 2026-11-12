use super::*;

const ALL: [Version; 3] = [Version::V3, Version::V5, Version::V8];

// §11.1: byte 0 of the header is the version. Only 3, 5 and 8 are run.
#[test]
fn only_versions_3_5_and_8_are_recognised() {
    assert_eq!(Version::from_byte(3), Some(Version::V3));
    assert_eq!(Version::from_byte(5), Some(Version::V5));
    assert_eq!(Version::from_byte(8), Some(Version::V8));
    for other in [0u8, 1, 2, 4, 6, 7, 9, 255] {
        assert_eq!(Version::from_byte(other), None, "version {other}");
    }
    assert_eq!(ALL.map(Version::number), [3, 5, 8]);
}

// §1.2.3: a packed address is 2P in versions 1 to 3, 4P in 4 and 5, 8P in 8.
#[test]
fn packed_address_multiplier() {
    assert_eq!(ALL.map(Version::packed_multiplier), [2, 4, 8]);
    assert_eq!(Version::V5.unpack(0x1234), 0x48D0);
    assert_eq!(Version::V8.unpack(0xFFFF), 0x7FFF8);
    assert_eq!(Version::V3.unpack(0xFFFF), 0x1FFFE);
}

// §11.1.6: the header's length field is the file length divided by 2 in versions 1
// to 3, by 4 in versions 4 and 5, by 8 in versions 6 and later.
#[test]
fn file_length_multiplier() {
    assert_eq!(ALL.map(Version::length_multiplier), [2, 4, 8]);
}

// §1.1.4: the maximum story file is 128K in versions 1 to 3, 256K in 4 and 5, and
// 512K in version 8.
#[test]
fn largest_story_file() {
    assert_eq!(ALL.map(Version::max_story_size), [128 * 1024, 256 * 1024, 512 * 1024]);
}

// §12.3.1, §12.3.2: object entries are 9 bytes with 32 attributes and byte links in
// versions 1 to 3; 14 bytes with 48 attributes and word links from version 4.
// §12.2: 31 property defaults, then 63.
#[test]
fn object_table_layout() {
    assert_eq!(ALL.map(Version::object_entry_size), [9, 14, 14]);
    assert_eq!(ALL.map(Version::attribute_count), [32, 48, 48]);
    assert_eq!(ALL.map(Version::property_defaults), [31, 63, 63]);
    assert_eq!(ALL.map(Version::wide_objects), [false, true, true]);
}

// §13.3, §13.4: dictionary words are 6 Z-characters in 4 bytes in versions 1 to 3,
// 9 Z-characters in 6 bytes from version 4.
#[test]
fn dictionary_key_size() {
    assert_eq!(ALL.map(Version::dictionary_key_len), [4, 6, 6]);
    assert_eq!(ALL.map(Version::dictionary_zchars), [6, 9, 9]);
}

// §5.2.1: routines hold initial values for their locals in versions 1 to 4; from
// version 5 locals start at zero and the header is just the count.
#[test]
fn routine_headers() {
    assert_eq!(ALL.map(Version::routines_have_initial_values), [true, false, false]);
}

// §8.2, §15: the interpreter draws the status line only in version 3; `save` and
// `restore` branch in version 3 and store from version 4; line input is `sread` in
// version 3 and `aread` from version 5.
#[test]
fn behaviours_that_change_after_version_3() {
    assert_eq!(ALL.map(Version::is_v3), [true, false, false]);
}
