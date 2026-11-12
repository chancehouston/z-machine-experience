//! Recognising the family of a story file (specification section 3).

use crate::inform::Inform;
use crate::table::{Bytes, Layout};
use crate::zil::Zil;
use crate::{Part, Tri};

/// The rules for one family, with what they found in the story file as shipped.
#[derive(Clone, Debug)]
pub(crate) enum Adapter {
    Inform(Inform),
    Zil(Zil),
    None,
}

/// Decides the family from the story file as shipped. A file must satisfy every
/// rule of a family; the rules are not scored.
pub(crate) fn recognise(layout: &Layout, shipped: Bytes) -> Adapter {
    if let Some(inform) = Inform::recognise(layout, shipped) {
        return Adapter::Inform(inform);
    }
    if let Some(zil) = Zil::recognise(layout, shipped) {
        return Adapter::Zil(zil);
    }
    Adapter::None
}

/// Whether header bytes 0x3C to 0x3F hold an Inform compiler version: a digit, a
/// full stop and two digits.
pub(crate) fn has_inform_stamp(b: Bytes) -> bool {
    let at = |i: usize| b.byte(0x3C + i).unwrap_or(0);
    at(0).is_ascii_digit() && at(1) == b'.' && at(2).is_ascii_digit() && at(3).is_ascii_digit()
}

/// Where the player is, as a family's rules find it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub room: u16,
    /// The player object, where the family's rule finds exactly one.
    pub player: Part<u16>,
    /// What the player is in or on within the room.
    pub holder: Part<Option<u16>>,
    pub lit: Tri,
}
