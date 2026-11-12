use super::*;
use crate::memory::Memory;
use crate::testing::{g, run_story, skip_if};
use crate::{DiagnosticKind, Event, FaultKind, header, text};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Object, Story};

/// Objects:  1 room ── 2 lamp ── 3 box ── 4 coin     5 shelf (no children)
///           (2, 3 and 4 are the children of 1, in that order)
fn story() -> Story {
    let mut story = Story::new();
    story.prop_default(7, 0x0707).prop_default(31, 0x3131);
    let mut room = Object::new("room");
    room.child = 2;
    room.attributes = 0x8000_0001; // attributes 0 and 31
    room.props = vec![
        (20, vec![0xAA, 0xBB]),
        (12, vec![0x7F]),
        (5, vec![1, 2, 3, 4]),
    ];
    story.object(room);
    for (name, sibling) in [("lamp", 3), ("box", 4), ("coin", 0)] {
        let mut o = Object::new(name);
        o.parent = 1;
        o.sibling = sibling;
        story.object(o);
    }
    story.object(Object::new("shelf"));
    story
}

fn fixture() -> (Memory, Objects) {
    let mut story = story();
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let h = header::parse(&image).unwrap();
    (
        Memory::new(image, h.static_base as usize),
        Objects::new(h.objects),
    )
}

fn family(m: &Memory, o: &Objects, object: u16) -> (u16, u16, u16) {
    (
        o.parent(m, object).unwrap(),
        o.sibling(m, object).unwrap(),
        o.child(m, object).unwrap(),
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

// §12.3.1: parent, sibling and child are bytes 4, 5 and 6 of a 9-byte entry.
#[test]
fn tree_links_are_read_from_the_object_entry() {
    let (m, o) = fixture();
    assert_eq!(family(&m, &o, 1), (0, 0, 2));
    assert_eq!(family(&m, &o, 2), (1, 3, 0));
    assert_eq!(family(&m, &o, 4), (1, 0, 0));
    assert_eq!(children(&m, &o, 1), vec![2, 3, 4]);
}

// §15 remove_obj: detach from the parent; the object keeps its own children and
// the parent's remaining children stay in order.
#[test]
fn remove_first_middle_and_last_child() {
    for (removed, left) in [(2u16, vec![3u16, 4]), (3, vec![2, 4]), (4, vec![2, 3])] {
        let (mut m, o) = fixture();
        o.remove(&mut m, removed).unwrap();
        assert_eq!(children(&m, &o, 1), left, "after removing {removed}");
        assert_eq!(o.parent(&m, removed).unwrap(), 0);
        assert_eq!(o.sibling(&m, removed).unwrap(), 0);
    }
}

// Removing an object with no parent changes nothing.
#[test]
fn removing_an_orphan_is_harmless() {
    let (mut m, o) = fixture();
    o.remove(&mut m, 5).unwrap();
    assert_eq!(family(&m, &o, 5), (0, 0, 0));
    assert_eq!(children(&m, &o, 1), vec![2, 3, 4]);
}

// §15 insert_obj: "Moves object O to become the first child of the destination
// object D ... All children of O move with it."
#[test]
fn insert_makes_the_object_the_first_child() {
    let (mut m, o) = fixture();
    o.insert(&mut m, 3, 5).unwrap();
    assert_eq!(children(&m, &o, 5), vec![3]);
    assert_eq!(children(&m, &o, 1), vec![2, 4]);
    o.insert(&mut m, 4, 5).unwrap();
    assert_eq!(children(&m, &o, 5), vec![4, 3]);
    assert_eq!(children(&m, &o, 1), vec![2]);

    // Children move with their parent.
    o.insert(&mut m, 5, 2).unwrap();
    assert_eq!(children(&m, &o, 2), vec![5]);
    assert_eq!(children(&m, &o, 5), vec![4, 3]);
}

// Re-inserting an object into its own parent moves it to the front.
#[test]
fn inserting_into_the_current_parent_moves_to_front() {
    let (mut m, o) = fixture();
    o.insert(&mut m, 4, 1).unwrap();
    assert_eq!(children(&m, &o, 1), vec![4, 2, 3]);
    o.insert(&mut m, 4, 1).unwrap();
    assert_eq!(children(&m, &o, 1), vec![4, 2, 3]);
}

// §12.3.1: 32 attributes, numbered from 0 at the top bit of the first byte.
#[test]
fn attributes_are_numbered_from_the_top_bit() {
    let (mut m, o) = fixture();
    assert!(o.test_attr(&m, 1, 0).unwrap());
    assert!(o.test_attr(&m, 1, 31).unwrap());
    for a in 1..31 {
        assert!(!o.test_attr(&m, 1, a).unwrap(), "attribute {a}");
    }
    for a in 0..32 {
        o.set_attr(&mut m, 5, a).unwrap();
        for b in 0..32 {
            assert_eq!(o.test_attr(&m, 5, b).unwrap(), b == a, "set {a}, test {b}");
        }
        o.clear_attr(&mut m, 5, a).unwrap();
        assert!(!o.test_attr(&m, 5, a).unwrap());
    }
    // Setting one object's attribute does not touch its neighbours.
    o.set_attr(&mut m, 2, 31).unwrap();
    assert!(!o.test_attr(&m, 3, 0).unwrap());
    assert_eq!(family(&m, &o, 2), (1, 3, 0));
}

// §12.4.1, §15 get_prop: one byte gives the byte, two give the word; a missing
// property gives its default (§12.2).
#[test]
fn get_prop_reads_values_and_defaults() {
    let (m, o) = fixture();
    assert_eq!(o.get_prop(&m, 1, 20).unwrap(), 0xAABB);
    assert_eq!(o.get_prop(&m, 1, 12).unwrap(), 0x007F);
    assert_eq!(o.get_prop(&m, 1, 7).unwrap(), 0x0707);
    assert_eq!(o.get_prop(&m, 2, 31).unwrap(), 0x3131);
    assert_eq!(o.get_prop(&m, 2, 1).unwrap(), 0);
}

// §15 put_prop: writes a byte or a word according to the property's length; a
// property the object lacks halts the interpreter.
#[test]
fn put_prop_writes_by_length_and_faults_when_missing() {
    let (mut m, o) = fixture();
    o.put_prop(&mut m, 1, 20, 0x1234).unwrap();
    assert_eq!(o.get_prop(&m, 1, 20).unwrap(), 0x1234);
    o.put_prop(&mut m, 1, 12, 0xABCD).unwrap();
    assert_eq!(
        o.get_prop(&m, 1, 12).unwrap(),
        0x00CD,
        "least significant byte"
    );
    assert_eq!(
        o.get_prop(&m, 1, 20).unwrap(),
        0x1234,
        "the neighbour is untouched"
    );
    assert_eq!(o.put_prop(&mut m, 1, 7, 1), Err(FaultKind::MissingProperty));
    assert_eq!(
        o.put_prop(&mut m, 2, 20, 1),
        Err(FaultKind::MissingProperty)
    );
}

// §15 get_prop_addr: the byte address of the property data, or 0 if absent.
// §15 get_prop_len: the length from that address; address 0 gives 0 (Standard 1.1).
#[test]
fn property_addresses_and_lengths() {
    let (m, o) = fixture();
    let a20 = o.get_prop_addr(&m, 1, 20).unwrap();
    let a12 = o.get_prop_addr(&m, 1, 12).unwrap();
    let a5 = o.get_prop_addr(&m, 1, 5).unwrap();
    assert_eq!(m.read_word(a20 as u32).unwrap(), 0xAABB);
    assert_eq!(m.read_byte(a12 as u32).unwrap(), 0x7F);
    assert_eq!(
        (a12 - a20, a5 - a12),
        (3, 2),
        "descending order, one size byte each"
    );
    assert_eq!(o.get_prop_addr(&m, 1, 7).unwrap(), 0);
    assert_eq!(get_prop_len(&m, a20).unwrap(), 2);
    assert_eq!(get_prop_len(&m, a12).unwrap(), 1);
    assert_eq!(get_prop_len(&m, a5).unwrap(), 4);
    assert_eq!(get_prop_len(&m, 0).unwrap(), 0);
}

// §15 get_next_prop: 0 gives the first property; the last gives 0; asking after a
// property the object lacks halts the interpreter.
#[test]
fn get_next_prop_walks_in_descending_order() {
    let (m, o) = fixture();
    assert_eq!(o.get_next_prop(&m, 1, 0).unwrap(), 20);
    assert_eq!(o.get_next_prop(&m, 1, 20).unwrap(), 12);
    assert_eq!(o.get_next_prop(&m, 1, 12).unwrap(), 5);
    assert_eq!(o.get_next_prop(&m, 1, 5).unwrap(), 0);
    assert_eq!(o.get_next_prop(&m, 2, 0).unwrap(), 0);
    assert_eq!(o.get_next_prop(&m, 1, 7), Err(FaultKind::MissingProperty));
}

// §12.4: the property table begins with the short name as a Z-string.
#[test]
fn short_names() {
    let (m, o) = fixture();
    let name = |object| {
        let at = o.name_address(&m, object).unwrap();
        at.map(|a| text::decode_string(&m, a, 0x40).unwrap().0)
    };
    assert_eq!(name(1).as_deref(), Some("room"));
    assert_eq!(name(5).as_deref(), Some("shelf"));
}

// The real thing: Zork I's object table (stories/zork1/SOURCE.md).
#[test]
fn zork1_object_names_and_tree() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");
    let story = std::fs::read(path).unwrap();
    let h = header::parse(&story).unwrap();
    let m = Memory::new(story, h.static_base as usize);
    let o = Objects::new(h.objects);
    let mut names = std::collections::HashMap::new();
    for object in 1..=250u16 {
        if let Some(at) = o.name_address(&m, object).unwrap() {
            let name = text::decode_string(&m, at, h.abbreviations).unwrap().0;
            names.insert(name, object);
        }
    }
    for expected in [
        "West of House",
        "small mailbox",
        "brass lantern",
        "leaflet",
        "troll",
    ] {
        assert!(
            names.contains_key(expected),
            "{expected} missing; have {} names",
            names.len()
        );
    }
    // The mailbox starts in West of House, with the leaflet inside it.
    let (mailbox, house, leaflet) = (
        names["small mailbox"],
        names["West of House"],
        names["leaflet"],
    );
    assert_eq!(o.parent(&m, mailbox).unwrap(), house);
    assert_eq!(o.parent(&m, leaflet).unwrap(), mailbox);
}

// ---- Through the executor: the object instructions (§15) ----

fn run_code(code: Vec<u8>) -> (crate::Machine, Event) {
    let mut story = story();
    let mut main = code;
    main.extend(op::op0(op::QUIT));
    story.main(&main);
    run_story(&mut story)
}

fn store_to(instruction: Vec<u8>, global: u8) -> Vec<u8> {
    [instruction, op::store(g(global))].concat()
}

// §15 get_parent, get_child, get_sibling (the last two branch if the result is
// not 0), jin, test_attr.
#[test]
fn tree_instructions() {
    let mark = op::op2(op::STORE, S(g(9)), S(1));
    let code = [
        store_to(op::op1(op::GET_PARENT, S(3)), 0),
        skip_if(store_to(op::op1(op::GET_CHILD, S(1)), 1), true, &mark),
        skip_if(store_to(op::op1(op::GET_SIBLING, S(4)), 2), false, &mark),
        store_to(op::op1(op::GET_SIBLING, S(2)), 3),
        op::branch(true, 2),
    ]
    .concat();
    let (m, e) = run_code(code);
    assert_eq!(e, Event::Finished);
    assert_eq!(
        (m.global(0), m.global(1), m.global(2), m.global(3)),
        (1, 2, 0, 3)
    );
    assert_eq!(m.global(9), 0, "both conditional skips were taken");

    let taken = |instruction: Vec<u8>| run_code(skip_if(instruction, true, &mark)).0.global(9) == 0;
    assert!(taken(op::op2(op::JIN, S(3), S(1))));
    assert!(!taken(op::op2(op::JIN, S(3), S(2))));
    assert!(taken(op::op2(op::TEST_ATTR, S(1), S(31))));
    assert!(!taken(op::op2(op::TEST_ATTR, S(1), S(5))));
}

// §15 set_attr, clear_attr, insert_obj, remove_obj, put_prop, get_prop,
// get_prop_addr, get_prop_len, get_next_prop.
#[test]
fn object_changing_instructions() {
    let code = [
        op::op2(op::SET_ATTR, S(5), S(9)),
        op::op2(op::CLEAR_ATTR, S(1), S(0)),
        op::op2(op::INSERT_OBJ, S(3), S(5)),
        op::op1(op::REMOVE_OBJ, S(4)),
        op::var(op::PUT_PROP, &[S(1), S(20), L(0x4321)]),
        store_to(op::op2(op::GET_PROP, S(1), S(20)), 0),
        store_to(op::op2(op::GET_PROP_ADDR, S(1), S(5)), 1),
        store_to(op::op1(op::GET_PROP_LEN, V(g(1))), 2),
        store_to(op::op2(op::GET_NEXT_PROP, S(1), S(20)), 3),
        store_to(op::op1(op::GET_CHILD, S(5)), 4),
        op::branch(true, 2),
        store_to(op::op1(op::GET_PARENT, S(4)), 5),
    ]
    .concat();
    let (m, e) = run_code(code);
    assert_eq!(e, Event::Finished);
    assert_eq!(m.global(0), 0x4321);
    assert_ne!(m.global(1), 0);
    assert_eq!(
        (m.global(2), m.global(3), m.global(4), m.global(5)),
        (4, 12, 3, 0)
    );
    let o = Objects::new(m.header.objects);
    assert!(o.test_attr(&m.memory, 5, 9).unwrap());
    assert!(!o.test_attr(&m.memory, 1, 0).unwrap());
    assert_eq!(children(&m.memory, &o, 1), vec![2]);
}

// §15 put_prop on a property the object lacks.
#[test]
fn put_prop_on_a_missing_property_is_a_fault() {
    let (_, e) = run_code(op::var(op::PUT_PROP, &[S(2), S(20), S(1)]));
    assert!(matches!(e, Event::Fault(f) if f.kind == FaultKind::MissingProperty));
}

// research.md R12: an instruction given object 0 does nothing or yields 0, the game
// continues, and the host can read a diagnostic. The Standard calls these illegal
// (§12.3.1 numbers objects from 1) but published games perform them.
#[test]
fn object_zero_is_tolerated_with_a_diagnostic() {
    let poison = |global: u8| op::op2(op::STORE, S(g(global)), L(0x5555));
    let cases: Vec<(&str, Vec<u8>, Option<u8>)> = vec![
        (
            "get_parent",
            store_to(op::op1(op::GET_PARENT, S(0)), 0),
            Some(0),
        ),
        (
            "get_child",
            [
                store_to(op::op1(op::GET_CHILD, S(0)), 0),
                op::branch(true, 2),
            ]
            .concat(),
            Some(0),
        ),
        (
            "get_sibling",
            [
                store_to(op::op1(op::GET_SIBLING, S(0)), 0),
                op::branch(true, 2),
            ]
            .concat(),
            Some(0),
        ),
        (
            "get_prop",
            store_to(op::op2(op::GET_PROP, S(0), S(20)), 0),
            Some(0),
        ),
        (
            "get_prop_addr",
            store_to(op::op2(op::GET_PROP_ADDR, S(0), S(20)), 0),
            Some(0),
        ),
        (
            "get_next_prop",
            store_to(op::op2(op::GET_NEXT_PROP, S(0), S(0)), 0),
            Some(0),
        ),
        (
            "jin",
            [op::op2(op::JIN, S(0), S(1)), op::branch(true, 2)].concat(),
            None,
        ),
        (
            "test_attr",
            [op::op2(op::TEST_ATTR, S(0), S(1)), op::branch(true, 2)].concat(),
            None,
        ),
        ("set_attr", op::op2(op::SET_ATTR, S(0), S(1)), None),
        ("clear_attr", op::op2(op::CLEAR_ATTR, S(0), S(1)), None),
        (
            "insert_obj object",
            op::op2(op::INSERT_OBJ, S(0), S(1)),
            None,
        ),
        (
            "insert_obj destination",
            op::op2(op::INSERT_OBJ, S(2), S(0)),
            None,
        ),
        ("remove_obj", op::op1(op::REMOVE_OBJ, S(0)), None),
        (
            "put_prop",
            op::var(op::PUT_PROP, &[S(0), S(20), S(1)]),
            None,
        ),
        ("print_obj", op::op1(op::PRINT_OBJ, S(0)), None),
    ];
    for (name, instruction, stores) in cases {
        let (mut m, e) = run_code([poison(0), instruction].concat());
        assert_eq!(e, Event::Finished, "{name}: the game continues");
        let diagnostics = m.take_diagnostics();
        assert_eq!(diagnostics.len(), 1, "{name}");
        assert_eq!(diagnostics[0].kind, DiagnosticKind::ObjectZero, "{name}");
        if let Some(value) = stores {
            assert_eq!(m.global(0), value as u16, "{name} yields 0");
        }
        assert_eq!(
            m.take_output(),
            "",
            "{name}: nothing is added to the game's text"
        );
        assert!(m.take_diagnostics().is_empty(), "{name}: drained");
        // Nothing in the tree moved.
        let o = Objects::new(m.header.objects);
        assert_eq!(children(&m.memory, &o, 1), vec![2, 3, 4], "{name}");
    }
}

// The diagnostic names the instruction that caused it.
#[test]
fn the_diagnostic_carries_the_instruction_address() {
    let mut s = story();
    let at = s.next_code_address();
    s.main(
        &[
            op::op0(op::NOP),
            op::op1(op::REMOVE_OBJ, S(0)),
            op::op0(op::QUIT),
        ]
        .concat(),
    );
    let (mut m, _) = run_story(&mut s);
    assert_eq!(m.take_diagnostics()[0].address, at + 1);
}
