use super::*;

fn memory() -> Memory {
    // 0x100 bytes, each holding its own low address byte; static memory from 0x80.
    Memory::new((0..=255u8).collect(), 0x80)
}

// §1.1: memory is an array of bytes addressed from 0.
#[test]
fn bytes_read_back_anywhere_in_the_file() {
    let m = memory();
    assert_eq!(m.len(), 0x100);
    assert_eq!(m.read_byte(0), Ok(0));
    assert_eq!(m.read_byte(0x7F), Ok(0x7F));
    assert_eq!(m.read_byte(0x80), Ok(0x80));
    assert_eq!(m.read_byte(0xFF), Ok(0xFF));
}

// §2.1: a word is two bytes, most significant first.
#[test]
fn words_are_big_endian() {
    let m = memory();
    assert_eq!(m.read_word(0x12), Ok(0x1213));
    assert_eq!(m.read_word(0xFE), Ok(0xFEFF));
}

// Reading past the end of the file is an error, never a panic.
#[test]
fn reads_outside_the_file_are_errors() {
    let m = memory();
    assert_eq!(m.read_byte(0x100), Err(MemoryError::ReadOutsideFile(0x100)));
    assert_eq!(m.read_byte(u32::MAX), Err(MemoryError::ReadOutsideFile(u32::MAX)));
    // A word whose second byte is outside the file.
    assert_eq!(m.read_word(0xFF), Err(MemoryError::ReadOutsideFile(0xFF)));
    assert_eq!(m.read_word(u32::MAX), Err(MemoryError::ReadOutsideFile(u32::MAX)));
}

// §1.1.1: dynamic memory, below the static memory base, can be written.
#[test]
fn writes_below_the_static_base_succeed() {
    let mut m = memory();
    assert_eq!(m.write_byte(0x40, 0xAB), Ok(()));
    assert_eq!(m.read_byte(0x40), Ok(0xAB));
    assert_eq!(m.write_word(0x7E, 0xBEEF), Ok(()));
    assert_eq!(m.read_word(0x7E), Ok(0xBEEF));
}

// §1.1.2, §1.1.3: static and high memory cannot be written by the game.
#[test]
fn writes_at_or_above_the_static_base_are_errors() {
    let mut m = memory();
    assert_eq!(m.write_byte(0x80, 1), Err(MemoryError::WriteOutsideDynamicMemory(0x80)));
    assert_eq!(m.write_byte(0xFF, 1), Err(MemoryError::WriteOutsideDynamicMemory(0xFF)));
    assert_eq!(m.write_byte(0x1000, 1), Err(MemoryError::WriteOutsideDynamicMemory(0x1000)));
    assert_eq!(m.read_byte(0x80), Ok(0x80), "a refused write changes nothing");
}

// A word that starts in dynamic memory but ends in static memory is refused whole.
#[test]
fn a_word_write_straddling_the_static_base_is_refused_whole() {
    let mut m = memory();
    assert_eq!(m.write_word(0x7F, 0xFFFF), Err(MemoryError::WriteOutsideDynamicMemory(0x7F)));
    assert_eq!(m.read_byte(0x7F), Ok(0x7F));
    assert_eq!(m.read_byte(0x80), Ok(0x80));
}

// The dynamic region is exactly the bytes below the static base (used by save).
#[test]
fn dynamic_is_the_region_below_the_static_base() {
    let mut m = memory();
    assert_eq!(m.static_base(), 0x80);
    assert_eq!(m.dynamic().len(), 0x80);
    m.dynamic_mut()[3] = 0x99;
    assert_eq!(m.read_byte(3), Ok(0x99));
}
