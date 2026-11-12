//! The adapter for Inform 6 with its standard library (specification section 3.1).
//!
//! The numbers below are fixed by the order in which the library declares its
//! attributes and properties; recognition checks that the story file agrees.

use crate::family::{Place, Way, has_inform_stamp};
use crate::table::{Bytes, Layout};
use crate::{Part, Tri, Unknown};

pub(crate) mod attribute {
    pub const ANIMATE: u8 = 0;
    pub const ABSENT: u8 = 1;
    pub const CONCEALED: u8 = 3;
    pub const CONTAINER: u8 = 4;
    pub const DOOR: u8 = 5;
    pub const LIGHT: u8 = 9;
    pub const OPEN: u8 = 14;
    pub const PROPER: u8 = 16;
    pub const SCENERY: u8 = 17;
    pub const SUPPORTER: u8 = 20;
    pub const TRANSPARENT: u8 = 23;
    pub const WORN: u8 = 26;
}

pub(crate) mod property {
    pub const NAME: u8 = 1;
    pub const BEFORE: u8 = 4;
    pub const LIFE: u8 = 6;
    pub const IN_TO: u8 = 17;
    pub const OUT_TO: u8 = 18;
    pub const DOOR_DIR: u8 = 21;
    pub const FOUND_IN: u8 = 39;
    pub const SHORT_NAME: u8 = 46;
}

/// The first data byte of a dictionary entry (`#dict_par1`).
pub(crate) mod flag {
    pub const VERB: u8 = 0x01;
    pub const META: u8 = 0x02;
    pub const PLURAL: u8 = 0x04;
    pub const GRAMMAR: u8 = 0x08;
    pub const NOUN: u8 = 0x80;
}

/// One direction of the compass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Direction {
    /// The property a room gives its exit in.
    pub property: u8,
    /// The dictionary words that name it, shortest first.
    pub words: Vec<String>,
}

/// What recognition found in the story file as shipped.
#[derive(Clone, Debug)]
pub(crate) struct Inform {
    pub player: u16,
    pub darkness: u16,
    /// Index of the library's `location` global variable.
    pub location_global: usize,
    pub directions: Vec<Direction>,
    /// The lowest packed address known to be a string: everything from here up
    /// is one.
    pub strings_from: Option<u16>,
    /// The highest packed address known to be a routine: every non-object value
    /// up to here is one.
    pub routines_to: Option<u16>,
}

impl Inform {
    /// Applies the rules of specification section 3.1 to the file as shipped.
    pub fn recognise(layout: &Layout, b: Bytes) -> Option<Inform> {
        // Rule 1: the compiler's stamp.
        if !has_inform_stamp(b) {
            return None;
        }
        // Rule 2: the four metaclasses.
        for (number, name) in [(1, "Class"), (2, "Object"), (3, "Routine"), (4, "String")] {
            if layout.name(b, number).as_deref() != Some(name) {
                return None;
            }
        }
        // Rule 3: exactly one object with each of the library's names.
        let mut found = [None; 4];
        let names = [
            "compass",
            "(self object)",
            "(darkness object)",
            "(Inform Library)",
        ];
        for number in 1..=layout.last_object {
            let Some(name) = layout.name(b, number) else {
                continue;
            };
            if let Some(i) = names.iter().position(|n| *n == name) {
                if found[i].is_some() {
                    return None;
                }
                found[i] = Some(number);
            }
        }
        let [Some(compass), Some(player), Some(darkness), Some(library)] = found else {
            return None;
        };
        // Rule 4: the compass's children each name a direction property.
        let children = layout.children(b, compass)?;
        if children.is_empty() {
            return None;
        }
        let mut directions: Vec<Direction> = Vec::new();
        for &child in &children {
            let value = layout.property_word(b, child, property::DOOR_DIR)?;
            let own: Vec<u8> = layout
                .properties(b, child)?
                .iter()
                .map(|p| p.number)
                .collect();
            let fits = (4..=63).contains(&value) && !own.contains(&(value as u8));
            if !fits || directions.iter().any(|d| d.property == value as u8) {
                return None;
            }
            let words = direction_words(layout, b, child, value as u8)?;
            directions.push(Direction {
                property: value as u8,
                words,
            });
        }
        // Rule 5: the attributes are numbered as the library declares them.
        use attribute::*;
        if layout.attributes(b, player)? != [ANIMATE, CONCEALED, PROPER, TRANSPARENT]
            || layout.attributes(b, compass)? != [CONCEALED]
            || children
                .iter()
                .any(|&c| layout.attributes(b, c).as_deref() != Some(&[SCENERY]))
        {
            return None;
        }
        // Rule 6: exactly one global holds the library object: `location`.
        let mut holders = (0..240).filter(|&i| layout.global(b, i) == Some(library));
        let location_global = holders.next()?;
        if holders.next().is_some() {
            return None;
        }
        let (strings_from, routines_to) = code_and_string_bounds(layout, b);
        Some(Inform {
            player,
            darkness,
            location_global,
            directions,
            strings_from,
            routines_to,
        })
    }
}

/// The words that name a compass direction: those in its `name` property. The
/// library takes `in` and `out` as verbs, and their compass objects have no
/// `name`; for them the words are `in` and `out` where the dictionary has those
/// as verbs.
fn direction_words(layout: &Layout, b: Bytes, child: u16, direction: u8) -> Option<Vec<String>> {
    let mut words = Vec::new();
    match layout.property(b, child, property::NAME)? {
        Some(name) => {
            for i in (0..name.len.saturating_sub(1)).step_by(2) {
                let entry = b.word(name.at + i)? as usize;
                words.extend(layout.word(b, entry));
            }
        }
        None => {
            let verb = match direction {
                property::IN_TO => Some("in"),
                property::OUT_TO => Some("out"),
                _ => None,
            };
            if let Some(verb) = verb.filter(|v| is_verb(layout, b, v)) {
                words.push(verb.to_string());
            }
        }
    }
    words.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
    words.dedup();
    Some(words)
}

fn is_verb(layout: &Layout, b: Bytes, word: &str) -> bool {
    let Some(d) = layout.dictionary(b) else {
        return false;
    };
    (0..d.count).any(|i| {
        let entry = d.first + i * d.entry_len;
        layout.word(b, entry).as_deref() == Some(word)
            && b.byte(entry + d.text_len)
                .is_some_and(|flags| flags & flag::VERB != 0)
    })
}

/// Finds the bounds that tell a routine from a string (specification section
/// 3.1): Inform places all routines before all strings.
fn code_and_string_bounds(layout: &Layout, b: Bytes) -> (Option<u16>, Option<u16>) {
    let (mut strings_from, mut routines_to): (Option<u16>, Option<u16>) = (None, None);
    for number in 1..=layout.last_object {
        let Some(properties) = layout.properties(b, number) else {
            continue;
        };
        for p in properties {
            // Properties 1 to 3 are the compiler's own and hold neither.
            if p.number <= 3 || p.len % 2 != 0 {
                continue;
            }
            for i in (0..p.len).step_by(2) {
                let Some(value) = b.word(p.at + i) else {
                    continue;
                };
                let address = layout.unpack(value);
                if value <= layout.last_object || address < layout.high_memory {
                    continue;
                }
                let Some(first) = b.byte(address) else {
                    continue;
                };
                // A routine begins with its count of locals, 0 to 15.
                if first > 15 {
                    strings_from = Some(strings_from.map_or(value, |s| s.min(value)));
                }
                if (property::BEFORE..=property::LIFE).contains(&p.number) {
                    routines_to = Some(routines_to.map_or(value, |r| r.max(value)));
                }
            }
        }
    }
    (strings_from, routines_to)
}

/// What a two-byte property value is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Value {
    Zero,
    Object(u16),
    Routine,
    Text,
    NotDeterminable,
}

impl Inform {
    /// Classifies a two-byte property value (specification section 3.1).
    pub fn classify(&self, layout: &Layout, value: u16) -> Value {
        if value == 0 {
            return Value::Zero;
        }
        if value <= layout.last_object {
            return Value::Object(value);
        }
        match (self.routines_to, self.strings_from) {
            (Some(routines), Some(strings)) if strings > routines => {
                if value >= strings {
                    Value::Text
                } else if value <= routines {
                    Value::Routine
                } else {
                    Value::NotDeterminable
                }
            }
            _ => Value::NotDeterminable,
        }
    }

    /// What the game prints for an object: its `short_name` string if it has one.
    pub fn printed_name(&self, layout: &Layout, b: Bytes, number: u16) -> Part<String> {
        let Some(short) = layout
            .property(b, number, property::SHORT_NAME)
            .ok_or(Unknown::Damaged)?
        else {
            return layout.name(b, number).ok_or(Unknown::Damaged);
        };
        if short.len != 2 {
            return Err(Unknown::NotDeterminable);
        }
        let value = b.word(short.at).ok_or(Unknown::Damaged)?;
        match self.classify(layout, value) {
            Value::Text => layout.text(b, layout.unpack(value)).ok_or(Unknown::Damaged),
            Value::Zero => layout.name(b, number).ok_or(Unknown::Damaged),
            _ => Err(Unknown::NotDeterminable),
        }
    }

    /// The room the player is in, what holds the player within it, and whether
    /// the room is lit (specification section 6).
    pub fn place(&self, layout: &Layout, b: Bytes) -> Part<Place> {
        let mut at = self.player;
        let mut holder = None;
        // The chain of parents is no longer than the number of objects.
        for _ in 0..=layout.last_object {
            let parent = layout.object(b, at).ok_or(Unknown::Damaged)?.parent;
            if parent == 0 {
                break;
            }
            if at == self.player {
                holder = Some(parent);
            }
            at = parent;
        }
        if at == self.player {
            return Err(Unknown::NoPlayer);
        }
        if layout.object(b, at).ok_or(Unknown::Damaged)?.parent != 0 {
            return Err(Unknown::Damaged);
        }
        let room = at;
        // The library's own record of the location must agree, or be darkness.
        let variable = layout
            .global(b, self.location_global)
            .ok_or(Unknown::Damaged)?;
        if variable != room && variable != self.darkness {
            return Err(Unknown::Inconsistent);
        }
        // If no variable holds the player, the game has made another object the
        // player.
        if !(0..240).any(|i| layout.global(b, i) == Some(self.player)) {
            return Err(Unknown::Inconsistent);
        }
        Ok(Place {
            room,
            player: Ok(self.player),
            holder: Ok(holder.filter(|&h| h != room)),
            lit: Tri::from_bool(variable != self.darkness),
        })
    }
}

impl Inform {
    /// What a room gives for one direction (specification section 8.1).
    pub fn way(&self, layout: &Layout, b: Bytes, room: u16, direction: &Direction) -> Way {
        let exit = match layout.property(b, room, direction.property) {
            None => return Way::Unknown(Unknown::Damaged),
            Some(None) => return Way::None,
            Some(Some(exit)) => exit,
        };
        if exit.len != 2 {
            return Way::Unknown(Unknown::NotDeterminable);
        }
        let Some(value) = b.word(exit.at) else {
            return Way::Unknown(Unknown::Damaged);
        };
        match self.classify(layout, value) {
            Value::Zero | Value::Text => Way::None,
            Value::Routine => Way::Conditional,
            Value::NotDeterminable => Way::Unknown(Unknown::NotDeterminable),
            Value::Object(object) => {
                // A door first: one that is in several rooms has no parent either.
                if layout.has_attribute(b, object, attribute::DOOR) != Some(false) {
                    return Way::Conditional;
                }
                let parentless = layout
                    .object(b, object)
                    .is_some_and(|node| node.parent == 0);
                if parentless && self.is_room(layout, b, object) {
                    Way::Fixed(object)
                } else {
                    Way::Conditional
                }
            }
        }
    }

    /// A parentless object is a room if it is lit or has an exit of its own.
    fn is_room(&self, layout: &Layout, b: Bytes, object: u16) -> bool {
        layout.has_attribute(b, object, attribute::LIGHT) == Some(true)
            || layout.properties(b, object).is_some_and(|properties| {
                properties
                    .iter()
                    .any(|p| self.directions.iter().any(|d| d.property == p.number))
            })
    }
}

impl Inform {
    /// How the contents of an object are reached, if they can be seen at all:
    /// on a supporter, or inside something open or see-through (specification
    /// section 9.1).
    pub fn contents_reach(&self, layout: &Layout, b: Bytes, holder: u16) -> Option<crate::Reach> {
        let has = |attribute: u8| layout.has_attribute(b, holder, attribute) == Some(true);
        if has(attribute::SUPPORTER) {
            Some(crate::Reach::On)
        } else if has(attribute::OPEN) || has(attribute::TRANSPARENT) {
            Some(crate::Reach::Inside)
        } else {
            None
        }
    }

    /// Whether an unplaced object is present in a room by a plain `found_in`
    /// list: an even number of bytes, every word an object number, one of them
    /// the room. A routine there places nothing that can be read.
    pub fn found_in(&self, layout: &Layout, b: Bytes, object: u16, room: u16) -> bool {
        let Some(Some(list)) = layout.property(b, object, property::FOUND_IN) else {
            return false;
        };
        if list.len % 2 != 0 || layout.has_attribute(b, object, attribute::ABSENT) != Some(false) {
            return false;
        }
        let words: Option<Vec<u16>> = (0..list.len)
            .step_by(2)
            .map(|i| b.word(list.at + i))
            .collect();
        words.is_some_and(|words| {
            words.iter().all(|&w| w >= 1 && w <= layout.last_object) && words.contains(&room)
        })
    }
}
