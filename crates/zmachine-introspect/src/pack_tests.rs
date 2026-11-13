//! Game packs on built story files (docs/introspection.md section 12, cited
//! I§12.n; specs/006-game-packs/contracts/pack-format.md, statements P§1 to 26).

use crate::fixtures::{Inform, Zil, clear_attr, set_attr, set_global, zl};
use crate::pack::Pack;
use crate::{DoorState, Exit, ExitKind, Introspector, PackState, Source, Unknown};
use sha2::{Digest, Sha256};
use zmachine_core::Machine;

/// The attribute the fixture's pack calls "open".
const OPEN: u8 = 7;
/// The variable number of the fixture's flag exit, as its table stores it.
const FLAG: u8 = 16 + zl::TRAP_OPEN;

/// The identity lines of a pack for this image, as shipped.
fn identity(image: &[u8]) -> String {
    let digest: String = Sha256::digest(image)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let release = u16::from_be_bytes([image[2], image[3]]);
    let serial = String::from_utf8(image[0x12..0x18].to_vec()).unwrap();
    format!("pack: 1\nsha256: {digest}\nrelease: {release}\nserial: {serial}\n")
}

/// A fixture whose header has a release and a serial worth matching.
fn fixture(setup: Vec<Vec<u8>>) -> (Vec<u8>, Machine) {
    let mut f = Zil::new();
    f.setup.extend(setup.concat());
    let mut image = f.build();
    image[2..4].copy_from_slice(&[0, 77]);
    image[0x12..0x18].copy_from_slice(b"861231");
    let machine = crate::fixtures::run(image.clone());
    (image, machine)
}

/// The exits of a built story with a pack for it that holds these entries. Each
/// build is a story file of its own, with its own digest, so each has its own
/// pack.
fn exits(built: &(Vec<u8>, Machine), entries: &str) -> crate::Exits {
    let pack = format!("{}{entries}", identity(&built.0));
    let view = built.1.view();
    let i = Introspector::with_pack(&view, &pack);
    assert_eq!(i.pack(), PackState::InForce, "{pack}");
    i.exits(&view).value.unwrap()
}

fn toward(exits: &crate::Exits, direction: u8) -> Option<&Exit> {
    exits.exits.iter().find(|e| e.direction == direction)
}

const EXAMPLE: &str = "pack: 1\nsha256: 0000000000000000000000000000000000000000000000000000000000000000\nrelease: 1\nserial: 000000\nopen-attribute: 7\nuntrusted-variable: 40\n";

// I§12.1 (P§1 to 5): the worked example is read, with comments, blank lines and
// a carriage return before the line feed.
#[test]
fn the_worked_example_is_read() {
    let pack = Pack::parse(EXAMPLE).unwrap();
    assert_eq!(pack.sha256, "0".repeat(64));
    assert_eq!((pack.release, &pack.serial), (1, b"000000"));
    assert_eq!(pack.open_attribute, Some(7));
    assert_eq!(pack.untrusted_variables, [40]);
    assert!(pack.has_entries());

    let dressed = format!(
        "# A pack.\r\n\n   \n{}# the end\n",
        EXAMPLE.replace('\n', "\r\n")
    );
    assert_eq!(Pack::parse(&dressed).unwrap(), pack);

    let bare = EXAMPLE.lines().take(4).collect::<Vec<_>>().join("\n");
    let bare = Pack::parse(&bare).unwrap();
    assert!(!bare.has_entries(), "a pack may hold no entry");

    let many = format!("{EXAMPLE}untrusted-variable: 16\nuntrusted-variable: 255\n");
    assert_eq!(
        Pack::parse(&many).unwrap().untrusted_variables,
        [40, 16, 255]
    );
    // Entries in any order.
    let turned = EXAMPLE.replace(
        "open-attribute: 7\nuntrusted-variable: 40\n",
        "untrusted-variable: 40\nopen-attribute: 0\n",
    );
    assert_eq!(Pack::parse(&turned).unwrap().open_attribute, Some(0));
}

// I§12.1 (P§6): anything else refuses the whole pack, and says which line.
#[test]
fn a_malformed_pack_is_refused_with_its_line() {
    let digest = "0".repeat(64);
    let head = format!("pack: 1\nsha256: {digest}\nrelease: 1\nserial: 000000\n");
    let cases: Vec<(String, usize)> = vec![
        // An identity line missing, out of order, or after an entry.
        (format!("pack: 1\nsha256: {digest}\nrelease: 1\n"), 4),
        (
            format!("sha256: {digest}\npack: 1\nrelease: 1\nserial: 000000\n"),
            1,
        ),
        (
            format!("pack: 1\nrelease: 1\nsha256: {digest}\nserial: 000000\n"),
            2,
        ),
        (format!("open-attribute: 7\n{head}"), 1),
        (String::new(), 1),
        // A version of the format this reader does not know.
        (head.replace("pack: 1", "pack: 4"), 1),
        (head.replace("pack: 1", "pack: 01"), 1),
        // Values out of range or of the wrong shape.
        (head.replace(&digest, &"0".repeat(63)), 2),
        (head.replace(&digest, &format!("{}A", "0".repeat(63))), 2),
        (head.replace("release: 1", "release: 65536"), 3),
        (head.replace("release: 1", "release: -1"), 3),
        (head.replace("release: 1", "release: +1"), 3),
        (head.replace("release: 1", "release: 01"), 3),
        (head.replace("release: 1", "release: "), 3),
        (head.replace("serial: 000000", "serial: 00000"), 4),
        (head.replace("serial: 000000", "serial: 00 000"), 4),
        (head.replace("serial: 000000", "serial: 0000000"), 4),
        (format!("{head}open-attribute: 32\n"), 5),
        (format!("{head}open-attribute: 7\nopen-attribute: 8\n"), 6),
        (format!("{head}untrusted-variable: 15\n"), 5),
        (format!("{head}untrusted-variable: 256\n"), 5),
        (
            format!("{head}untrusted-variable: 40\n\n# again\nuntrusted-variable: 40\n"),
            8,
        ),
        // Lines that are not a key, a colon, a space and a value.
        (format!("{head}open-attribute: 7 # the door\n"), 5),
        (format!("{head}open-attribute:7\n"), 5),
        (format!("{head}open-attribute 7\n"), 5),
        (format!("{head}open-attribute:  7\n"), 5),
        (format!("{head} open-attribute: 7\n"), 5),
        // A key this format does not have.
        (format!("{head}lit-attribute: 3\n"), 5),
        (format!("{head}pack: 1\n"), 5),
        (format!("{head}name: zork\n"), 5),
    ];
    for (text, line) in cases {
        match Pack::parse(&text) {
            Ok(pack) => panic!("read as {pack:?}:\n{text}"),
            Err(error) => assert_eq!(error.line, line, "{error}:\n{text}"),
        }
    }
}

// I§12.2 (P§7 to 10): a pack is in force exactly when digest, release and serial
// are all the story file's; the reader says what became of it.
#[test]
fn a_pack_is_in_force_only_for_its_own_story() {
    let (image, machine) = fixture(vec![]);
    let view = machine.view();
    assert_eq!(Introspector::new(&view).pack(), PackState::None);

    let right = identity(&image);
    assert!(right.contains("release: 77\nserial: 861231\n"), "{right}");
    assert_eq!(
        Introspector::with_pack(&view, &right).pack(),
        PackState::InForce
    );
    let with_entry = format!("{right}open-attribute: {OPEN}\n");
    assert_eq!(
        Introspector::with_pack(&view, &with_entry).pack(),
        PackState::InForce
    );

    // One digit of the digest, the release by one, one character of the serial.
    let digit = right.as_bytes()[16] as char;
    let other = if digit == '0' { '1' } else { '0' };
    let mut wrong_digest = right.clone();
    wrong_digest.replace_range(16..17, &other.to_string());
    for wrong in [
        wrong_digest,
        right.replace("release: 77", "release: 78"),
        right.replace("serial: 861231", "serial: 861232"),
    ] {
        assert_ne!(wrong, right);
        assert_eq!(
            Introspector::with_pack(&view, &wrong).pack(),
            PackState::NotMatching,
            "{wrong}"
        );
    }

    // I§12.1: a pack that cannot be read is refused, and nothing fails.
    let noise: String = (0..100_000u32)
        .map(|n| char::from_u32(n.wrapping_mul(2_654_435_761) % 0x250 + 1).unwrap())
        .collect();
    for bad in [
        "",
        "pack: 2\n",
        "\u{0}\u{1}",
        noise.as_str(),
        &right[..right.len() - 2],
    ] {
        assert_eq!(
            Introspector::with_pack(&view, bad).pack(),
            PackState::Refused
        );
    }

    // I§12.2 (P§12): the pack's state belongs to the story file, not to the
    // game in progress: it is the same however far the game has run.
    let (image, later) = fixture(vec![set_attr(zl::DOOR, OPEN), set_global(zl::TRAP_OPEN, 1)]);
    let its_own = format!("{}open-attribute: {OPEN}\n", identity(&image));
    assert_eq!(
        Introspector::with_pack(&later.view(), &its_own).pack(),
        PackState::InForce
    );
    // And the first story's pack is not this story's: its start-up code, and so
    // its digest, differs.
    assert_eq!(
        Introspector::with_pack(&later.view(), &with_entry).pack(),
        PackState::NotMatching
    );
}

// I§12.2 (P§11): a pack that holds an entry is refused for a story file that is
// not of the ZIL family; one that holds none is merely in force.
#[test]
fn an_entry_is_refused_outside_the_zil_family() {
    let image = Inform::new().build();
    let machine = crate::fixtures::run(image.clone());
    let view = machine.view();
    let right = identity(&image);
    assert_eq!(
        Introspector::with_pack(&view, &right).pack(),
        PackState::InForce
    );
    for entry in ["open-attribute: 7\n", "untrusted-variable: 40\n"] {
        assert_eq!(
            Introspector::with_pack(&view, &format!("{right}{entry}")).pack(),
            PackState::Refused
        );
    }
}

// I§12.3 (P§13 to 16), I§12.5 (P§19 to 21): with `open-attribute` in force a
// 5-byte exit is a door, open or closed, with the room behind it, and marked.
#[test]
fn a_door_is_open_or_closed() {
    let pack = format!("open-attribute: {OPEN}\n");
    let answer = exits(&fixture(vec![]), &pack);
    let west = toward(&answer, zl::WEST).expect("a closed door is still an exit");
    assert_eq!(west.kind, ExitKind::Door);
    assert_eq!(west.state, Some(DoorState::Closed));
    assert_eq!(west.destination, Some(zl::HALL));
    assert_eq!(west.destination_name, Some(Ok("Hall".to_string())));
    assert_eq!(west.door, Some(zl::DOOR));
    assert_eq!(west.source, Source::Pack);

    let answer = exits(&fixture(vec![set_attr(zl::DOOR, OPEN)]), &pack);
    let west = toward(&answer, zl::WEST).unwrap();
    assert_eq!(
        (
            west.kind,
            west.state,
            west.destination,
            west.door,
            west.source
        ),
        (
            ExitKind::Door,
            Some(DoorState::Open),
            Some(zl::HALL),
            Some(zl::DOOR),
            Source::Pack
        )
    );

    // Shut again, it is closed again: the answer is the state at the query.
    let shut = fixture(vec![set_attr(zl::DOOR, OPEN), clear_attr(zl::DOOR, OPEN)]);
    assert_eq!(
        toward(&exits(&shut, &pack), zl::WEST).unwrap().state,
        Some(DoorState::Closed)
    );

    // Another attribute of the door is not the one the pack names.
    let other = fixture(vec![set_attr(zl::DOOR, OPEN + 1)]);
    assert_eq!(
        toward(&exits(&other, &pack), zl::WEST).unwrap().state,
        Some(DoorState::Closed)
    );

    // I§12.5: every other exit is the file's own, with no door.
    for exit in answer.exits.iter().filter(|e| e.direction != zl::WEST) {
        assert_eq!(
            (exit.door, exit.state, exit.source),
            (None, None, Source::File),
            "{exit:?}"
        );
    }
    assert_eq!(toward(&answer, zl::NORTH).unwrap().kind, ExitKind::Fixed);
    assert_eq!(
        toward(&answer, zl::EAST).unwrap().kind,
        ExitKind::Conditional
    );
}

// I§12.3 (P§14): a 5-byte exit whose room is not a room, or whose door is not an
// object, is unknown; I§12.5 (P§23): and an unknown direction carries no mark.
#[test]
fn a_door_exit_that_cannot_be_read_is_unknown() {
    let build = |room: u8, door: u8| {
        let mut f = Zil::new();
        f.prop(zl::KITCHEN, zl::WEST, vec![room, door, 0x12, 0x34, 0]);
        let image = f.build();
        (image.clone(), crate::fixtures::run(image))
    };
    let objects = Zil::new().objects.len() as u8;
    for (room, door) in [
        (zl::LAMP as u8, zl::DOOR as u8),
        (0, zl::DOOR as u8),
        (zl::HALL as u8, 0),
        (zl::HALL as u8, objects + 1),
        (zl::HALL as u8, 255),
    ] {
        let answer = exits(&build(room, door), &format!("open-attribute: {OPEN}\n"));
        assert!(toward(&answer, zl::WEST).is_none(), "{room} {door}");
        let unknown: Vec<(u8, Unknown)> = answer
            .unknown
            .iter()
            .map(|u| (u.direction, u.reason))
            .collect();
        assert_eq!(
            unknown,
            [(zl::WEST, Unknown::NotDeterminable)],
            "{room} {door}"
        );
    }
}

// I§12.4 (P§17, 18), I§12.5 (P§20): a 4-byte exit on an untrusted variable is
// conditional whatever the variable holds, and marked; on any other variable it
// is read as without a pack.
#[test]
fn an_untrusted_variable_makes_its_exit_conditional() {
    let unset = fixture(vec![]);
    let set = fixture(vec![set_global(zl::TRAP_OPEN, 1)]);
    let named = format!("untrusted-variable: {FLAG}\n");
    for machine in [&unset, &set] {
        let answer = exits(machine, &named);
        let down = toward(&answer, zl::DOWN).expect("conditional, whatever it holds");
        assert_eq!(
            (
                down.kind,
                down.destination,
                down.door,
                down.state,
                down.source
            ),
            (ExitKind::Conditional, None, None, None, Source::Pack)
        );
        // A door is as it is without an `open-attribute`: conditional, the file's.
        let west = toward(&answer, zl::WEST).unwrap();
        assert_eq!(
            (west.kind, west.source),
            (ExitKind::Conditional, Source::File)
        );
    }

    let another = format!("untrusted-variable: {}\n", FLAG + 1);
    assert!(
        toward(&exits(&unset, &another), zl::DOWN).is_none(),
        "the flag is not set"
    );
    let down = exits(&set, &another);
    let down = toward(&down, zl::DOWN).unwrap();
    assert_eq!(
        (down.kind, down.destination, down.source),
        (ExitKind::Fixed, Some(zl::CELLAR), Source::File)
    );

    // Both kinds of entry in one pack, each applied.
    let both = format!("{named}open-attribute: {OPEN}\n");
    let answer = exits(&set, &both);
    assert_eq!(toward(&answer, zl::DOWN).unwrap().source, Source::Pack);
    assert_eq!(toward(&answer, zl::WEST).unwrap().kind, ExitKind::Door);
}

// I§12.2 (P§10), I§12.5 (P§20): a pack that is not in force changes nothing,
// and with none every exit is the file's.
#[test]
fn a_pack_not_in_force_changes_nothing() {
    for setup in [
        vec![],
        vec![set_attr(zl::DOOR, OPEN), set_global(zl::TRAP_OPEN, 1)],
    ] {
        let (image, machine) = fixture(setup);
        let view = machine.view();
        let plain = Introspector::new(&view).exits(&view);
        for exit in &plain.value.as_ref().unwrap().exits {
            assert_eq!(
                (exit.door, exit.state, exit.source),
                (None, None, Source::File)
            );
        }
        let right = format!(
            "{}open-attribute: {OPEN}\nuntrusted-variable: {FLAG}\n",
            identity(&image)
        );
        assert_ne!(Introspector::with_pack(&view, &right).exits(&view), plain);
        for (text, state) in [
            (
                right.replace("release: 77", "release: 76"),
                PackState::NotMatching,
            ),
            (format!("{right}open-attribute: 9\n"), PackState::Refused),
            (right.replace("pack: 1", "pack: 4"), PackState::Refused),
        ] {
            let i = Introspector::with_pack(&view, &text);
            assert_eq!(i.pack(), state);
            assert_eq!(i.exits(&view), plain, "{text}");
        }
    }
}

// I§12.6 (P§24, 25): a pack changes no query but `exits`, and an answer with a
// pack in force is still a read: asked twice it is the same, and the machine's
// memory is what it was.
#[test]
fn a_pack_changes_only_exits_and_reads_only() {
    let (image, machine) = fixture(vec![set_attr(zl::DOOR, OPEN)]);
    let view = machine.view();
    let pack = format!(
        "{}open-attribute: {OPEN}\nuntrusted-variable: {FLAG}\n",
        identity(&image)
    );
    let (plain, packed) = (
        Introspector::new(&view),
        Introspector::with_pack(&view, &pack),
    );
    let before: Vec<Option<u8>> = (0..view.len()).map(|a| view.byte(a)).collect();
    assert_eq!(packed.location(&view), plain.location(&view));
    assert_eq!(packed.inventory(&view), plain.inventory(&view));
    assert_eq!(packed.scope(&view), plain.scope(&view));
    assert_eq!(packed.object_tree(&view), plain.object_tree(&view));
    assert_eq!(packed.dictionary(&view), plain.dictionary(&view));
    assert_eq!(packed.family(), plain.family());
    assert_eq!(packed.exits(&view), packed.exits(&view));
    let after: Vec<Option<u8>> = (0..view.len()).map(|a| view.byte(a)).collect();
    assert_eq!(before, after);
}

// ---------------------------------------------------------------------------
// The inventory (feature 007; specs/007-inventory-panel/contracts/answers.md)
// ---------------------------------------------------------------------------

/// The attribute the fixture's pack calls "see-through".
const CLEAR: u8 = 8;

/// What the player carries in a built story, with a pack of these entries.
fn carried(built: &(Vec<u8>, Machine), version: u8, entries: &str) -> Vec<crate::Carried> {
    let pack = format!("{}{entries}", identity(&built.0))
        .replace("pack: 1\n", &format!("pack: {version}\n"));
    let view = built.1.view();
    let i = Introspector::with_pack(&view, &pack);
    assert_eq!(i.pack(), PackState::InForce, "{pack}");
    i.inventory(&view).value.unwrap()
}

/// The sack, carried, with the garlic in it and a diamond in the garlic.
fn sack(mut setup: Vec<Vec<u8>>) -> (Vec<u8>, Machine) {
    setup.insert(0, crate::fixtures::insert(zl::SACK, zl::PLAYER));
    setup.insert(1, crate::fixtures::insert(zl::DIAMOND, zl::GARLIC));
    fixture(setup)
}

/// The marks of the sack, the garlic in it and the diamond in the garlic.
type Marks = (Option<crate::ContainerState>, crate::Tri, Source);
fn marks(carried: &[crate::Carried]) -> [Marks; 3] {
    let sack = &carried[0];
    assert_eq!(sack.object, zl::SACK);
    let garlic = &sack.contents[0];
    let diamond = &garlic.contents[0];
    [sack, garlic, diamond].map(|e| (e.state, e.seen, e.source))
}

// I§7.2, I§12.3 (A§10, 15): with `open-attribute` in force a carried thing that
// holds something is open or closed, what an open one holds is seen, and what a
// closed one holds is not. I§7 (A§3): a thing in a thing is seen only if every
// thing it is in lets it be. A§5: what a pack changed says so.
#[test]
fn a_pack_says_whether_a_carried_container_is_open() {
    use crate::ContainerState::{Closed, Open};
    use crate::Tri::{No, Yes};
    let open = format!("open-attribute: {OPEN}\n");
    let pack = Source::Pack;
    // Both closed: the garlic is not seen, nor the diamond in it.
    assert_eq!(
        marks(&carried(&sack(vec![]), 1, &open)),
        [
            (Some(Closed), Yes, pack),
            (Some(Closed), No, pack),
            (None, No, pack)
        ]
    );
    // The sack open, the garlic closed.
    assert_eq!(
        marks(&carried(&sack(vec![set_attr(zl::SACK, OPEN)]), 1, &open)),
        [
            (Some(Open), Yes, pack),
            (Some(Closed), Yes, pack),
            (None, No, pack)
        ]
    );
    // The garlic open in a closed sack: nothing inside the sack is seen.
    assert_eq!(
        marks(&carried(&sack(vec![set_attr(zl::GARLIC, OPEN)]), 1, &open)),
        [
            (Some(Closed), Yes, pack),
            (Some(Open), No, pack),
            (None, No, pack)
        ]
    );
    // Both open.
    let both = sack(vec![set_attr(zl::SACK, OPEN), set_attr(zl::GARLIC, OPEN)]);
    assert_eq!(
        marks(&carried(&both, 1, &open)),
        [
            (Some(Open), Yes, pack),
            (Some(Open), Yes, pack),
            (None, Yes, pack)
        ]
    );
    // The lamp holds nothing and is at the top: nothing about it rests on the pack.
    let lamp = carried(&both, 1, &open)[1].clone();
    assert_eq!(
        (lamp.object, lamp.state, lamp.seen, lamp.source),
        (zl::LAMP, None, Yes, Source::File)
    );
    // The exits are as they were with this entry (I§12.3).
    assert_eq!(
        toward(&exits(&both, &open), zl::WEST).unwrap().kind,
        ExitKind::Door
    );
}

// I§7.2, I§12.8 (A§11, 12): with `transparent-attribute` in force, what a thing
// with that attribute holds is seen, whatever its state.
#[test]
fn a_pack_says_which_containers_are_see_through() {
    use crate::ContainerState::{Closed, Unknown};
    use crate::Tri::{No, Yes};
    let pack = Source::Pack;
    let both = format!("open-attribute: {OPEN}\ntransparent-attribute: {CLEAR}\n");
    // A closed, see-through sack: the garlic is seen; the garlic is closed and
    // not see-through, so the diamond is not.
    assert_eq!(
        marks(&carried(&sack(vec![set_attr(zl::SACK, CLEAR)]), 2, &both)),
        [
            (Some(Closed), Yes, pack),
            (Some(Closed), Yes, pack),
            (None, No, pack)
        ]
    );
    // Without the attribute it is as with `open-attribute` alone.
    assert_eq!(
        marks(&carried(&sack(vec![]), 2, &both))[1],
        (Some(Closed), No, pack)
    );
    // With only `transparent-attribute`: a sack that has it shows what it holds
    // and its state stays unknown; the garlic, which has it not, is as with no
    // pack, so the diamond is unknown.
    let only = format!("transparent-attribute: {CLEAR}\n");
    assert_eq!(
        marks(&carried(&sack(vec![set_attr(zl::SACK, CLEAR)]), 2, &only)),
        [
            (Some(Unknown), Yes, Source::File),
            (Some(Unknown), Yes, pack),
            (None, crate::Tri::Unknown, Source::File)
        ]
    );
    assert_eq!(
        marks(&carried(&sack(vec![]), 2, &only)),
        [
            (Some(Unknown), Yes, Source::File),
            (Some(Unknown), crate::Tri::Unknown, Source::File),
            (None, crate::Tri::Unknown, Source::File)
        ]
    );
    // It changes nothing about exits (I§12.8).
    let built = sack(vec![]);
    let view = built.1.view();
    let with = Introspector::with_pack(
        &view,
        &format!("{}{only}", identity(&built.0)).replace("pack: 1", "pack: 2"),
    );
    assert_eq!(with.pack(), PackState::InForce);
    assert_eq!(with.exits(&view), Introspector::new(&view).exits(&view));
}

// I§12.1, I§12.2 (A§12 to 14): the format's version 2 adds one key. A reader of
// version 2 reads versions 1 and 2, and refuses any other; the new key in a pack
// of version 1 is a fault.
#[test]
fn the_formats_second_version_adds_one_key() {
    let two = EXAMPLE.replace("pack: 1", "pack: 2");
    let pack = Pack::parse(&format!("{two}transparent-attribute: 12\n")).unwrap();
    assert_eq!((pack.version, pack.transparent_attribute), (2, Some(12)));
    assert!(pack.has_entries());
    let plain = Pack::parse(&two).unwrap();
    assert_eq!((plain.version, plain.transparent_attribute), (2, None));
    assert_eq!(Pack::parse(EXAMPLE).unwrap().version, 1);
    for (text, line) in [
        (EXAMPLE.replace("pack: 1", "pack: 4"), 1),
        (EXAMPLE.replace("pack: 1", "pack: 0"), 1),
        (format!("{EXAMPLE}transparent-attribute: 12\n"), 7),
        (format!("{two}transparent-attribute: 32\n"), 7),
        (
            format!("{two}transparent-attribute: 12\ntransparent-attribute: 12\n"),
            8,
        ),
        (format!("{two}transparent-attribute: glass\n"), 7),
    ] {
        match Pack::parse(&text) {
            Ok(pack) => panic!("read as {pack:?}:\n{text}"),
            Err(error) => assert_eq!(error.line, line, "{error}:\n{text}"),
        }
    }
    // It is an entry, and so refused outside the ZIL family (I§12.2).
    let image = Inform::new().build();
    let machine = crate::fixtures::run(image.clone());
    let text =
        format!("{}transparent-attribute: 3\n", identity(&image)).replace("pack: 1", "pack: 2");
    assert_eq!(
        Introspector::with_pack(&machine.view(), &text).pack(),
        PackState::Refused
    );
}

// ---------------------------------------------------------------------------
// Light (feature 008; specs/008-darkness/contracts/answers.md)
// ---------------------------------------------------------------------------

/// The attribute the fixture's pack calls "lit".
const LIT: u8 = 5;

/// The lit, open and see-through entries, as a pack of version 3 gives them.
fn lit_entries() -> String {
    format!("open-attribute: {OPEN}\ntransparent-attribute: {CLEAR}\nlit-attribute: {LIT}\n")
}

/// The interface for a built story with a version 3 pack of these entries.
fn with_light(built: &(Vec<u8>, Machine), entries: &str) -> Introspector {
    let pack = format!("{}{entries}", identity(&built.0)).replace("pack: 1\n", "pack: 3\n");
    let i = Introspector::with_pack(&built.1.view(), &pack);
    assert_eq!(i.pack(), PackState::InForce, "{pack}");
    i
}

/// `lit` of the location and `light` of the scope, with their sources.
fn light(built: &(Vec<u8>, Machine), entries: &str) -> (crate::Tri, Source, crate::Light, Source) {
    let i = with_light(built, entries);
    let view = built.1.view();
    let location = i.location(&view).value.unwrap();
    let scope = i.scope(&view).value.unwrap();
    (location.lit, location.source, scope.light, scope.source)
}

// I§12.1 (A§1, 2): the format's version 3 adds `lit-attribute`.
#[test]
fn the_formats_third_version_adds_the_lit_attribute() {
    let three = EXAMPLE.replace("pack: 1", "pack: 3");
    let pack = Pack::parse(&format!(
        "{three}lit-attribute: 19\ntransparent-attribute: 12\n"
    ))
    .unwrap();
    assert_eq!(
        (pack.version, pack.lit_attribute, pack.transparent_attribute),
        (3, Some(19), Some(12))
    );
    assert!(
        Pack::parse(&format!("{three}lit-attribute: 0\n"))
            .unwrap()
            .has_entries()
    );
    assert_eq!(Pack::parse(EXAMPLE).unwrap().lit_attribute, None);
    let two = EXAMPLE.replace("pack: 1", "pack: 2");
    for (text, line) in [
        (EXAMPLE.replace("pack: 1", "pack: 4"), 1),
        (format!("{two}lit-attribute: 19\n"), 7),
        (format!("{EXAMPLE}lit-attribute: 19\n"), 7),
        (format!("{three}lit-attribute: 32\n"), 7),
        (format!("{three}lit-attribute: 19\nlit-attribute: 19\n"), 8),
    ] {
        match Pack::parse(&text) {
            Ok(pack) => panic!("read as {pack:?}:\n{text}"),
            Err(error) => assert_eq!(error.line, line, "{error}:\n{text}"),
        }
    }
    let image = Inform::new().build();
    let machine = crate::fixtures::run(image.clone());
    let text = format!("{}lit-attribute: 3\n", identity(&image)).replace("pack: 1", "pack: 3");
    assert_eq!(
        Introspector::with_pack(&machine.view(), &text).pack(),
        PackState::Refused
    );
}

// I§12.9, I§6.2, I§9.2 (A§4, 5, 8, 9): with `lit-attribute` in force the room is
// lit if it has the attribute or a light can be seen in it, and dark otherwise;
// `scope` says the same; and both are marked.
#[test]
fn light_is_the_rooms_or_a_light_that_can_be_seen() {
    use crate::Light::{Dark, Lit};
    use crate::Tri::{No, Yes};
    use crate::fixtures::insert;
    let entries = lit_entries();
    let pack = Source::Pack;
    let lit = (Yes, pack, Lit, pack);
    let dark = (No, pack, Dark, pack);
    // The fixture's player is in the kitchen, carrying the lamp.
    assert_eq!(
        light(&fixture(vec![set_attr(zl::KITCHEN, LIT)]), &entries),
        lit
    );
    assert_eq!(light(&fixture(vec![]), &entries), dark);
    // A light carried; and lying in the room.
    assert_eq!(
        light(&fixture(vec![set_attr(zl::LAMP, LIT)]), &entries),
        lit
    );
    assert_eq!(
        light(
            &fixture(vec![set_attr(zl::LAMP, LIT), insert(zl::LAMP, zl::KITCHEN)]),
            &entries
        ),
        lit
    );
    // Shut in the closed sack; in the open sack; in a closed see-through sack.
    let in_sack = |more: Vec<Vec<u8>>| {
        let mut setup = vec![set_attr(zl::LAMP, LIT), insert(zl::LAMP, zl::SACK)];
        setup.extend(more);
        light(&fixture(setup), &entries)
    };
    assert_eq!(in_sack(vec![]), dark);
    assert_eq!(in_sack(vec![set_attr(zl::SACK, OPEN)]), lit);
    assert_eq!(in_sack(vec![set_attr(zl::SACK, CLEAR)]), lit);
    // The sack carried, open, with the light in it.
    assert_eq!(
        in_sack(vec![set_attr(zl::SACK, OPEN), insert(zl::SACK, zl::PLAYER)]),
        lit
    );
    assert_eq!(in_sack(vec![insert(zl::SACK, zl::PLAYER)]), dark);
    // A light in another room lights nothing here; nor does another room's own.
    assert_eq!(
        light(
            &fixture(vec![set_attr(zl::LAMP, LIT), insert(zl::LAMP, zl::HALL)]),
            &entries
        ),
        dark
    );
    assert_eq!(
        light(&fixture(vec![set_attr(zl::HALL, LIT)]), &entries),
        dark
    );
    // I§9.2: the room's contents are listed in the dark as in the light.
    let built = fixture(vec![]);
    let view = built.1.view();
    let in_the_dark = with_light(&built, &entries).scope(&view).value.unwrap();
    let plain = Introspector::new(&view).scope(&view).value.unwrap();
    let objects = |s: &crate::Scope| s.objects.iter().map(|e| e.object).collect::<Vec<_>>();
    assert_eq!(objects(&in_the_dark), objects(&plain));
}

// I§12.9 (A§6, 7): where what the player carries is not known, or whether a
// thing is shut in is not known, and nothing else settles it, light is unknown.
#[test]
fn light_unknown_where_a_fact_is_missing() {
    use crate::Light;
    use crate::Tri::{Unknown as NotKnown, Yes};
    use crate::fixtures::insert;
    let entries = lit_entries();
    // Two objects tie for the player (as in zil_tests).
    let tie = |more: Vec<Vec<u8>>| {
        let mut setup = vec![
            insert(zl::DIAMOND, zl::KITCHEN),
            set_global(30, zl::DIAMOND),
        ];
        setup.extend(more);
        light(&fixture(setup), &entries)
    };
    assert_eq!(
        tie(vec![]),
        (NotKnown, Source::File, Light::Unknown, Source::File)
    );
    // The lamp the true player carries is lit: it is under an object that tied.
    assert_eq!(tie(vec![set_attr(zl::LAMP, LIT)]).0, NotKnown);
    assert_eq!(tie(vec![set_attr(zl::KITCHEN, LIT)]).0, Yes);
    assert_eq!(
        tie(vec![set_attr(zl::BOAT, LIT), insert(zl::BOAT, zl::KITCHEN)]).0,
        Yes
    );
    // No `open-attribute`: a light inside the sack may or may not be seen.
    let alone = format!("lit-attribute: {LIT}\n");
    let shut_in = fixture(vec![set_attr(zl::LAMP, LIT), insert(zl::LAMP, zl::SACK)]);
    assert_eq!(light(&shut_in, &alone).0, NotKnown);
    let loose = fixture(vec![set_attr(zl::LAMP, LIT), insert(zl::LAMP, zl::KITCHEN)]);
    assert_eq!(light(&loose, &alone).0, Yes);
    // Nothing with the attribute anywhere near: dark, without knowing "open".
    assert_eq!(light(&fixture(vec![]), &alone).0, crate::Tri::No);
}

/// `dark_ahead` and `source` of the exit in a direction.
fn ahead(built: &(Vec<u8>, Machine), entries: &str, direction: u8) -> (crate::Tri, Source) {
    let view = built.1.view();
    let exits = with_light(built, entries).exits(&view).value.unwrap();
    let exit = toward(&exits, direction).expect("an exit");
    (exit.dark_ahead, exit.source)
}

// I§12.10, I§8 (A§10 to 13, 15): whether the room a way leads to would be dark
// for the player as they are, read from stored facts.
#[test]
fn dark_ahead_is_the_light_of_the_room_a_way_leads_to() {
    use crate::Tri::{No, Unknown as NotKnown, Yes};
    use crate::fixtures::insert;
    let entries = lit_entries();
    let pack = Source::Pack;
    // North from the kitchen is a fixed exit to the hall.
    assert_eq!(ahead(&fixture(vec![]), &entries, zl::NORTH), (Yes, pack));
    assert_eq!(
        ahead(&fixture(vec![set_attr(zl::HALL, LIT)]), &entries, zl::NORTH),
        (No, pack)
    );
    // A light carried goes with the player; one lying in the hall is there.
    assert_eq!(
        ahead(&fixture(vec![set_attr(zl::LAMP, LIT)]), &entries, zl::NORTH),
        (No, pack)
    );
    assert_eq!(
        ahead(&fixture(vec![set_attr(zl::BOAT, LIT)]), &entries, zl::NORTH),
        (No, pack),
        "the boat lies in the hall"
    );
    // A light in the kitchen that is not carried stays behind.
    assert_eq!(
        ahead(
            &fixture(vec![set_attr(zl::LAMP, LIT), insert(zl::LAMP, zl::KITCHEN)]),
            &entries,
            zl::NORTH
        ),
        (Yes, pack)
    );
    // A light carried shut in a closed sack does not count.
    let carried_sack = vec![
        set_attr(zl::LAMP, LIT),
        insert(zl::LAMP, zl::SACK),
        insert(zl::SACK, zl::PLAYER),
    ];
    assert_eq!(
        ahead(&fixture(carried_sack), &entries, zl::NORTH),
        (Yes, pack)
    );
    // The kitchen's own light is not the hall's.
    assert_eq!(
        ahead(
            &fixture(vec![set_attr(zl::KITCHEN, LIT)]),
            &entries,
            zl::NORTH
        ),
        (Yes, pack)
    );
    // A door has a room behind it, open or closed; a way a routine decides has none.
    assert_eq!(ahead(&fixture(vec![]), &entries, zl::WEST), (Yes, pack));
    assert_eq!(
        ahead(&fixture(vec![set_attr(zl::HALL, LIT)]), &entries, zl::WEST),
        (No, pack)
    );
    assert_eq!(
        ahead(&fixture(vec![]), &entries, zl::EAST),
        (NotKnown, Source::File)
    );
    // The player cannot be told: what goes with them is not known.
    let tie = vec![
        insert(zl::DIAMOND, zl::KITCHEN),
        set_global(30, zl::DIAMOND),
    ];
    assert_eq!(
        ahead(&fixture(tie.clone()), &entries, zl::NORTH),
        (NotKnown, Source::File)
    );
    let mut lit_hall = tie;
    lit_hall.push(set_attr(zl::HALL, LIT));
    assert_eq!(ahead(&fixture(lit_hall), &entries, zl::NORTH), (No, pack));
    // With no `lit-attribute` nothing is said, and the sources are feature 006's.
    let without = format!("open-attribute: {OPEN}\n");
    assert_eq!(
        ahead(&fixture(vec![]), &without, zl::NORTH),
        (NotKnown, Source::File)
    );
    assert_eq!(
        ahead(&fixture(vec![]), &without, zl::WEST),
        (NotKnown, pack)
    );
}
