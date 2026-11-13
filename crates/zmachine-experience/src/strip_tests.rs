//! The strip, from hand-made inventory answers
//! (specs/007-inventory-panel/data-model.md "Strip"; research R10; FR-019 to
//! FR-022; docs/introspection.md section 7, I§7).

use crate::strip::{ItemState, NOTE_NOT_KNOWN, NOTE_NOTHING, Strip};
use zmachine_introspect::{Answer, Carried, ContainerState, Exactness, Source, Tri, Unknown};

fn item(name: &str, seen: Tri, state: Option<ContainerState>, contents: Vec<Carried>) -> Carried {
    Carried {
        object: 1,
        name: Ok(name.to_string()),
        worn: Tri::Unknown,
        state,
        seen,
        source: Source::File,
        contents,
    }
}

fn plain(name: &str) -> Carried {
    item(name, Tri::Yes, None, vec![])
}

fn strip(carried: Vec<Carried>) -> Strip {
    Strip::of(&Answer::exact(carried))
}

/// The strip's items as (name, state, names of what is shown inside).
fn shape(strip: &Strip) -> Vec<(&str, ItemState, Vec<&str>)> {
    strip
        .items
        .iter()
        .map(|i| {
            (
                i.name.as_str(),
                i.state,
                i.contents.iter().map(|c| c.name.as_str()).collect(),
            )
        })
        .collect()
}

// FR-019, FR-022: nothing carried is known, and said in words.
#[test]
fn nothing_carried_is_said() {
    let s = strip(vec![]);
    assert!(s.known);
    assert!(s.items.is_empty());
    assert_eq!(s.note, Some(NOTE_NOTHING));
    assert_eq!(s.spoken, "Carrying nothing");
}

// FR-019: the items are the answer's, in its order, by the game's names.
#[test]
fn plain_items_in_the_answers_order() {
    let s = strip(vec![
        plain("brass lantern"),
        plain("leaflet"),
        plain("sword"),
    ]);
    assert!(s.known);
    assert_eq!(s.note, None);
    assert_eq!(
        shape(&s),
        [
            ("brass lantern", ItemState::Plain, vec![]),
            ("leaflet", ItemState::Plain, vec![]),
            ("sword", ItemState::Plain, vec![])
        ]
    );
    assert_eq!(s.spoken, "Carrying: brass lantern; leaflet; sword");
}

// D§ "Strip", row by row: what a carried entry becomes.
#[test]
fn a_container_is_shown_by_its_state() {
    use ContainerState::{Closed, Open, Unknown};
    let garlic = |seen| item("garlic", seen, None, vec![]);
    // Closed, and what it holds is not seen.
    let s = strip(vec![item(
        "sack",
        Tri::Yes,
        Some(Closed),
        vec![garlic(Tri::No)],
    )]);
    assert_eq!(shape(&s), [("sack", ItemState::Closed, vec![])]);
    assert_eq!(s.spoken, "Carrying: sack, closed");
    // Open, with what it holds.
    let s = strip(vec![item(
        "sack",
        Tri::Yes,
        Some(Open),
        vec![garlic(Tri::Yes), item("lunch", Tri::Yes, None, vec![])],
    )]);
    assert_eq!(
        shape(&s),
        [("sack", ItemState::Open, vec!["garlic", "lunch"])]
    );
    assert_eq!(s.spoken, "Carrying: sack, open, holding garlic and lunch");
    // Closed and see-through: closed, with what it holds.
    let s = strip(vec![item(
        "bottle",
        Tri::Yes,
        Some(Closed),
        vec![item("water", Tri::Yes, None, vec![])],
    )]);
    assert_eq!(shape(&s), [("bottle", ItemState::Closed, vec!["water"])]);
    assert_eq!(s.spoken, "Carrying: bottle, closed, holding water");
    // State unknown: a plain item, and nothing of what it holds.
    let s = strip(vec![item(
        "sack",
        Tri::Yes,
        Some(Unknown),
        vec![garlic(Tri::Unknown)],
    )]);
    assert_eq!(shape(&s), [("sack", ItemState::Plain, vec![])]);
    assert_eq!(s.spoken, "Carrying: sack");
}

// FR-020: a thing that is not seen, or not known to be seen, is not an item at
// any depth, and nothing about it is said.
#[test]
fn what_is_not_seen_is_kept_back_at_any_depth() {
    use ContainerState::{Closed, Open};
    let deep = item(
        "sack",
        Tri::Yes,
        Some(Open),
        vec![
            item(
                "box",
                Tri::Yes,
                Some(Closed),
                vec![item("ruby", Tri::No, None, vec![])],
            ),
            item(
                "pouch",
                Tri::Yes,
                Some(Open),
                vec![item("coin", Tri::Yes, None, vec![])],
            ),
            item("secret", Tri::Unknown, None, vec![]),
        ],
    );
    let s = strip(vec![deep, plain("lamp")]);
    let sack = &s.items[0];
    assert_eq!(
        sack.contents
            .iter()
            .map(|c| (c.name.as_str(), c.state))
            .collect::<Vec<_>>(),
        [("box", ItemState::Closed), ("pouch", ItemState::Open)]
    );
    assert!(sack.contents[0].contents.is_empty());
    assert_eq!(sack.contents[1].contents[0].name, "coin");
    for hidden in ["ruby", "secret"] {
        assert!(!s.spoken.contains(hidden), "{}", s.spoken);
    }
    assert_eq!(
        s.spoken,
        "Carrying: sack, open, holding box, closed, and pouch, open, holding coin; lamp"
    );
    // Two things of one name are both shown.
    let s = strip(vec![plain("coin"), plain("coin")]);
    assert_eq!(s.items.len(), 2);
    // A thing whose name could not be read is still shown.
    let mut nameless = plain("");
    nameless.name = Err(Unknown::Damaged);
    assert_eq!(strip(vec![nameless]).items[0].name, "something");
}

// FR-022; US4 scenarios 1 and 4: where the interface cannot say what is carried,
// the strip says so and shows nothing.
#[test]
fn an_unknown_inventory_is_said_to_be_unknown() {
    for reason in Unknown::ALL {
        let s = Strip::of(&Answer::unknown(reason, Exactness::Exact));
        assert!(!s.known, "{reason:?}");
        assert!(s.items.is_empty());
        assert_eq!(s.note, Some(NOTE_NOT_KNOWN));
        assert_eq!(s.spoken, "Not known: type to ask");
    }
}
