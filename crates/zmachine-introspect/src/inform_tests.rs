//! The Inform 6 adapter on built story files (docs/introspection.md, cited I§).

use crate::fixtures::{Inform, Late, clear_attr, inf, insert, remove, set_attr, set_global};
use crate::{Introspector, Tri, Unknown};
use zmachine_core::Machine;

fn introspect(machine: &Machine) -> Introspector {
    Introspector::new(&machine.view())
}

/// The fixture with extra start-up code.
fn after(setup: Vec<Vec<u8>>) -> Machine {
    let mut f = Inform::new();
    f.setup.extend(setup.concat());
    f.machine()
}

// I§6: the room is the top of the player's chain of parents; `holder` is `none`
// when the player is directly in it; a lit room is `lit` `yes`.
#[test]
fn location_is_the_top_of_the_players_parents() {
    let m = Inform::new().machine();
    let view = m.view();
    let location = introspect(&m).location(&view).value.unwrap();
    assert_eq!(location.room, inf::KITCHEN);
    assert_eq!(location.name, Ok("Kitchen".to_string()));
    assert_eq!(location.holder, Ok(None));
    assert_eq!(location.lit, Tri::Yes);
}

// I§6: a player inside something inside the room: the room, and the holder.
#[test]
fn a_player_inside_something_has_a_holder() {
    let m = after(vec![insert(inf::PLAYER, inf::CAGE)]);
    let view = m.view();
    let location = introspect(&m).location(&view).value.unwrap();
    assert_eq!(location.room, inf::KITCHEN);
    assert_eq!(location.holder, Ok(Some(inf::CAGE)));
}

// I§6: a player with no parent is nowhere: `no-player`.
#[test]
fn a_player_with_no_parent_is_no_player() {
    let m = after(vec![remove(inf::PLAYER)]);
    let view = m.view();
    let i = introspect(&m);
    assert_eq!(i.location(&view).value.err(), Some(Unknown::NoPlayer));
    assert_eq!(i.inventory(&view).value.err(), Some(Unknown::NoPlayer));
    // Before the game has run at all the player is not yet placed.
    let fresh = Machine::load(Inform::new().build(), crate::fixtures::OPTIONS).unwrap();
    let view = fresh.view();
    assert_eq!(
        introspect(&fresh).location(&view).value.err(),
        Some(Unknown::NoPlayer)
    );
}

// I§6: the library's `location` variable must hold the room found from the tree,
// or the darkness object. Anything else is `inconsistent`.
#[test]
fn the_location_variable_must_agree_with_the_tree() {
    let m = after(vec![set_global(inf::LOCATION, inf::HALL)]);
    let view = m.view();
    let i = introspect(&m);
    assert_eq!(i.location(&view).value.err(), Some(Unknown::Inconsistent));
    assert_eq!(i.inventory(&view).value.err(), Some(Unknown::Inconsistent));
}

// I§6: darkness. The variable holds the darkness object, the tree still holds
// the room: the room is reported, with `lit` `no`.
#[test]
fn darkness_is_the_darkness_object_in_the_location_variable() {
    let m = after(vec![set_global(inf::LOCATION, inf::DARKNESS)]);
    let view = m.view();
    let location = introspect(&m).location(&view).value.unwrap();
    assert_eq!(location.room, inf::KITCHEN);
    assert_eq!(location.lit, Tri::No);
}

// I§6, I§10: the guard against a change of player. If no variable holds the
// number of `(self object)`, some other object is the player: `inconsistent`.
#[test]
fn no_variable_holding_the_player_is_inconsistent() {
    let m = after(vec![set_global(inf::PLAYER_VARIABLE, inf::LAMP)]);
    let view = m.view();
    let i = introspect(&m);
    assert_eq!(i.location(&view).value.err(), Some(Unknown::Inconsistent));
    assert_eq!(i.inventory(&view).value.err(), Some(Unknown::Inconsistent));
}

// I§3.1 printed names: the `short_name` string when there is one; the stored
// name when the object has no `short_name`; unknown when it holds a routine.
#[test]
fn printed_names_follow_short_name() {
    let m = after(vec![
        insert(inf::PLAYER, inf::MAZE),
        set_global(inf::LOCATION, inf::MAZE),
    ]);
    let view = m.view();
    let i = introspect(&m);
    assert_eq!(
        i.location(&view).value.unwrap().name,
        Ok("Maze".to_string())
    );
    let tree = i.object_tree(&view).value.unwrap();
    let maze = &tree[inf::MAZE as usize - 1];
    assert_eq!(
        (maze.name.as_str(), maze.printed_name.as_deref()),
        ("(Maze_1)", Ok("Maze"))
    );
    assert_eq!(
        tree[inf::KITCHEN as usize - 1].printed_name.as_deref(),
        Ok("Kitchen")
    );

    let mut f = Inform::new();
    f.late(inf::KITCHEN, 46, Late::Routine(0));
    let m = f.machine();
    let view = m.view();
    let location = introspect(&m).location(&view).value.unwrap();
    assert_eq!(location.room, inf::KITCHEN);
    assert_eq!(location.name, Err(Unknown::NotDeterminable));
}

// I§7: what the player carries, in the tree's order, with contents nested; `worn`
// is attribute 26; an empty list when nothing is carried.
#[test]
fn inventory_is_the_players_children() {
    let m = Inform::new().machine();
    let view = m.view();
    assert_eq!(introspect(&m).inventory(&view).value, Ok(vec![]));

    let m = after(vec![
        insert(inf::LAMP, inf::PLAYER),
        insert(inf::BOX, inf::PLAYER),
        insert(inf::HAT, inf::PLAYER),
        set_attr(inf::HAT, 26),
    ]);
    let view = m.view();
    let answer = introspect(&m).inventory(&view);
    assert_eq!(answer.exactness, crate::Exactness::Exact);
    let carried = answer.value.unwrap();
    // `insert_obj` makes the newest the first child.
    let names: Vec<&str> = carried.iter().map(|c| c.name.as_deref().unwrap()).collect();
    assert_eq!(names, ["hat", "box", "lamp"]);
    assert_eq!(carried[0].worn, Tri::Yes);
    assert_eq!(carried[1].worn, Tri::No);
    assert_eq!(carried[1].contents.len(), 1);
    assert_eq!(carried[1].contents[0].name.as_deref(), Ok("coin"));
    assert_eq!(carried[1].contents[0].object, inf::COIN);
    assert!(carried[2].contents.is_empty());
    let _ = clear_attr;
}

// I§8, I§3.1: each kind of value a direction property can hold, tested in the
// specification's order.
#[test]
fn exits_by_the_kind_of_value() {
    use crate::{ExitKind, Exits};
    let exits_of = |f: &Inform| -> Exits {
        let m = f.machine();
        let view = m.view();
        let answer = introspect(&m).exits(&view);
        assert_eq!(answer.exactness, crate::Exactness::Exact);
        answer.value.unwrap()
    };
    // The kitchen: north to the hall (a room); south a string; in a routine; out
    // the door object.
    let exits = exits_of(&Inform::new());
    let shape: Vec<(u8, ExitKind, Option<u16>)> = exits.exits.iter().map(|e| (e.direction, e.kind, e.destination)).collect();
    assert_eq!(shape, [(7, ExitKind::Fixed, Some(inf::HALL)), (17, ExitKind::Conditional, None), (18, ExitKind::Conditional, None)]);
    assert_eq!(exits.exits[0].destination_name, Some(Ok("Hall".to_string())));
    assert_eq!(exits.exits[1].destination_name, None);
    assert!(exits.unknown.is_empty(), "south is refused by a string: in neither list");

    // A value of 0 is no exit.
    let mut f = Inform::new();
    f.prop(inf::KITCHEN, 7, vec![0, 0]);
    assert!(!exits_of(&f).exits.iter().any(|e| e.direction == 7));

    // An object that is neither a room nor a door: conditional.
    let mut f = Inform::new();
    f.prop(inf::KITCHEN, 7, inf::LAMP.to_be_bytes().to_vec());
    let exits = exits_of(&f);
    assert_eq!((exits.exits[0].direction, exits.exits[0].kind, exits.exits[0].destination), (7, ExitKind::Conditional, None));

    // A property that is not two bytes: that direction is unknown.
    for data in [vec![9u8], vec![0, 14, 0, 13]] {
        let mut f = Inform::new();
        f.prop(inf::KITCHEN, 7, data.clone());
        let exits = exits_of(&f);
        assert!(!exits.exits.iter().any(|e| e.direction == 7), "{data:?}");
        assert_eq!(exits.unknown.len(), 1);
        assert_eq!((exits.unknown[0].direction, exits.unknown[0].reason), (7, Unknown::NotDeterminable));
        assert_eq!(exits.unknown[0].words, ["n", "north"]);
    }
}

// I§8: a door is tested for before a room. A door that is in several rooms has
// no parent, as a room has none, and must not be taken for one.
#[test]
fn a_parentless_door_is_conditional_not_a_room() {
    let m = Inform::new().machine();
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    let out = exits.exits.iter().find(|e| e.direction == 18).unwrap();
    assert_eq!((out.kind, out.destination), (crate::ExitKind::Conditional, None));
    assert_eq!(out.words, ["out"]);
}

// I§3.1: the words for a direction. From the compass child's `name`, shortest
// first; `in` and `out` from the dictionary only where they are verbs there.
#[test]
fn direction_words() {
    let m = Inform::new().machine();
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    assert_eq!(exits.exits[0].words, ["n", "north"]);
    assert_eq!(exits.exits[1].words, ["in"]);

    let mut f = Inform::new();
    f.words.retain(|(w, _)| *w != "in");
    f.words.push(("in", [0x80, 0, 0]));
    let m = f.machine();
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    let inward = exits.exits.iter().find(|e| e.direction == 17).unwrap();
    assert!(inward.words.is_empty(), "`in` is not a verb in this dictionary: {:?}", inward.words);
}

// I§3.1: telling a routine from a string. Properties 1, 2 and 3 are left out of
// the search; a value between the bounds, and every value when the bounds cannot
// be trusted, is not determinable.
#[test]
fn routines_and_strings_are_told_apart_by_the_layout() {
    // Property 3 on the compass children holds 0, and property 2 holds 1: neither
    // misleads here. Give property 3 a value that, taken as a packed address,
    // would land on a byte above 15 in high memory, below every routine.
    let mut f = Inform::new();
    let image = f.build();
    let high = u16::from_be_bytes([image[4], image[5]]) as usize;
    // The main code begins at the high-memory base; find a byte above 15 there.
    let offset = (0..64).find(|&i| (high + i) % 4 == 0 && image[high + i] > 15).expect("a byte above 15 in the main code");
    let misleading = ((high + offset) / 4) as u16;
    f.prop(inf::N, 3, misleading.to_be_bytes().to_vec());
    let m = f.machine();
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    let kinds: Vec<(u8, crate::ExitKind)> = exits.exits.iter().map(|e| (e.direction, e.kind)).collect();
    assert_eq!(kinds, [(7, crate::ExitKind::Fixed), (17, crate::ExitKind::Conditional), (18, crate::ExitKind::Conditional)], "the routine at `in` is still a routine");
    assert!(exits.unknown.is_empty());

    // With no property that is certainly a routine (before, after, life), the
    // upper bound of the routines is not known: nothing is determinable.
    let mut f = Inform::new();
    f.without_prop(inf::KITCHEN, 4).without_prop(inf::HALL, 4);
    let m = f.machine();
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    let unknown: Vec<u8> = exits.unknown.iter().map(|u| u.direction).collect();
    assert_eq!(unknown, [8, 17], "the string and the routine can no longer be told apart");
    assert_eq!(exits.exits.iter().map(|e| e.direction).collect::<Vec<_>>(), [7, 18]);

    // A routine above every routine known to be one lies between the bounds.
    let mut f = Inform::new();
    f.without_prop(inf::HALL, 4);
    f.late(inf::KITCHEN, 9, Late::Routine(3));
    let m = f.machine();
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    assert!(exits.unknown.iter().any(|u| u.direction == 17), "`in` is laid out after the last certain routine: {exits:?}");
}

// I§8: unknown as a whole only when the location is.
#[test]
fn exits_are_unknown_when_the_location_is() {
    let m = after(vec![remove(inf::PLAYER)]);
    let view = m.view();
    assert_eq!(introspect(&m).exits(&view).value.err(), Some(Unknown::NoPlayer));
}
