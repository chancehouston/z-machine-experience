//! Tests of the machine-level queries on built and damaged images
//! (docs/introspection.md sections 4, 5 and 1.6, cited I§).

use crate::{Introspector, Unknown};
use zmachine_core::{LoadOptions, Machine};
use zmachine_testkit::op;
use zmachine_testkit::{Object5, Story5};

const OPTIONS: LoadOptions = LoadOptions { seed: Some(1), entropy: 0 };

/// 1 Room > (2 lamp > 4 battery, 3 box); 5 loose. Returns the image and the
/// address of the first object's entry.
fn tree() -> (Vec<u8>, usize) {
    let mut s = Story5::new(5);
    s.word("lamp").word("take").word("north");
    let mut room = Object5::new("Room");
    room.child = 2;
    let mut lamp = Object5::new("lamp");
    (lamp.parent, lamp.sibling, lamp.child) = (1, 3, 4);
    let mut boxo = Object5::new("");
    boxo.parent = 1;
    let mut battery = Object5::new("battery");
    battery.parent = 2;
    for o in [room, lamp, boxo, battery, Object5::new("loose")] {
        s.object(o);
    }
    s.main(&op::op0(op::QUIT));
    let entries = s.object_table_address() as usize + 126;
    (s.build(), entries)
}

/// Sets the word at `offset` within object `n`'s 14-byte entry.
fn with(image: &[u8], entries: usize, n: usize, offset: usize, value: u16) -> Vec<u8> {
    let mut image = image.to_vec();
    let at = entries + (n - 1) * 14 + offset;
    image[at..at + 2].copy_from_slice(&value.to_be_bytes());
    image
}

const PARENT: usize = 6;
const SIBLING: usize = 8;
const CHILD: usize = 10;

fn objects(image: Vec<u8>) -> Vec<crate::Object> {
    let m = Machine::load(image, OPTIONS).unwrap();
    let view = m.view();
    Introspector::new(&view).object_tree(&view).value.expect("the tree is answered")
}

// I§4: the plain case, and an object with no name.
#[test]
fn a_small_tree() {
    let (image, _) = tree();
    let o = objects(image);
    assert_eq!(o.len(), 5);
    assert_eq!(o[0].children, Ok(vec![2, 3]));
    assert_eq!(o[1].children, Ok(vec![4]));
    assert_eq!(o[1].parent, Ok(Some(1)));
    assert_eq!(o[4].parent, Ok(None));
    assert_eq!(o[2].name, "");
    assert_eq!(o[2].printed_name, Ok(String::new()));
}

// I§4, I§1.6: a sibling chain that loops is `damaged` for that parent, and the
// other objects are still given.
#[test]
fn a_loop_of_siblings_is_damaged_not_endless() {
    let (image, entries) = tree();
    // 3's sibling is 2, whose sibling is 3.
    let o = objects(with(&image, entries, 3, SIBLING, 2));
    assert_eq!(o[0].children, Err(Unknown::Damaged));
    assert_eq!(o[1].children, Ok(vec![4]));
    assert_eq!(o[3].parent, Ok(Some(2)));
}

// I§4, I§1.6: a child or sibling number past the last object.
#[test]
fn a_link_past_the_last_object_is_damaged() {
    let (image, entries) = tree();
    let o = objects(with(&image, entries, 1, CHILD, 200));
    assert_eq!(o[0].children, Err(Unknown::Damaged));
    let o = objects(with(&image, entries, 2, SIBLING, 0xFFFF));
    assert_eq!(o[0].children, Err(Unknown::Damaged));
    assert_eq!(o[1].children, Ok(vec![4]), "the lamp's own children are intact");
    let o = objects(with(&image, entries, 4, PARENT, 77));
    assert_eq!(o[3].parent, Err(Unknown::Damaged));
}

// I§4: a child whose own parent field names someone else. The two stored facts
// disagree, so the list is `damaged`.
#[test]
fn a_child_whose_parent_disagrees_is_damaged() {
    let (image, entries) = tree();
    let o = objects(with(&image, entries, 3, PARENT, 5));
    assert_eq!(o[0].children, Err(Unknown::Damaged), "3 is in 1's chain but says its parent is 5");
    assert_eq!(o[2].parent, Ok(Some(5)), "what is stored is reported");
    assert_eq!(o[4].children, Ok(vec![]), "5's own chain is empty");
}

// I§4: a parent that is its own ancestor is reported as stored; nothing follows
// parents without bound.
#[test]
fn a_loop_of_parents_is_reported_as_stored() {
    let (image, entries) = tree();
    let o = objects(with(&image, entries, 1, PARENT, 4));
    assert_eq!(o[0].parent, Ok(Some(4)));
    assert_eq!(o.len(), 5);
}

// I§5: an unsorted dictionary (a negative count, §13.2) is listed in table order.
#[test]
fn an_unsorted_dictionary_is_listed_in_table_order() {
    let mut s = Story5::new(5);
    s.unsorted_dictionary();
    s.word("zebra").word("apple").word("mango");
    s.main(&op::op0(op::QUIT));
    let m = Machine::load(s.build(), OPTIONS).unwrap();
    let view = m.view();
    let words: Vec<String> = Introspector::new(&view).dictionary(&view).value.unwrap().into_iter().map(|w| w.word).collect();
    assert_eq!(words, ["zebra", "apple", "mango"]);
}

// I§5, I§1.6: an entry length shorter than the text it must hold, and a table
// that runs off the end of the file, are `damaged`.
#[test]
fn a_damaged_dictionary_is_damaged() {
    let mut s = Story5::new(5);
    s.word("lamp");
    s.main(&op::op0(op::QUIT));
    let dictionary = s.dictionary_address() as usize;
    let image = s.build();
    let separators = image[dictionary] as usize;
    for (offset, value) in [(0usize, 3u8), (1, 0x7F)] {
        let mut damaged = image.clone();
        damaged[dictionary + 1 + separators + offset] = value;
        let m = Machine::load(damaged, OPTIONS).unwrap();
        let view = m.view();
        let answer = Introspector::new(&view).dictionary(&view);
        assert_eq!(answer.value.err(), Some(Unknown::Damaged), "byte {offset} set to {value}");
    }
}

// I§5: for a story file of no recognised family the parts of speech are unknown.
#[test]
fn parts_of_speech_are_unknown_without_a_family() {
    let (image, _) = tree();
    let m = Machine::load(image, OPTIONS).unwrap();
    let view = m.view();
    let words = Introspector::new(&view).dictionary(&view).value.unwrap();
    assert_eq!(words.len(), 3);
    assert!(words.iter().all(|w| w.parts == Err(Unknown::UnrecognisedFamily) && w.direction.is_none()));
}
