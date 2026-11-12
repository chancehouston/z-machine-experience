//! The adapter for Infocom's ZIL (specification section 3.2).

use crate::family::{Place, Way, has_inform_stamp};
use crate::table::{Bytes, Layout};
use crate::{Part, Tri, Unknown};

/// The first data byte of a dictionary entry: the parts of speech.
pub(crate) mod flag {
    pub const NOUN: u8 = 0x80;
    pub const VERB: u8 = 0x40;
    pub const ADJECTIVE: u8 = 0x20;
    pub const DIRECTION: u8 = 0x10;
    pub const PREPOSITION: u8 = 0x08;
    pub const BUZZ: u8 = 0x04;
}

/// One direction a player can go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Direction {
    /// The property a room gives its exit in.
    pub property: u8,
    /// The dictionary words that name it, shortest first.
    pub words: Vec<String>,
}

/// What recognition found in the story file as shipped.
#[derive(Clone, Debug)]
pub(crate) struct Zil {
    /// The object every room is a child of.
    pub rooms: u16,
    pub directions: Vec<Direction>,
    /// Objects with no parent and no children as shipped: the player is one.
    pub unplaced: Vec<u16>,
}

/// The direction property a dictionary entry names, if it is a direction word:
/// the second data byte when the low two bits of the first are 3, else the third.
pub(crate) fn direction_of(data: [u8; 3]) -> Option<u8> {
    (data[0] & flag::DIRECTION != 0).then(|| if data[0] & 3 == 3 { data[1] } else { data[2] })
}

impl Zil {
    /// Applies the rules of specification section 3.2 to the file as shipped.
    pub fn recognise(layout: &Layout, b: Bytes) -> Option<Zil> {
        // Rule 1: version 3, and not compiled by Inform.
        if layout.version != 3 || has_inform_stamp(b) {
            return None;
        }
        // Rule 2: 7-byte dictionary entries.
        let d = layout.dictionary(b)?;
        if d.entry_len != 7 {
            return None;
        }
        // Rule 3: at least four direction words.
        let mut directions: Vec<Direction> = Vec::new();
        let mut count = 0;
        for i in 0..d.count {
            let entry = d.first + i * d.entry_len;
            let data = [b.byte(entry + 4)?, b.byte(entry + 5)?, b.byte(entry + 6)?];
            let Some(property) = direction_of(data) else {
                continue;
            };
            let word = layout.word(b, entry)?;
            count += 1;
            match directions.iter_mut().find(|d| d.property == property) {
                Some(direction) => direction.words.push(word),
                None => directions.push(Direction {
                    property,
                    words: vec![word],
                }),
            }
        }
        if count < 4 {
            return None;
        }
        for direction in &mut directions {
            direction
                .words
                .sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
        }
        // In the order of a property table: highest number first.
        directions.sort_by_key(|d| std::cmp::Reverse(d.property));
        let is_direction = |property: u8| directions.iter().any(|d| d.property == property);
        // Rule 4: every object with a direction property, parentless ones aside,
        // has one and the same parent, which itself has none.
        let mut rooms = None;
        let mut with_exits = Vec::new();
        for number in 1..=layout.last_object {
            let node = layout.object(b, number)?;
            let properties = layout.properties(b, number)?;
            if node.parent == 0 || !properties.iter().any(|p| is_direction(p.number)) {
                continue;
            }
            if *rooms.get_or_insert(node.parent) != node.parent {
                return None;
            }
            with_exits.push((number, properties));
        }
        let rooms = rooms?;
        if layout.object(b, rooms)?.parent != 0 {
            return None;
        }
        // Rule 5: each exit is 1 to 5 bytes, and a 1-byte exit names a room.
        for (_, properties) in &with_exits {
            for p in properties.iter().filter(|p| is_direction(p.number)) {
                if !(1..=5).contains(&p.len) {
                    return None;
                }
                if p.len == 1 && layout.object(b, b.byte(p.at)? as u16)?.parent != rooms {
                    return None;
                }
            }
        }
        let unplaced = (1..=layout.last_object)
            .filter(|&n| {
                layout
                    .object(b, n)
                    .is_some_and(|node| node.parent == 0 && node.child == 0)
            })
            .collect();
        Some(Zil {
            rooms,
            directions,
            unplaced,
        })
    }
}

impl Zil {
    /// The room the player is in: the object in global 0, which must be a room
    /// (specification section 6).
    pub fn place(&self, layout: &Layout, b: Bytes) -> Part<Place> {
        let room = layout.global(b, 0).ok_or(Unknown::Damaged)?;
        match layout.object(b, room) {
            Some(node) if node.parent == self.rooms => {}
            _ => return Err(Unknown::NoPlayer),
        }
        let player = self.player(layout, b, room);
        let holder = player.and_then(|player| {
            let parent = layout.object(b, player).ok_or(Unknown::Damaged)?.parent;
            Ok((parent != room).then_some(parent))
        });
        Ok(Place {
            room,
            player,
            holder,
            lit: Tri::Unknown,
        })
    }

    /// The player: among the objects unplaced as shipped, those now inside the
    /// room and not inside another of them; of those, the one whose number some
    /// variable other than global 0 holds. A tie is not broken (specification
    /// section 7).
    fn player(&self, layout: &Layout, b: Bytes, room: u16) -> Part<u16> {
        match self.contenders(layout, b, room)?.as_slice() {
            [player] => Ok(*player),
            [] => Err(Unknown::NoPlayer),
            _ => Err(Unknown::AmbiguousPlayer),
        }
    }

    /// The objects that fit the rule for the player. More than one is a tie.
    pub fn contenders(&self, layout: &Layout, b: Bytes, room: u16) -> Part<Vec<u16>> {
        let mut candidates = Vec::new();
        'candidate: for &object in &self.unplaced {
            let mut at = object;
            for _ in 0..=layout.last_object {
                let parent = layout.object(b, at).ok_or(Unknown::Damaged)?.parent;
                if parent == room {
                    candidates.push(object);
                    continue 'candidate;
                }
                // Not placed, or inside another candidate.
                if parent == 0 || self.unplaced.contains(&parent) {
                    continue 'candidate;
                }
                at = parent;
            }
        }
        candidates.retain(|&object| (1..240).any(|i| layout.global(b, i) == Some(object)));
        Ok(candidates)
    }
}

impl Zil {
    /// What a room gives for one direction, by the length of the property
    /// (specification section 8.2).
    pub fn way(&self, layout: &Layout, b: Bytes, room: u16, direction: &Direction) -> Way {
        let exit = match layout.property(b, room, direction.property) {
            None => return Way::Unknown(Unknown::Damaged),
            Some(None) => return Way::None,
            Some(Some(exit)) => exit,
        };
        let Some(data) = layout.data(b, &exit) else {
            return Way::Unknown(Unknown::Damaged);
        };
        let to = |destination: u8| match layout.object(b, destination as u16) {
            Some(node) if node.parent == self.rooms => Way::Fixed(destination as u16),
            _ => Way::Unknown(Unknown::NotDeterminable),
        };
        match data.as_slice() {
            // To a room.
            [destination] => to(*destination),
            // A message of refusal.
            [_, _] => Way::None,
            // A routine decides; so does a door.
            [_, _, _] | [_, _, _, _, _] => Way::Conditional,
            // To a room while a global variable is not zero. The byte is a
            // Z-machine variable number: 16 is the first global.
            [destination, variable, _, _] => match variable.checked_sub(16) {
                None => Way::Conditional,
                Some(index) => match layout.global(b, index as usize) {
                    Some(0) => Way::None,
                    Some(_) => to(*destination),
                    None => Way::Unknown(Unknown::Damaged),
                },
            },
            _ => Way::Unknown(Unknown::Damaged),
        }
    }
}
