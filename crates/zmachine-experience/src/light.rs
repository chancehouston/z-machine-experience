//! Whether the player is in the dark, for a host to show and to speak
//! (specs/008-darkness/research.md R10).

use zmachine_introspect::{Answer, Location, Tri};

/// Whether the player is in the dark at a stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Light {
    /// Whether it is dark where the player is; none where that is not known.
    pub dark: Option<bool>,
    /// What to show, if anything.
    pub note: Option<&'static str>,
}

/// The note shown where the player is in the dark.
pub const NOTE_DARK: &str = "It is dark";

impl Light {
    /// The light for a location answer, as the introspection interface gives
    /// it. Nothing is said where the answer does not say.
    pub(crate) fn of(location: &Answer<Location>) -> Light {
        match location.value.as_ref().map(|l| l.lit) {
            Ok(Tri::No) => Light {
                dark: Some(true),
                note: Some(NOTE_DARK),
            },
            Ok(Tri::Yes) => Light {
                dark: Some(false),
                note: None,
            },
            _ => Light::default(),
        }
    }
}
