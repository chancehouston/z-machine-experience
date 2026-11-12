//! The read-only view of the machine (feature 003, T004;
//! specs/003-introspection/contracts/rust-api.md, "Addition to `zmachine-core`").

mod common;

use common::*;
use zmachine_core::{Event, Machine};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Story, Story5};

/// A story that stores 0xBEEF in a scratch word, then waits for a key (version 5)
/// or quits (version 3).
fn storing_story5(version: u8) -> (Vec<u8>, u16) {
    let mut story = Story5::new(version);
    let scratch = story.reserve(4);
    story.main(&cat(&[
        op::var(op::STOREW, &[L(scratch), S(0), L(0xBEEF)]),
        read_char(g(0)),
        quit(),
    ]));
    (story.build(), scratch)
}

// rust-api.md: `version` is 3, 5 or 8 and `len` is the number of bytes in the file.
#[test]
fn a_view_reports_the_version_and_length() {
    let mut v3 = Story::new();
    v3.main(&quit());
    let image = v3.build();
    let m = Machine::load(image.clone(), OPTIONS).unwrap();
    assert_eq!(m.view().version(), 3);
    assert_eq!(m.view().len(), image.len());
    assert!(!m.view().is_empty());
    for version in [5u8, 8] {
        let (image, _) = storing_story5(version);
        let m = Machine::load(image.clone(), OPTIONS).unwrap();
        assert_eq!(m.view().version(), version);
        assert_eq!(m.view().len(), image.len());
    }
}

// rust-api.md: `byte` and `word` read memory as it is now; `shipped_byte` and
// `shipped_word` read the file as loaded, whatever the game has since written.
#[test]
fn a_view_shows_memory_now_and_the_file_as_shipped() {
    let (image, scratch) = storing_story5(5);
    let at = scratch as usize;
    let mut m = Machine::load(image.clone(), OPTIONS).unwrap();
    assert_eq!(m.view().word(at), Some(0), "before the game has run");
    assert_eq!(run_until_event(&mut m, BIG), Event::KeyWanted);
    let view = m.view();
    assert_eq!(view.word(at), Some(0xBEEF));
    assert_eq!(view.byte(at), Some(0xBE));
    assert_eq!(view.byte(at + 1), Some(0xEF));
    assert_eq!(view.shipped_word(at), Some(0));
    assert_eq!(view.shipped_byte(at), Some(0));
    // Static memory and the rest of the file read the same either way.
    let last = image.len() - 1;
    assert_eq!(view.byte(last), Some(image[last]));
    assert_eq!(view.shipped_byte(last), Some(image[last]));
    // The serial number, which neither the game nor the interpreter writes.
    assert_eq!(
        view.word(0x12),
        Some(u16::from_be_bytes([image[0x12], image[0x13]]))
    );
    // The interpreter's own header writes are in memory now and not in the file
    // as shipped (§11.1: screen width at 0x21).
    assert_eq!(view.byte(0x21), Some(80));
    assert_eq!(view.shipped_byte(0x21), Some(image[0x21]));
}

// rust-api.md: "An address outside the file gives `None`. Nothing panics."
#[test]
fn addresses_outside_the_file_give_none() {
    let (image, _) = storing_story5(5);
    let m = Machine::load(image.clone(), OPTIONS).unwrap();
    let view = m.view();
    let len = image.len();
    assert!(view.byte(len - 1).is_some());
    assert!(view.word(len - 2).is_some());
    for address in [len, len + 1, usize::MAX - 1, usize::MAX] {
        assert_eq!(view.byte(address), None, "{address}");
        assert_eq!(view.word(address), None, "{address}");
        assert_eq!(view.shipped_byte(address), None, "{address}");
        assert_eq!(view.shipped_word(address), None, "{address}");
    }
    // A word needs both its bytes.
    assert_eq!(view.word(len - 1), None);
    assert_eq!(view.shipped_word(len - 1), None);
}

// rust-api.md: a view borrows the machine and cannot change it. Two may be held
// at once, and reading through one leaves the machine's next event as it was.
#[test]
fn views_are_shared_borrows() {
    let (image, scratch) = storing_story5(8);
    let mut m = Machine::load(image, OPTIONS).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::KeyWanted);
    let (a, b) = (m.view(), m.view());
    let sum: u64 = (0..a.len()).map(|i| a.byte(i).unwrap() as u64).sum();
    let again: u64 = (0..b.len()).map(|i| b.byte(i).unwrap() as u64).sum();
    assert!(sum > 0, "the view reads real bytes");
    assert_eq!(sum, again);
    assert_eq!(a.word(scratch as usize), b.word(scratch as usize));
    m.provide_key(zmachine_core::Key::Return).unwrap();
    assert_eq!(run_until_event(&mut m, BIG), Event::Finished);
}
