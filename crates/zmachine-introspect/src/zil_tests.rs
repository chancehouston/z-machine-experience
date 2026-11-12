//! The ZIL adapter on built story files (docs/introspection.md, cited I§).

use crate::fixtures::{Zil, insert, remove, set_global, zl};
use crate::{Introspector, Tri, Unknown};
use zmachine_core::Machine;

fn introspect(machine: &Machine) -> Introspector {
    Introspector::new(&machine.view())
}

fn after(setup: Vec<Vec<u8>>) -> Machine {
    let mut f = Zil::new();
    f.setup.extend(setup.concat());
    f.machine()
}

// I§6: the room is the object in global 0. ZIL does not record light.
#[test]
fn location_is_global_0() {
    let m = Zil::new().machine();
    let view = m.view();
    let answer = introspect(&m).location(&view);
    assert_eq!(answer.exactness, crate::Exactness::Exact);
    let location = answer.value.unwrap();
    assert_eq!(location.room, zl::KITCHEN);
    assert_eq!(location.name, Ok("Kitchen".to_string()));
    assert_eq!(location.lit, Tri::Unknown);
    assert_eq!(location.holder, Ok(None));
}

// I§6: global 0 holding something that is not a room, nothing, or a number past
// the last object: `no-player`.
#[test]
fn global_0_must_hold_a_room() {
    for value in [0u16, zl::LAMP, zl::ROOMS, 200, 0xFFFF] {
        let m = after(vec![set_global(0, value)]);
        let view = m.view();
        let i = introspect(&m);
        assert_eq!(
            i.location(&view).value.err(),
            Some(Unknown::NoPlayer),
            "global 0 = {value}"
        );
        assert_eq!(i.exits(&view).value.err(), Some(Unknown::NoPlayer));
    }
    // Before the game has run, global 0 is not yet set.
    let fresh = Machine::load(Zil::new().build(), crate::fixtures::OPTIONS).unwrap();
    let view = fresh.view();
    assert_eq!(
        introspect(&fresh).location(&view).value.err(),
        Some(Unknown::NoPlayer)
    );
}

// I§7: the player is the one object, among those unplaced as shipped, that is now
// in the room and whose number some variable other than global 0 holds.
#[test]
fn the_player_is_found_by_the_familys_rule() {
    let m = Zil::new().machine();
    let view = m.view();
    let carried = introspect(&m).inventory(&view).value.unwrap();
    assert_eq!(carried.len(), 1);
    assert_eq!(carried[0].object, zl::LAMP);
    assert_eq!(carried[0].name.as_deref(), Ok("lamp"));
    assert_eq!(
        carried[0].worn,
        Tri::Unknown,
        "ZIL does not record what is worn"
    );
}

// I§7, I§1.4: two candidates in the room, each held by a variable: a tie, which
// is never broken. The room is still exact, and its holder is unknown.
#[test]
fn a_tie_for_the_player_is_ambiguous() {
    let m = after(vec![
        insert(zl::DIAMOND, zl::KITCHEN),
        set_global(30, zl::DIAMOND),
    ]);
    let view = m.view();
    let i = introspect(&m);
    assert_eq!(
        i.inventory(&view).value.err(),
        Some(Unknown::AmbiguousPlayer)
    );
    let location = i.location(&view).value.unwrap();
    assert_eq!(location.room, zl::KITCHEN);
    assert_eq!(location.holder, Err(Unknown::AmbiguousPlayer));
}

// I§7: a candidate in the room that no variable holds does not tie.
#[test]
fn a_candidate_no_variable_holds_is_not_the_player() {
    let m = after(vec![insert(zl::DIAMOND, zl::KITCHEN)]);
    let view = m.view();
    let carried = introspect(&m).inventory(&view).value.unwrap();
    assert_eq!(carried[0].object, zl::LAMP);
}

// I§7: a candidate the player carries is inside another candidate and does not
// tie with it, even when a variable holds its number.
#[test]
fn a_candidate_carried_by_the_player_does_not_tie() {
    let m = after(vec![
        insert(zl::DIAMOND, zl::PLAYER),
        set_global(30, zl::DIAMOND),
    ]);
    let view = m.view();
    let carried = introspect(&m).inventory(&view).value.unwrap();
    let objects: Vec<u16> = carried.iter().map(|c| c.object).collect();
    assert_eq!(objects, [zl::DIAMOND, zl::LAMP]);
}

// I§7: global 0 is the location, not a holder of the player. A candidate whose
// number is only in global 0 is not found by it; and with no candidate in the
// room held by a variable, `no-player`.
#[test]
fn no_candidate_is_no_player() {
    let m = after(vec![set_global(zl::WINNER, 0)]);
    let view = m.view();
    let i = introspect(&m);
    assert_eq!(i.inventory(&view).value.err(), Some(Unknown::NoPlayer));
    assert_eq!(
        i.location(&view).value.unwrap().holder,
        Err(Unknown::NoPlayer)
    );
    let m = after(vec![remove(zl::PLAYER)]);
    let view = m.view();
    assert_eq!(
        introspect(&m).inventory(&view).value.err(),
        Some(Unknown::NoPlayer)
    );
    // The player in another room than global 0 names.
    let m = after(vec![insert(zl::PLAYER, zl::HALL)]);
    let view = m.view();
    assert_eq!(
        introspect(&m).inventory(&view).value.err(),
        Some(Unknown::NoPlayer)
    );
}

// I§6, I§7: the player inside a vehicle in the room is still found, and the
// vehicle is the holder.
#[test]
fn a_player_in_a_vehicle_has_a_holder() {
    let m = after(vec![
        insert(zl::BOAT, zl::KITCHEN),
        insert(zl::PLAYER, zl::BOAT),
    ]);
    let view = m.view();
    let i = introspect(&m);
    let location = i.location(&view).value.unwrap();
    assert_eq!(
        (location.room, location.holder),
        (zl::KITCHEN, Ok(Some(zl::BOAT)))
    );
    assert_eq!(i.inventory(&view).value.unwrap()[0].object, zl::LAMP);
}

// I§7: contents are nested under what holds them.
#[test]
fn carried_containers_nest_their_contents() {
    let m = after(vec![insert(zl::SACK, zl::PLAYER)]);
    let view = m.view();
    let carried = introspect(&m).inventory(&view).value.unwrap();
    assert_eq!(carried[0].object, zl::SACK);
    assert_eq!(carried[0].contents.len(), 1);
    assert_eq!(carried[0].contents[0].object, zl::GARLIC);
    assert_eq!(carried[1].object, zl::LAMP);
}

// I§8, I§3.2: the five forms of exit, by the length of the property.
#[test]
fn exits_by_the_length_of_the_property() {
    use crate::ExitKind;
    let m = Zil::new().machine();
    let view = m.view();
    let answer = introspect(&m).exits(&view);
    assert_eq!(answer.exactness, crate::Exactness::Exact);
    let exits = answer.value.unwrap();
    // North: one byte, a room. South: two bytes, a message: not an exit. East:
    // three bytes, a routine. West: five bytes, a door. Down: four bytes, a flag,
    // which is not set.
    let shape: Vec<(u8, ExitKind, Option<u16>)> = exits
        .exits
        .iter()
        .map(|e| (e.direction, e.kind, e.destination))
        .collect();
    assert_eq!(
        shape,
        [
            (zl::NORTH, ExitKind::Fixed, Some(zl::HALL)),
            (zl::EAST, ExitKind::Conditional, None),
            (zl::WEST, ExitKind::Conditional, None)
        ]
    );
    assert_eq!(
        exits.exits[0].destination_name,
        Some(Ok("Hall".to_string()))
    );
    assert!(exits.unknown.is_empty());
}

// I§8: an exit a stored value opens. The byte names a Z-machine variable, 16
// being the first global: the exit is there while that global is not zero.
#[test]
fn a_flag_exit_follows_its_flag() {
    let down = |m: &Machine| {
        let view = m.view();
        let exits = introspect(m).exits(&view).value.unwrap();
        exits.exits.into_iter().find(|e| e.direction == zl::DOWN)
    };
    assert_eq!(down(&Zil::new().machine()), None, "the flag is clear");
    let open = down(&after(vec![set_global(zl::TRAP_OPEN, 1)])).expect("the flag is set");
    assert_eq!(
        (open.kind, open.destination),
        (crate::ExitKind::Fixed, Some(zl::CELLAR))
    );
    assert_eq!(open.destination_name, Some(Ok("Cellar".to_string())));
    assert_eq!(open.words, ["down"]);
    // Any non-zero value opens it; the neighbouring global does not.
    assert!(down(&after(vec![set_global(zl::TRAP_OPEN, 0xFFFF)])).is_some());
    assert!(
        down(&after(vec![set_global(zl::TRAP_OPEN + 16, 1)])).is_none(),
        "the byte is a variable number, not an index"
    );
    assert!(down(&after(vec![set_global(zl::TRAP_OPEN - 1, 1)])).is_none());
}

// I§8: a flag byte below 16 names the stack or a local, which mean nothing
// outside a routine: the exit is conditional.
#[test]
fn a_flag_that_is_not_a_global_is_conditional() {
    let m = after(vec![
        insert(zl::PLAYER, zl::CELLAR),
        set_global(0, zl::CELLAR),
    ]);
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    assert_eq!(exits.exits.len(), 1);
    assert_eq!(
        (
            exits.exits[0].direction,
            exits.exits[0].kind,
            exits.exits[0].destination
        ),
        (zl::UP, crate::ExitKind::Conditional, None)
    );
}

// I§8, I§3.2: every dictionary word for the direction, shortest first; and an
// exit that leads back to its own room is reported like any other.
#[test]
fn direction_words_and_an_exit_to_the_same_room() {
    let m = Zil::new().machine();
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    assert_eq!(exits.exits[0].words, ["n", "north"]);
    assert_eq!(exits.exits[2].words, ["west"]);
    let m = after(vec![insert(zl::PLAYER, zl::HALL), set_global(0, zl::HALL)]);
    let view = m.view();
    let exits = introspect(&m).exits(&view).value.unwrap();
    let north = exits
        .exits
        .iter()
        .find(|e| e.direction == zl::NORTH)
        .unwrap();
    assert_eq!(north.destination, Some(zl::HALL));
}
