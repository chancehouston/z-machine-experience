//! Tests of the table readers, on story files built by the testkit. "§" cites
//! the Z-Machine Standard 1.1.

use crate::table::*;
use zmachine_core::{LoadOptions, Machine};
use zmachine_testkit::op;
use zmachine_testkit::{Object, Object5, Story, Story5};

const OPTIONS: LoadOptions = LoadOptions {
    seed: Some(1),
    entropy: 0,
};

fn machine(image: Vec<u8>) -> Machine {
    Machine::load(image, OPTIONS).expect("the test story loads")
}

/// A version 3 story with a small tree: 1 Room > (2 lamp > 4 battery, 3 box); 5 loose.
fn story3() -> Vec<u8> {
    let mut s = Story::new();
    s.abbreviation(0, "the ");
    s.global(0, 1).global(7, 0xABCD);
    s.word("lamp").word("take").word("north");
    let mut room = Object::new("Room");
    room.child = 2;
    room.attributes = 0x8000_0001; // attributes 0 and 31
    room.props = vec![(5, vec![1, 2, 3]), (20, vec![9]), (31, vec![0xAA, 0xBB])];
    let mut lamp = Object::new("brass lamp");
    lamp.parent = 1;
    lamp.sibling = 3;
    lamp.child = 4;
    let mut boxo = Object::new("");
    boxo.parent = 1;
    let mut battery = Object::new("battery");
    battery.parent = 2;
    let loose = Object::new("the the thing");
    for o in [room, lamp, boxo, battery, loose] {
        s.object(o);
    }
    s.main(&op::op0(op::QUIT));
    s.build()
}

/// The same shape in version 5 or 8.
fn story5(version: u8) -> Vec<u8> {
    let mut s = Story5::new(version);
    s.abbreviation(0, "the ");
    s.global(0, 1).global(7, 0xABCD);
    s.word("lantern").word("take").word("northeast");
    let mut room = Object5::new("Room");
    room.child = 2;
    room.attributes = 1 << 47 | 1; // attributes 0 and 47
    room.props = vec![
        (5, vec![1, 2, 3]),
        (20, vec![9]),
        (63, vec![0xAA, 0xBB]),
        (40, vec![7; 20]),
    ];
    let mut lamp = Object5::new("brass lamp");
    lamp.parent = 1;
    lamp.sibling = 3;
    lamp.child = 4;
    let mut boxo = Object5::new("");
    boxo.parent = 1;
    let mut battery = Object5::new("battery");
    battery.parent = 2;
    let loose = Object5::new("the the thing");
    for o in [room, lamp, boxo, battery, loose] {
        s.object(o);
    }
    s.main(&op::op0(op::QUIT));
    s.build()
}

fn images() -> Vec<(u8, Vec<u8>)> {
    vec![(3, story3()), (5, story5(5)), (8, story5(8))]
}

// §11.1: the header says where the tables are; §1.2.3: a packed address is the
// byte address over 2, 4 or 8.
#[test]
fn the_layout_is_read_from_the_header() {
    for (version, image) in images() {
        let m = machine(image.clone());
        let b = Bytes::now(&m.view());
        let layout = Layout::read(b);
        assert_eq!(layout.version, version);
        assert_eq!(
            layout.unpack(0x100),
            0x100 * [0, 0, 0, 2, 0, 4, 0, 0, 8][version as usize]
        );
        assert_eq!(
            layout.high_memory,
            u16::from_be_bytes([image[4], image[5]]) as usize
        );
        assert_eq!(b.len(), image.len());
        assert_eq!(b.byte(0), Some(version));
        assert_eq!(
            b.word(0x12),
            Some(u16::from_be_bytes([image[0x12], image[0x13]]))
        );
        assert_eq!(b.byte(image.len()), None);
    }
}

// §12.3: the object table has no count. The last object is the one before the
// first property table.
#[test]
fn the_last_object_is_found_from_the_property_tables() {
    for (version, image) in images() {
        let m = machine(image);
        let layout = Layout::read(Bytes::now(&m.view()));
        assert_eq!(layout.last_object, 5, "version {version}");
    }
}

// §12.3.1, §12.3.2: parent, sibling and child; attributes numbered from the top bit.
#[test]
fn an_objects_links_and_attributes() {
    for (version, image) in images() {
        let m = machine(image);
        let b = Bytes::now(&m.view());
        let layout = Layout::read(b);
        let room = layout.object(b, 1).unwrap();
        assert_eq!(
            (room.parent, room.sibling, room.child),
            (0, 0, 2),
            "version {version}"
        );
        let lamp = layout.object(b, 2).unwrap();
        assert_eq!((lamp.parent, lamp.sibling, lamp.child), (1, 3, 4));
        let top = if version == 3 { 31 } else { 47 };
        assert_eq!(layout.attributes(b, 1), Some(vec![0, top]));
        assert_eq!(layout.attributes(b, 2), Some(vec![]));
        assert_eq!(layout.has_attribute(b, 1, top), Some(true));
        assert_eq!(layout.has_attribute(b, 1, 1), Some(false));
        assert_eq!(
            layout.has_attribute(b, 1, top + 1),
            None,
            "no such attribute"
        );
    }
}

// §12.4: the property table starts with the short name; §3.3: abbreviations.
#[test]
fn an_objects_name() {
    for (version, image) in images() {
        let m = machine(image);
        let b = Bytes::now(&m.view());
        let layout = Layout::read(b);
        assert_eq!(
            layout.name(b, 1).as_deref(),
            Some("Room"),
            "version {version}"
        );
        assert_eq!(layout.name(b, 2).as_deref(), Some("brass lamp"));
        assert_eq!(
            layout.name(b, 3).as_deref(),
            Some(""),
            "an object with no name"
        );
        assert_eq!(layout.name(b, 5).as_deref(), Some("the the thing"));
    }
}

// §12.4.1, §12.4.2: properties in table order (descending number), in both size
// formats, including a version 5 property longer than 8 bytes.
#[test]
fn an_objects_properties() {
    for (version, image) in images() {
        let m = machine(image);
        let b = Bytes::now(&m.view());
        let layout = Layout::read(b);
        let props = layout.properties(b, 1).unwrap();
        let shape: Vec<(u8, usize)> = props.iter().map(|p| (p.number, p.len)).collect();
        if version == 3 {
            assert_eq!(shape, [(31, 2), (20, 1), (5, 3)]);
        } else {
            assert_eq!(shape, [(63, 2), (40, 20), (20, 1), (5, 3)]);
        }
        let five = layout.property(b, 1, 5).unwrap().unwrap();
        let data: Vec<u8> = (0..five.len)
            .map(|i| b.byte(five.at + i).unwrap())
            .collect();
        assert_eq!(data, [1, 2, 3]);
        let first = props[0];
        assert_eq!(b.word(first.at), Some(0xAABB));
        assert_eq!(layout.property(b, 1, 6), Some(None), "not provided");
        assert_eq!(layout.properties(b, 3).map(|p| p.len()), Some(0));
    }
}

// §6.2: globals, now and as shipped.
#[test]
fn globals_now_and_as_shipped() {
    for (_, image) in images() {
        let m = machine(image);
        let view = m.view();
        let layout = Layout::read(Bytes::now(&view));
        assert_eq!(layout.global(Bytes::now(&view), 7), Some(0xABCD));
        assert_eq!(layout.global(Bytes::shipped(&view), 7), Some(0xABCD));
        assert_eq!(layout.global(Bytes::now(&view), 0), Some(1));
        assert_eq!(layout.global(Bytes::now(&view), 239), Some(0));
        assert_eq!(
            layout.global(Bytes::now(&view), 240),
            None,
            "there are 240 globals"
        );
    }
}

// §13.2: the dictionary's header; §13.3, §13.4: 4 bytes of text in version 3
// (6 characters), 6 bytes from version 4 (9 characters).
#[test]
fn the_dictionary_and_its_words() {
    for (version, image) in images() {
        let m = machine(image);
        let b = Bytes::now(&m.view());
        let layout = Layout::read(b);
        let d = layout.dictionary(b).unwrap();
        assert_eq!(d.count, 3, "version {version}");
        assert_eq!(
            (d.entry_len, d.text_len),
            if version == 3 { (7, 4) } else { (9, 6) }
        );
        let words: Vec<String> = (0..d.count)
            .map(|i| layout.word(b, d.first + i * d.entry_len).unwrap())
            .collect();
        if version == 3 {
            assert_eq!(words, ["lamp", "north", "take"]);
        } else {
            assert_eq!(words, ["lantern", "northeast", "take"]);
        }
    }
}

// §3: text. Capitals and punctuation (alphabets A1 and A2), a new-line, and a
// character written as a ten-bit ZSCII code.
#[test]
fn text_is_decoded() {
    let mut s = Story5::new(5);
    let plain = s.string("Hello, World! 42\nnext @ line");
    s.main(&op::op0(op::QUIT));
    let image = s.build();
    let m = machine(image);
    let b = Bytes::now(&m.view());
    let layout = Layout::read(b);
    assert_eq!(
        layout.text(b, layout.unpack(plain)).as_deref(),
        Some("Hello, World! 42\nnext @ line")
    );
}

// §3.5.5, §3.8.5: a story file may supply its own alphabet table and Unicode
// table; text is decoded with them.
#[test]
fn a_custom_alphabet_and_unicode_table_are_used() {
    let mut s = Story5::new(5);
    // An alphabet table that swaps a and b in A0 and is otherwise the default.
    let mut table = Vec::new();
    table.extend(b"bacdefghijklmnopqrstuvwxyz");
    table.extend(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    table.extend(b" \n0123456789.,!?_#'\"/\\-:()");
    let alphabet = s.scratch(&table);
    s.header_word(0x34, alphabet);
    // A Unicode table whose first entry, ZSCII 155, is a Greek alpha.
    let unicode = s.scratch(&[1, 0x03, 0xB1]);
    let extension = s.scratch(&[0, 3, 0, 0, 0, 0, (unicode >> 8) as u8, unicode as u8]);
    s.header_word(0x36, extension);
    // ZSCII 155, written as a ten-bit code.
    let escape = s.raw_string(&{
        // z-chars 5 6 then the ten bits of 155 (4, 27), padded with 5.
        let z = [5u16, 6, 4, 27, 5, 5];
        let w1 = z[0] << 10 | z[1] << 5 | z[2];
        let w2 = 0x8000 | z[3] << 10 | z[4] << 5 | z[5];
        [w1.to_be_bytes(), w2.to_be_bytes()].concat()
    });
    s.main(&op::op0(op::QUIT));
    let image = s.build();
    let m = machine(image);
    let b = Bytes::now(&m.view());
    let layout = Layout::read(b);
    assert_eq!(
        layout.text(b, layout.unpack(escape)).as_deref(),
        Some("\u{3b1}")
    );
    // Z-characters 6 and 7 are 'a' and 'b' by the default table, 'b' and 'a' by this one.
    let mut s2 = Story5::new(5);
    let alphabet = s2.scratch(&table);
    s2.header_word(0x34, alphabet);
    let ab = s2.raw_string(&(0x8000u16 | 6 << 10 | 7 << 5 | 5).to_be_bytes());
    s2.main(&op::op0(op::QUIT));
    let m = machine(s2.build());
    let b = Bytes::now(&m.view());
    let layout = Layout::read(b);
    assert_eq!(layout.text(b, layout.unpack(ab)).as_deref(), Some("ba"));
}

// "Every reader returns `None`, never panics": addresses outside the file, object
// numbers out of range, a property table that runs off the end, text with no end.
#[test]
fn nothing_outside_the_file_is_read() {
    for (version, image) in images() {
        let m = machine(image.clone());
        let b = Bytes::now(&m.view());
        let layout = Layout::read(b);
        for number in [0u16, 6, 255, 256, 0xFFFF] {
            assert_eq!(
                layout.object(b, number),
                None,
                "version {version}, object {number}"
            );
            assert_eq!(layout.name(b, number), None);
            assert_eq!(layout.attributes(b, number), None);
            assert_eq!(layout.properties(b, number), None);
            assert_eq!(layout.property(b, number, 1), None);
        }
        assert_eq!(layout.text(b, image.len()), None);
        assert_eq!(layout.text(b, image.len() - 1), None);
        assert_eq!(layout.text(b, usize::MAX), None);
        assert_eq!(layout.word(b, image.len() - 2), None);
    }
}

// A story whose tables are nonsense is still read without a panic.
#[test]
fn damaged_tables_do_not_panic() {
    for (_, image) in images() {
        for (at, value) in [
            (0x0A, 0xFFu8),
            (0x0B, 0xFE),
            (0x08, 0xFF),
            (0x0C, 0xFF),
            (0x18, 0xFF),
        ] {
            let mut damaged = image.clone();
            damaged[at] = value;
            // The core may refuse the file; if it loads, every reader must cope.
            let Ok(m) = Machine::load(damaged, OPTIONS) else {
                continue;
            };
            let b = Bytes::now(&m.view());
            let layout = Layout::read(b);
            for number in 0..=layout.last_object.min(300) {
                let _ = layout.object(b, number);
                let _ = layout.name(b, number);
                let _ = layout.attributes(b, number);
                let _ = layout.properties(b, number);
            }
            if let Some(d) = layout.dictionary(b) {
                for i in 0..d.count.min(500) {
                    let _ = layout.word(b, d.first + i * d.entry_len);
                }
            }
            for i in 0..240 {
                let _ = layout.global(b, i);
            }
        }
    }
}
