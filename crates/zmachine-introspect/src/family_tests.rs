//! Recognition, rule by rule, on built story files (docs/introspection.md
//! sections 3.1 and 3.2, cited I§3.1 and I§3.2).

use crate::fixtures::{Inform, Zil, inf, zl};
use crate::{Family, Introspector};

fn family_of(image: Vec<u8>) -> Family {
    let m = crate::fixtures::run(image);
    let view = m.view();
    Introspector::new(&view).family()
}

// I§3.1: a file that satisfies every rule is of the Inform 6 family, in version
// 5 and in version 8.
#[test]
fn a_file_meeting_every_inform_rule_is_recognised() {
    assert_eq!(family_of(Inform::new().build()), Family::Inform6);
    let mut v8 = Inform::new();
    v8.version = 8;
    assert_eq!(family_of(v8.build()), Family::Inform6);
}

// I§3.1 rule 1: the compiler's version stamp at 0x3C: digit, full stop, two digits.
#[test]
fn inform_rule_1_the_compiler_stamp() {
    for stamp in [*b"\0\0\0\0", *b"6x44", *b"a.44", *b"6.4 ", *b"6.4a"] {
        let mut f = Inform::new();
        f.stamp = stamp;
        assert_eq!(family_of(f.build()), Family::Unrecognised, "{stamp:?}");
    }
    let mut f = Inform::new();
    f.stamp = *b"6.31";
    assert_eq!(family_of(f.build()), Family::Inform6);
}

// I§3.1 rule 2: objects 1 to 4 are the four metaclasses.
#[test]
fn inform_rule_2_the_metaclasses() {
    for (number, name) in [(1u16, "Klass"), (2, "object"), (3, ""), (4, "Strings")] {
        let mut f = Inform::new();
        f.object(number).name = name.to_string();
        assert_eq!(family_of(f.build()), Family::Unrecognised, "object {number} named {name:?}");
    }
}

// I§3.1 rule 3: exactly one object with each of the library's four names.
#[test]
fn inform_rule_3_the_librarys_objects() {
    for number in [inf::COMPASS, inf::PLAYER, inf::DARKNESS, inf::LIBRARY] {
        let mut f = Inform::new();
        f.object(number).name = "something else".to_string();
        assert_eq!(family_of(f.build()), Family::Unrecognised, "object {number} renamed");
    }
    // Two objects named `compass`: which is meant is not settled.
    let mut f = Inform::new();
    f.object(inf::LAMP).name = "compass".to_string();
    assert_eq!(family_of(f.build()), Family::Unrecognised);
    let mut f = Inform::new();
    f.object(inf::LAMP).name = "(self object)".to_string();
    assert_eq!(family_of(f.build()), Family::Unrecognised);
}

// I§3.1 rule 4: every child of the compass provides property 21 with a distinct
// value from 4 to 63 that is not a property the child itself provides.
#[test]
fn inform_rule_4_the_compass() {
    let mut f = Inform::new();
    f.without_prop(inf::S, 21);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "a child without property 21");
    let mut f = Inform::new();
    f.prop(inf::S, 21, vec![0, 7]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "two children with one value");
    for value in [0u16, 3, 64, 0x1234] {
        let mut f = Inform::new();
        f.prop(inf::S, 21, value.to_be_bytes().to_vec());
        assert_eq!(family_of(f.build()), Family::Unrecognised, "value {value}");
    }
    let mut f = Inform::new();
    f.prop(inf::S, 21, vec![0, 21]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "a value the child itself provides");
    let mut f = Inform::new();
    f.prop(inf::S, 21, vec![8]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "a property of one byte");
    // A compass with no children gives no directions to check, and no exits.
    let mut f = Inform::new();
    for child in [inf::N, inf::S, inf::IN, inf::OUT] {
        f.object(child).parent = 0;
    }
    assert_eq!(family_of(f.build()), Family::Unrecognised, "an empty compass");
}

// I§3.1 rule 5: the attributes are numbered as the library declares them.
#[test]
fn inform_rule_5_attribute_numbering() {
    let mut f = Inform::new();
    f.attrs(inf::PLAYER, &[0, 3, 16, 23, 30]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "the player with an extra attribute");
    let mut f = Inform::new();
    f.attrs(inf::PLAYER, &[1, 4, 17, 24]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "every attribute shifted by one");
    let mut f = Inform::new();
    f.attrs(inf::COMPASS, &[4]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "the compass not concealed");
    let mut f = Inform::new();
    f.attrs(inf::N, &[18]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "a direction not scenery");
}

// I§3.1 rule 6: exactly one global holds the library object as shipped.
#[test]
fn inform_rule_6_the_location_variable() {
    let mut f = Inform::new();
    f.globals.clear();
    assert_eq!(family_of(f.build()), Family::Unrecognised, "no global holds it");
    let mut f = Inform::new();
    f.globals.push((40, inf::LIBRARY));
    assert_eq!(family_of(f.build()), Family::Unrecognised, "two globals hold it");
}

// I§3: recognition reads the story file as shipped. What the game has since
// written does not change the family.
#[test]
fn recognition_reads_the_file_as_shipped() {
    let mut f = Inform::new();
    // Start-up overwrites the location variable (as every game does) and gives
    // the player another attribute.
    f.setup.extend(crate::fixtures::set_attr(inf::PLAYER, 30));
    assert_eq!(family_of(f.build()), Family::Inform6);
}

// I§3.2: a file that satisfies every rule is of the ZIL family, with the object
// that holds the rooms carrying a direction property itself.
#[test]
fn a_file_meeting_every_zil_rule_is_recognised() {
    assert_eq!(family_of(Zil::new().build()), Family::Zil);
}

// I§3.2 rule 1: version 3, and not carrying an Inform compiler stamp.
#[test]
fn zil_rule_1_not_inform() {
    let mut f = Zil::new();
    f.version_stamp = *b"6.44";
    assert_eq!(family_of(f.build()), Family::Unrecognised);
    // A version 5 file is never ZIL, whatever else it holds: the Inform fixture
    // with its stamp removed is neither family.
    let mut f = Inform::new();
    f.stamp = [0; 4];
    assert_eq!(family_of(f.build()), Family::Unrecognised);
}

// I§3.2 rule 3: at least four direction words.
#[test]
fn zil_rule_3_direction_words() {
    let mut f = Zil::new();
    // Leave three: north (with its synonym n), south, east.
    f.words.retain(|(w, _)| !["west", "down", "up"].contains(w));
    for number in [zl::KITCHEN, zl::CELLAR] {
        f.object(number).props.retain(|(p, _)| ![zl::WEST, zl::DOWN, zl::UP].contains(p));
    }
    assert_eq!(family_of(f.build()), Family::Unrecognised, "three directions, four words");
    let mut f = Zil::new();
    f.words.retain(|(_, data)| data[0] & 0x10 == 0);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "no direction words");
}

// I§3.2 rule 4: every object with a direction property, parentless ones aside,
// has the same parent, which has no parent.
#[test]
fn zil_rule_4_one_parent_for_all_rooms() {
    let mut f = Zil::new();
    f.prop(zl::BOAT, zl::NORTH, vec![zl::KITCHEN as u8]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "a direction property on a thing in a room");
    let mut f = Zil::new();
    f.object(zl::ROOMS).parent = zl::DIAMOND as u8;
    assert_eq!(family_of(f.build()), Family::Unrecognised, "the rooms' parent has a parent");
    // A parentless object with a direction property is left aside, as the rule says.
    let mut f = Zil::new();
    f.prop(zl::DIAMOND, zl::NORTH, vec![99]);
    assert_eq!(family_of(f.build()), Family::Zil);
}

// I§3.2 rule 5: a direction property is 1 to 5 bytes, and a 1-byte value is a room.
#[test]
fn zil_rule_5_the_shape_of_exits() {
    let mut f = Zil::new();
    f.prop(zl::HALL, zl::EAST, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "six bytes");
    let mut f = Zil::new();
    f.prop(zl::HALL, zl::EAST, vec![zl::LAMP as u8]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "a one-byte value that is not a room");
    let mut f = Zil::new();
    f.prop(zl::HALL, zl::EAST, vec![200]);
    assert_eq!(family_of(f.build()), Family::Unrecognised, "a one-byte value past the last object");
}

// I§3.1, I§3.2: no file can satisfy both sets of rules. Rule 1 of each excludes
// the other: ZIL needs version 3 without the stamp that Inform needs.
#[test]
fn the_two_families_exclude_each_other() {
    let mut f = Zil::new();
    f.version_stamp = *b"6.44";
    assert_eq!(family_of(f.build()), Family::Unrecognised);
}
