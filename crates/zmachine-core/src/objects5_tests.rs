//! The object table for versions 5 and 8 (feature 002, T011).

use super::*;
use crate::FaultKind;
use crate::header;
use crate::memory::Memory;
use crate::text::Charset;
use crate::version::Version;
use zmachine_testkit::{Object5, Story5, op};

/// Objects:  1 room ── 2 lamp ── 3 box ── 4 coin     5 shelf     ...     300 far
fn fixture() -> (Memory, Objects) {
    let mut story = Story5::new(5);
    story.prop_default(40, 0x4040).prop_default(63, 0x6363);
    let mut room = Object5::new("room");
    room.child = 2;
    room.attributes = 1 << 47 | 1 << 15 | 1; // attributes 0, 32 and 47
    room.props = vec![
        (63, vec![0xAA, 0xBB]),
        (40, vec![0x7F]),
        (33, vec![1, 2, 3]),
        (5, vec![0x55; 64]),
    ];
    story.object(room);
    for (name, sibling) in [("lamp", 3), ("box", 4), ("coin", 0)] {
        let mut o = Object5::new(name);
        o.parent = 1;
        o.sibling = sibling;
        story.object(o);
    }
    story.object(Object5::new("shelf"));
    for _ in 6..300 {
        story.object(Object5::new(""));
    }
    let mut far = Object5::new("far");
    far.parent = 5;
    story.object(far);
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let h = header::parse(&image).unwrap();
    (
        Memory::new(image, h.static_base as usize),
        Objects::for_version(h.objects, h.v),
    )
}

fn children(m: &Memory, o: &Objects, parent: u16) -> Vec<u16> {
    let mut out = Vec::new();
    let mut c = o.child(m, parent).unwrap();
    while c != 0 {
        out.push(c);
        c = o.sibling(m, c).unwrap();
    }
    out
}

// §12.3.2: from version 4 the links are words at offsets 6, 8 and 10 of a 14-byte
// entry, so object numbers above 255 work.
#[test]
fn links_are_words_and_reach_objects_above_255() {
    let (mut m, o) = fixture();
    assert_eq!(children(&m, &o, 1), vec![2, 3, 4]);
    assert_eq!(o.parent(&m, 300).unwrap(), 5);
    // The fixture leaves object 5's child link unset; insert 300 properly.
    o.insert(&mut m, 300, 5).unwrap();
    assert_eq!(children(&m, &o, 5), vec![300]);
    o.insert(&mut m, 3, 300).unwrap();
    assert_eq!(children(&m, &o, 300), vec![3]);
    assert_eq!(children(&m, &o, 1), vec![2, 4]);
    assert_eq!(o.parent(&m, 3).unwrap(), 300);
    o.remove(&mut m, 300).unwrap();
    assert_eq!(children(&m, &o, 5), vec![]);
    assert_eq!(
        children(&m, &o, 300),
        vec![3],
        "an object keeps its children when removed"
    );
}

// §15 remove_obj, insert_obj: the same rules as version 3, on the wider table.
#[test]
fn remove_first_middle_and_last_child() {
    for (removed, left) in [(2u16, vec![3u16, 4]), (3, vec![2, 4]), (4, vec![2, 3])] {
        let (mut m, o) = fixture();
        o.remove(&mut m, removed).unwrap();
        assert_eq!(children(&m, &o, 1), left, "after removing {removed}");
        assert_eq!(
            (
                o.parent(&m, removed).unwrap(),
                o.sibling(&m, removed).unwrap()
            ),
            (0, 0)
        );
    }
}

// §12.3.2: 48 attributes, numbered from the top bit of the first of six bytes.
#[test]
fn forty_eight_attributes() {
    let (mut m, o) = fixture();
    for a in 0..48 {
        assert_eq!(
            o.test_attr(&m, 1, a).unwrap(),
            [0, 32, 47].contains(&a),
            "attribute {a}"
        );
    }
    for a in 0..48 {
        o.set_attr(&mut m, 5, a).unwrap();
        for b in 0..48 {
            assert_eq!(o.test_attr(&m, 5, b).unwrap(), b == a, "set {a}, test {b}");
        }
        o.clear_attr(&mut m, 5, a).unwrap();
    }
    // Attribute 48 does not exist: testing it is false and setting it does nothing,
    // in particular it does not reach into the parent link that follows.
    assert!(!o.test_attr(&m, 2, 48).unwrap());
    o.set_attr(&mut m, 2, 48).unwrap();
    assert_eq!(o.parent(&m, 2).unwrap(), 1);
}

// §12.4.2: property numbers up to 63; one size byte for lengths 1 and 2, two for
// longer. §12.2: 63 defaults.
#[test]
fn get_prop_reads_values_and_defaults() {
    let (m, o) = fixture();
    assert_eq!(o.get_prop(&m, 1, 63).unwrap(), 0xAABB);
    assert_eq!(o.get_prop(&m, 1, 40).unwrap(), 0x007F);
    assert_eq!(o.get_prop(&m, 2, 40).unwrap(), 0x4040, "default");
    assert_eq!(o.get_prop(&m, 2, 63).unwrap(), 0x6363, "default 63");
    assert_eq!(o.get_prop(&m, 2, 1).unwrap(), 0);
}

// §15 get_prop_addr, get_prop_len: the length comes from the byte before the data,
// which for a long property is its second size byte. §12.4.2.1.1: 0 means 64.
#[test]
fn property_addresses_and_lengths_in_both_size_forms() {
    let (m, o) = fixture();
    let len = |number: u16| {
        let at = o.get_prop_addr(&m, 1, number).unwrap();
        assert_ne!(at, 0, "property {number}");
        o.prop_len(&m, at).unwrap()
    };
    assert_eq!((len(63), len(40), len(33), len(5)), (2, 1, 3, 64));
    assert_eq!(o.get_prop_addr(&m, 1, 62).unwrap(), 0);
    assert_eq!(
        o.prop_len(&m, 0).unwrap(),
        0,
        "Standard 1.1: get_prop_len 0 is 0"
    );
    let at33 = o.get_prop_addr(&m, 1, 33).unwrap() as u32;
    assert_eq!(
        [
            m.read_byte(at33).unwrap(),
            m.read_byte(at33 + 1).unwrap(),
            m.read_byte(at33 + 2).unwrap()
        ],
        [1, 2, 3]
    );
}

// §15 get_next_prop and put_prop on the version 4+ table.
#[test]
fn next_property_and_put_prop() {
    let (mut m, o) = fixture();
    let order: Vec<u16> = std::iter::successors(Some(o.get_next_prop(&m, 1, 0).unwrap()), |&p| {
        (p != 0).then(|| o.get_next_prop(&m, 1, p).unwrap())
    })
    .collect();
    assert_eq!(order, vec![63, 40, 33, 5, 0]);
    assert_eq!(o.get_next_prop(&m, 1, 62), Err(FaultKind::MissingProperty));

    o.put_prop(&mut m, 1, 63, 0x1234).unwrap();
    o.put_prop(&mut m, 1, 40, 0xABCD).unwrap();
    assert_eq!(
        (
            o.get_prop(&m, 1, 63).unwrap(),
            o.get_prop(&m, 1, 40).unwrap()
        ),
        (0x1234, 0xCD)
    );
    assert_eq!(
        o.put_prop(&mut m, 2, 63, 1),
        Err(FaultKind::MissingProperty)
    );
}

// §12.4: the short name is found the same way in every version.
#[test]
fn short_names() {
    let (m, o) = fixture();
    let name = |object| {
        let at = o.name_address(&m, object).unwrap();
        at.map(|a| Charset::default().decode(&m, a, 0x40).unwrap().0)
    };
    assert_eq!(name(1).as_deref(), Some("room"));
    assert_eq!(name(300).as_deref(), Some("far"));
    assert_eq!(name(6), None);
}

// Version 8's object table is version 5's.
#[test]
fn version_8_uses_the_same_layout() {
    let mut story = Story5::new(8);
    let mut a = Object5::new("a");
    a.child = 2;
    a.props = vec![(50, vec![9, 9, 9])];
    story.object(a);
    let mut b = Object5::new("b");
    b.parent = 1;
    story.object(b);
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let h = header::parse(&image).unwrap();
    assert_eq!(h.v, Version::V8);
    let m = Memory::new(image, h.static_base as usize);
    let o = Objects::for_version(h.objects, h.v);
    assert_eq!((o.child(&m, 1).unwrap(), o.parent(&m, 2).unwrap()), (2, 1));
    assert_eq!(
        o.prop_len(&m, o.get_prop_addr(&m, 1, 50).unwrap()).unwrap(),
        3
    );
}

// The real thing: Advent's object tree, read from both builds. At the start the
// lamp, keys, food and bottle are inside the building.
#[test]
fn advents_object_tree() {
    for version in [5u8, 8] {
        let path = format!(
            "{}/../../stories/advent/advent.z{version}",
            env!("CARGO_MANIFEST_DIR")
        );
        let story = std::fs::read(path).unwrap();
        let h = header::parse(&story).unwrap();
        let m = Memory::new(story, h.static_base as usize);
        let o = Objects::for_version(h.objects, h.v);
        let c = Charset::default();
        let mut by_name = std::collections::HashMap::new();
        for object in 1..=300u16 {
            if let Ok(Some(at)) = o.name_address(&m, object)
                && let Ok((name, _)) = c.decode(&m, at, h.abbreviations)
            {
                by_name.insert(name, object);
            }
        }
        let building = by_name["Inside Building"];
        let inside: Vec<u16> = children(&m, &o, building);
        for thing in ["brass lantern", "set of keys", "tasty food", "small bottle"] {
            assert!(
                inside.contains(&by_name[thing]),
                "version {version}: {thing} is in the building"
            );
        }
    }
}
