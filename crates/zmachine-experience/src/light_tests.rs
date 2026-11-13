//! The warning of darkness, from hand-made introspection answers
//! (specs/008-darkness/research.md R10; FR-018, FR-021).

use crate::light::{Light, NOTE_DARK};
use zmachine_introspect::{Answer, Exactness, Location, Source, Tri, Unknown};

fn location(lit: Tri) -> Answer<Location> {
    Answer::exact(Location {
        room: 7,
        name: Ok("Somewhere".to_string()),
        holder: Ok(None),
        lit,
        source: Source::Pack,
    })
}

// FR-018: where the interface says the place is dark, the layer says so, in
// words a host shows as they are.
#[test]
fn a_dark_place_has_the_note() {
    let light = Light::of(&location(Tri::No));
    assert_eq!((light.dark, light.note), (Some(true), Some("It is dark")));
    assert_eq!(NOTE_DARK, "It is dark");
}

#[test]
fn a_lit_place_has_no_note() {
    let light = Light::of(&location(Tri::Yes));
    assert_eq!((light.dark, light.note), (Some(false), None));
}

// FR-021: where light is not known nothing is said, and nothing is guessed.
#[test]
fn where_light_is_not_known_nothing_is_said() {
    let light = Light::of(&location(Tri::Unknown));
    assert_eq!((light.dark, light.note), (None, None));
    let unknown = Answer::unknown(Unknown::UnrecognisedFamily, Exactness::Exact);
    assert_eq!(Light::of(&unknown), Light::default());
    assert_eq!(Light::default().dark, None);
}
