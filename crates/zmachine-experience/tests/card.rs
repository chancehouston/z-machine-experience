//! The inventory card, on Zork I and Adventure
//! (specs/014-inventory-card/contracts/card.md, cited L§ and S§).

mod common;

use common::{read, read_text, say};
use zmachine_experience::{Outcome, Refused, Seed, Session, Source, card_json};

fn zork() -> Session {
    Session::with_pack(
        read("stories/zork1/zork1.z3"),
        Seed::Fixed(11),
        Some(&read_text("packs/zork1.pack")),
    )
    .unwrap()
}

/// Zork I with the leaflet in hand.
fn with_leaflet() -> Session {
    let mut s = zork();
    for line in ["open mailbox", "take leaflet"] {
        say(&mut s, line);
    }
    s
}

// L§2, S§1: the leaflet is the one item, and can be tapped.
#[test]
fn a_carried_item_can_be_tapped() {
    let mut s = zork();
    assert!(s.inventory().items.is_empty());
    say(&mut s, "open mailbox");
    say(&mut s, "take leaflet");
    let items = &s.inventory().items;
    assert_eq!(items.len(), 1);
    assert_eq!(
        (items[0].name.as_str(), items[0].index, items[0].card),
        ("leaflet", 0, true)
    );
    // At a question nothing can be tapped.
    say(&mut s, "quit");
    assert!(s.inventory().items.iter().all(|item| !item.card));
    assert_eq!(s.card(s.stamp(), 0).err(), Some(Refused::NotOffered));
}

// L§4, L§5, L§6; SC-002: the card's description is the reply the game gives to
// the same line, and making it leaves the game exactly as it was.
#[test]
fn a_card_holds_the_games_description_and_changes_nothing() {
    let s = with_leaflet();
    let (kept, snapshot, turns, page) = (
        s.kept(),
        s.snapshot_json(0),
        s.turns().len(),
        s.map().page(),
    );
    let card = s.card(s.stamp(), 0).expect("a card");
    assert_eq!(card.name, "leaflet");
    assert_eq!(card.note, None);
    let description = card.description.clone().expect("a description");
    assert!(description.contains("ZORK"), "{description}");
    assert_eq!(card.spoken, format!("leaflet. {description}"));
    // The game has not moved.
    assert_eq!(s.kept(), kept);
    assert_eq!(s.snapshot_json(0), snapshot);
    assert_eq!(s.turns().len(), turns);
    assert_eq!(s.map().page(), page);
    // The same again.
    assert_eq!(s.card(s.stamp(), 0).unwrap(), card);
    // What the game says when the line is typed.
    let mut typed = with_leaflet();
    say(&mut typed, "examine leaflet");
    let said = typed.turns().last().unwrap().text.clone();
    assert_eq!(said.trim_end().trim_end_matches('>').trim(), description);
    // And a game left after a card comes back as it was.
    let (brought, outcome) = Session::open_kept(
        read("stories/zork1/zork1.z3"),
        Some(&read_text("packs/zork1.pack")),
        Seed::Fresh(5),
        Some(&s.kept()),
    )
    .unwrap();
    assert_eq!(outcome, Outcome::BroughtBack);
    assert_eq!(brought.snapshot_json(0), snapshot);
}

// L§3: a stale stamp and an item that is not there are refused.
#[test]
fn a_card_is_refused_for_another_turn_or_no_item() {
    let mut s = with_leaflet();
    let mut stale = s.stamp();
    stale.turn_count -= 1;
    assert_eq!(s.card(stale, 0).err(), Some(Refused::Stale));
    assert_eq!(s.card(s.stamp(), 7).err(), Some(Refused::NotOffered));
    assert_eq!(s.tap_action(stale, 0, 0), Err(Refused::Stale));
    assert_eq!(s.tap_action(s.stamp(), 7, 0), Err(Refused::NotOffered));
    assert_eq!(s.tap_action(s.stamp(), 0, 5), Err(Refused::NotOffered));
    assert_eq!(s.turns().len(), 3);
}

// L§7, L§8: Drop is on the card of an item held in the hand, and sends the
// story's word for dropping with the item's word, as a tap.
#[test]
fn an_item_is_dropped_from_its_card() {
    let mut s = with_leaflet();
    let card = s.card(s.stamp(), 0).unwrap();
    let actions: Vec<(&str, &str, &str)> = card
        .actions
        .iter()
        .map(|a| (a.label.as_str(), a.spoken.as_str(), a.command.as_str()))
        .collect();
    assert_eq!(actions, [("Drop", "Drop leaflet", "drop leaflet")]);
    s.tap_action(s.stamp(), 0, 0).expect("the drop is sent");
    let turn = s.turns().last().unwrap();
    assert_eq!(
        (turn.command.as_deref(), turn.source),
        (Some("drop leaflet"), Some(Source::Tap))
    );
    assert!(s.inventory().items.is_empty(), "{:?}", s.inventory().items);
}

// L§7: an item inside another has a card, and no Drop.
#[test]
fn an_item_inside_another_has_no_drop() {
    let mut s = zork();
    for line in [
        "north",
        "east",
        "open window",
        "west",
        "take sack",
        "open sack",
    ] {
        say(&mut s, line);
    }
    let sack = s.inventory().items[0].clone();
    assert_eq!(sack.name, "brown sack");
    let inside = sack.contents.first().expect("the sack shows what it holds");
    assert!(inside.card);
    let card = s.card(s.stamp(), inside.index).unwrap();
    assert!(card.actions.is_empty(), "{:?}", card.actions);
    assert!(card.description.is_some());
    let card = s.card(s.stamp(), sack.index).unwrap();
    assert_eq!(card.actions.len(), 1);
}

// SC-002 on Adventure: the lamp's card is the game's own description.
#[test]
fn adventures_lamp_has_a_card() {
    let mut s = Session::new(read("stories/advent/advent.z5"), Seed::Fixed(11)).unwrap();
    for line in ["east", "take lamp"] {
        say(&mut s, line);
    }
    let lamp = s
        .inventory()
        .items
        .iter()
        .find(|item| item.name.contains("lamp") || item.name.contains("lantern"))
        .expect("the lamp is carried")
        .clone();
    assert!(lamp.card);
    let card = s.card(s.stamp(), lamp.index).unwrap();
    let description = card.description.expect("a description");
    let mut typed = Session::new(read("stories/advent/advent.z5"), Seed::Fixed(11)).unwrap();
    for line in ["east", "take lamp"] {
        say(&mut typed, line);
    }
    say(&mut typed, &format!("examine {}", lamp.word().unwrap()));
    let said = typed.turns().last().unwrap().text.clone();
    assert_eq!(said.trim_end().trim_end_matches('>').trim(), description);
}

// S§2: the card as JSON, key for key.
#[test]
fn a_card_as_json() {
    let s = with_leaflet();
    let card = s.card(s.stamp(), 0).unwrap();
    let json = card_json(&card);
    assert!(
        json.starts_with(r#"{"name":"leaflet","state":"plain","description":""#),
        "{json}"
    );
    for key in [
        "\"note\":null",
        "\"spoken\":\"leaflet. ",
        "\"actions\":[{\"label\":\"Drop\",\"spoken\":\"Drop leaflet\",\"command\":\"drop leaflet\"}]}",
    ] {
        assert!(json.contains(key), "{key}: {json}");
    }
    // A card with a description carries no words of the layer's about it
    // (the owner's ruling at the phone trial, 2026-11-14).
    for gone in ["ahead", "not been told"] {
        assert!(!json.contains(gone), "{gone}: {json}");
    }
}
