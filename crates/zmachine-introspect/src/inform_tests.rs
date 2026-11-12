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
