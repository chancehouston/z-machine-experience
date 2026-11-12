//! Small story files of each family, built for the unit tests. Each is laid out
//! as its compiler lays one out, as far as the specification's rules look.

#![allow(dead_code)]

use zmachine_core::{Event, LoadOptions, Machine};
use zmachine_testkit::op::{self, Arg};
use zmachine_testkit::{Object, Object5, Story, Story5};

pub const OPTIONS: LoadOptions = LoadOptions {
    seed: Some(1),
    entropy: 0,
};

/// Runs an image to its end, so that whatever its start-up code set is in memory.
pub fn run(image: Vec<u8>) -> Machine {
    let mut m = Machine::load(image, OPTIONS).expect("the fixture loads");
    loop {
        match m.run(100_000).expect("the fixture runs") {
            Event::Finished => return m,
            Event::BudgetExhausted => continue,
            other => panic!("the fixture stopped with {other:?}"),
        }
    }
}

/// `store` of a value into global `index`.
pub fn set_global(index: u8, value: u16) -> Vec<u8> {
    op::op2(op::STORE, Arg::S(16 + index), Arg::L(value))
}

pub fn insert(object: u16, into: u16) -> Vec<u8> {
    op::op2(op::INSERT_OBJ, Arg::L(object), Arg::L(into))
}

pub fn remove(object: u16) -> Vec<u8> {
    op::op1(op::REMOVE_OBJ, Arg::L(object))
}

pub fn set_attr(object: u16, attribute: u8) -> Vec<u8> {
    op::op2(op::SET_ATTR, Arg::L(object), Arg::S(attribute))
}

pub fn clear_attr(object: u16, attribute: u8) -> Vec<u8> {
    op::op2(op::CLEAR_ATTR, Arg::L(object), Arg::S(attribute))
}

// ---------------------------------------------------------------------------
// Inform 6
// ---------------------------------------------------------------------------

/// A property value that is only known once the story is laid out.
#[derive(Clone, Debug)]
pub enum Late {
    /// The packed address of a routine with this many locals.
    Routine(usize),
    /// The packed address of this string.
    Text(&'static str),
    /// The dictionary addresses of these words, one after another.
    Words(Vec<&'static str>),
}

/// Object numbers of the Inform fixture.
pub mod inf {
    pub const COMPASS: u16 = 5;
    pub const N: u16 = 6;
    pub const S: u16 = 7;
    pub const IN: u16 = 8;
    pub const OUT: u16 = 9;
    pub const DARKNESS: u16 = 10;
    pub const PLAYER: u16 = 11;
    pub const LIBRARY: u16 = 12;
    pub const KITCHEN: u16 = 13;
    pub const HALL: u16 = 14;
    pub const CAGE: u16 = 15;
    pub const LAMP: u16 = 16;
    pub const BOX: u16 = 17;
    pub const COIN: u16 = 18;
    pub const TABLE: u16 = 19;
    pub const PLATE: u16 = 20;
    pub const DOOR: u16 = 21;
    pub const STREAM: u16 = 22;
    pub const MAZE: u16 = 23;
    pub const HAT: u16 = 24;
    /// The library's `location` variable in this fixture.
    pub const LOCATION: u8 = 1;
    /// The variable that holds the player, as the library's `player` does.
    pub const PLAYER_VARIABLE: u8 = 9;
}

/// What an Inform fixture is built from. Tests change it before building.
pub struct Inform {
    pub version: u8,
    pub stamp: [u8; 4],
    pub objects: Vec<Object5>,
    pub globals: Vec<(usize, u16)>,
    pub words: Vec<(&'static str, [u8; 3])>,
    /// (object number, property, value): filled in at layout.
    pub late: Vec<(u16, u8, Late)>,
    /// Code run before the fixture stops: the game's "start-up".
    pub setup: Vec<u8>,
}

fn attributes(numbers: &[u8]) -> u64 {
    numbers.iter().fold(0, |bits, &n| bits | 1 << (47 - n))
}

fn object5(name: &str, parent: u16, attrs: &[u8], props: Vec<(u8, Vec<u8>)>) -> Object5 {
    let mut o = Object5::new(name);
    o.parent = parent;
    o.attributes = attributes(attrs);
    o.props = props;
    o
}

fn word(value: u16) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

impl Inform {
    /// A file that satisfies every rule of specification section 3.1: the four
    /// metaclasses, a compass with four directions, the library's objects, two
    /// rooms and some things. Start-up puts the player in the kitchen.
    pub fn new() -> Inform {
        use inf::*;
        let direction = |name: &str, property: u8| {
            object5(
                name,
                COMPASS,
                &[17],
                vec![(21, word(property as u16)), (2, word(1)), (3, word(0))],
            )
        };
        let objects = vec![
            object5("Class", 0, &[], vec![]),
            object5("Object", 0, &[], vec![]),
            object5("Routine", 0, &[], vec![]),
            object5("String", 0, &[], vec![]),
            object5("compass", 0, &[3], vec![]),
            direction("(n_obj)", 7),
            direction("(s_obj)", 8),
            direction("(in_obj)", 17),
            direction("(out_obj)", 18),
            object5("(darkness object)", 0, &[], vec![]),
            object5("(self object)", 0, &[0, 3, 16, 23], vec![(2, word(1))]),
            object5("(Inform Library)", 0, &[16], vec![]),
            // Kitchen: north to the hall, south refused by a string (late), in by
            // a routine (late), out to the door.
            object5(
                "Kitchen",
                0,
                &[9],
                vec![(7, word(HALL)), (18, word(DOOR)), (2, word(1))],
            ),
            object5("Hall", 0, &[9], vec![(8, word(KITCHEN))]),
            // A cage in the kitchen: a container, closed and opaque.
            object5("cage", KITCHEN, &[4, 15], vec![]),
            object5("lamp", KITCHEN, &[], vec![]),
            // A box the player will carry, open, with a coin in it.
            object5("box", KITCHEN, &[4, 14], vec![]),
            object5("coin", BOX, &[], vec![]),
            // A table (supporter) with a plate on it.
            object5("table", KITCHEN, &[20, 17], vec![]),
            object5("plate", TABLE, &[3], vec![]),
            // A door between the rooms: parentless, with the door attribute.
            object5(
                "door",
                0,
                &[5, 17],
                vec![(39, [word(KITCHEN), word(HALL)].concat())],
            ),
            // A stream present in the kitchen and the maze by `found_in`.
            object5(
                "stream",
                0,
                &[17],
                vec![(39, [word(KITCHEN), word(MAZE)].concat())],
            ),
            // A room whose printed name is its short_name (late).
            object5("(Maze_1)", 0, &[9], vec![(7, word(MAZE))]),
            object5("hat", KITCHEN, &[2], vec![]),
        ];
        let mut setup = Vec::new();
        setup.extend(set_global(LOCATION, KITCHEN));
        setup.extend(set_global(PLAYER_VARIABLE, PLAYER));
        setup.extend(insert(PLAYER, KITCHEN));
        Inform {
            version: 5,
            stamp: *b"6.44",
            objects,
            globals: vec![(LOCATION as usize, LIBRARY)],
            words: vec![
                ("n", [0x80, 0, 0]),
                ("north", [0x80, 0, 0]),
                ("s", [0x80, 0, 0]),
                ("south", [0x80, 0, 0]),
                ("in", [0x41, 0xFF, 0]),
                ("out", [0x41, 0xFE, 0]),
                ("take", [0x41, 0xFD, 0]),
                ("lamp", [0x80, 0, 0]),
                ("to", [0x08, 0, 0]),
                ("all", [0x8C, 0, 0]),
            ],
            late: vec![
                (N, 1, Late::Words(vec!["n", "north"])),
                (S, 1, Late::Words(vec!["s", "south"])),
                (KITCHEN, 8, Late::Text("You can't go that way.")),
                (KITCHEN, 17, Late::Routine(1)),
                (KITCHEN, 4, Late::Routine(0)),
                (HALL, 4, Late::Routine(2)),
                (MAZE, 46, Late::Text("Maze")),
            ],
            setup,
        }
    }

    pub fn object(&mut self, number: u16) -> &mut Object5 {
        &mut self.objects[number as usize - 1]
    }

    /// Sets or replaces a property with plain bytes.
    pub fn prop(&mut self, number: u16, property: u8, data: Vec<u8>) -> &mut Self {
        let props = &mut self.object(number).props;
        props.retain(|(p, _)| *p != property);
        props.push((property, data));
        self
    }

    pub fn without_prop(&mut self, number: u16, property: u8) -> &mut Self {
        self.object(number).props.retain(|(p, _)| *p != property);
        self.late
            .retain(|(n, p, _)| !(*n == number && *p == property));
        self
    }

    pub fn attrs(&mut self, number: u16, attrs: &[u8]) -> &mut Self {
        self.object(number).attributes = attributes(attrs);
        self
    }

    pub fn late(&mut self, number: u16, property: u8, value: Late) -> &mut Self {
        self.without_prop(number, property);
        self.late.push((number, property, value));
        self
    }

    /// Lays the story out twice: once to learn where things fall, once for real.
    pub fn build(&self) -> Vec<u8> {
        let placeholder = |late: &Late| match late {
            Late::Words(words) => vec![0; 2 * words.len()],
            _ => vec![0; 2],
        };
        let lay = |values: &[Vec<u8>]| {
            let mut story = Story5::new(self.version);
            story.header_word(0x3C, u16::from_be_bytes([self.stamp[0], self.stamp[1]]));
            story.header_word(0x3E, u16::from_be_bytes([self.stamp[2], self.stamp[3]]));
            for &(index, value) in &self.globals {
                story.global(index, value);
            }
            for (word, data) in &self.words {
                story.word_data(word, *data);
            }
            let mut objects = self.objects.clone();
            for ((number, property, _), value) in self.late.iter().zip(values) {
                let props = &mut objects[*number as usize - 1].props;
                props.retain(|(p, _)| p != property);
                props.push((*property, value.clone()));
            }
            // Children follow from parents, in order of number.
            for i in 0..objects.len() {
                let parent = objects[i].parent;
                if parent == 0 {
                    continue;
                }
                let me = i as u16 + 1;
                let siblings: Vec<u16> = (0..objects.len())
                    .filter(|&j| objects[j].parent == parent)
                    .map(|j| j as u16 + 1)
                    .collect();
                let position = siblings.iter().position(|&s| s == me).unwrap();
                objects[i].sibling = siblings.get(position + 1).copied().unwrap_or(0);
                if position == 0 {
                    objects[parent as usize - 1].child = me;
                }
            }
            for o in objects {
                story.object(o);
            }
            // The game's own text and code first, as a compiler lays them: main
            // code, then routines, then strings.
            let mut code = self.setup.clone();
            code.extend(op::op0(op::QUIT));
            story.main(&code);
            let mut resolved: Vec<Vec<u8>> = vec![Vec::new(); self.late.len()];
            for (i, (_, _, late)) in self.late.iter().enumerate() {
                if let Late::Routine(locals) = late {
                    resolved[i] = word(story.routine(*locals, &op::op0(op::RTRUE)));
                }
            }
            for (i, (_, _, late)) in self.late.iter().enumerate() {
                match late {
                    Late::Text(text) => resolved[i] = word(story.string(text)),
                    Late::Words(words) => {
                        resolved[i] = words
                            .iter()
                            .flat_map(|w| word(story.word_address(w)))
                            .collect();
                    }
                    Late::Routine(_) => {}
                }
            }
            (story.build(), resolved)
        };
        let placeholders: Vec<Vec<u8>> = self
            .late
            .iter()
            .map(|(_, _, late)| placeholder(late))
            .collect();
        let (_, resolved) = lay(&placeholders);
        lay(&resolved).0
    }

    pub fn machine(&self) -> Machine {
        run(self.build())
    }
}

// ---------------------------------------------------------------------------
// ZIL
// ---------------------------------------------------------------------------

/// Object numbers and direction properties of the ZIL fixture.
pub mod zl {
    pub const ROOMS: u16 = 1;
    pub const KITCHEN: u16 = 2;
    pub const HALL: u16 = 3;
    pub const CELLAR: u16 = 4;
    pub const PLAYER: u16 = 5;
    pub const LAMP: u16 = 6;
    pub const SACK: u16 = 7;
    pub const GARLIC: u16 = 8;
    pub const DIAMOND: u16 = 9;
    pub const BOAT: u16 = 10;
    pub const DOOR: u16 = 11;
    pub const NORTH: u8 = 31;
    pub const SOUTH: u8 = 30;
    pub const EAST: u8 = 29;
    pub const WEST: u8 = 28;
    pub const DOWN: u8 = 22;
    pub const UP: u8 = 23;
    /// The variable the fixture keeps the player in, as ZIL's `WINNER`.
    pub const WINNER: u8 = 12;
    /// A flag variable that opens the kitchen's way down.
    pub const TRAP_OPEN: u8 = 20;
}

pub struct Zil {
    pub version_stamp: [u8; 4],
    pub objects: Vec<Object>,
    pub words: Vec<(&'static str, [u8; 3])>,
    pub globals: Vec<(usize, u16)>,
    pub setup: Vec<u8>,
}

fn object3(name: &str, parent: u8, props: Vec<(u8, Vec<u8>)>) -> Object {
    let mut o = Object::new(name);
    o.parent = parent;
    o.props = props;
    o
}

impl Zil {
    /// A file that satisfies every rule of specification section 3.2. Start-up
    /// puts the player, holding the lamp, in the kitchen.
    pub fn new() -> Zil {
        use zl::*;
        let (kitchen, hall, cellar) = (KITCHEN as u8, HALL as u8, CELLAR as u8);
        let objects = vec![
            // ROOMS carries a direction property itself, as the real thing does.
            object3("", 0, vec![(21, vec![ROOMS as u8])]),
            object3(
                "Kitchen",
                ROOMS as u8,
                vec![
                    (NORTH, vec![hall]),                              // UEXIT
                    (SOUTH, vec![0x12, 0x34]),                        // NEXIT: a message
                    (EAST, vec![0x23, 0x45, 0]),                      // FEXIT: a routine
                    (DOWN, vec![cellar, 16 + TRAP_OPEN, 0x12, 0x34]), // CEXIT: a flag
                    (WEST, vec![hall, DOOR as u8, 0x12, 0x34, 0]),    // DEXIT: a door
                ],
            ),
            object3(
                "Hall",
                ROOMS as u8,
                vec![(SOUTH, vec![kitchen]), (NORTH, vec![hall])],
            ),
            object3("Cellar", ROOMS as u8, vec![(UP, vec![kitchen, 5, 0, 0])]),
            // The player, and two more things that start with no parent.
            object3("you", 0, vec![]),
            object3("lamp", kitchen, vec![]),
            object3("sack", kitchen, vec![]),
            object3("garlic", SACK as u8, vec![]),
            object3("diamond", 0, vec![]),
            object3("boat", hall, vec![]),
            object3("door", kitchen, vec![]),
        ];
        let mut setup = Vec::new();
        setup.extend(set_global(0, KITCHEN));
        setup.extend(set_global(WINNER, PLAYER));
        setup.extend(insert(PLAYER, KITCHEN));
        setup.extend(insert(LAMP, PLAYER));
        Zil {
            version_stamp: [0; 4],
            objects,
            words: vec![
                ("north", [0x13, NORTH, 0]),
                ("n", [0x13, NORTH, 0]),
                ("south", [0x13, SOUTH, 0]),
                ("east", [0x13, EAST, 0]),
                ("west", [0x33, WEST, 0xA1]),
                ("down", [0x18, 0xFA, DOWN]),
                ("up", [0x18, 0xFC, UP]),
                ("take", [0x41, 0xF0, 0]),
                ("lamp", [0x80, 1, 0]),
                ("with", [0x08, 0xEE, 0]),
                ("the", [0x04, 0, 0]),
                ("brass", [0x22, 3, 0]),
            ],
            globals: vec![],
            setup,
        }
    }

    pub fn object(&mut self, number: u16) -> &mut Object {
        &mut self.objects[number as usize - 1]
    }

    pub fn prop(&mut self, number: u16, property: u8, data: Vec<u8>) -> &mut Self {
        let props = &mut self.object(number).props;
        props.retain(|(p, _)| *p != property);
        props.push((property, data));
        self
    }

    pub fn build(&self) -> Vec<u8> {
        let mut story = Story::new();
        for (i, byte) in self.version_stamp.iter().enumerate() {
            story.header_byte(0x3C + i, *byte);
        }
        for &(index, value) in &self.globals {
            story.global(index, value);
        }
        for (word, data) in &self.words {
            story.word_data(word, *data);
        }
        let mut objects = self.objects.clone();
        for i in 0..objects.len() {
            let parent = objects[i].parent;
            if parent == 0 {
                continue;
            }
            let me = i as u8 + 1;
            let siblings: Vec<u8> = (0..objects.len())
                .filter(|&j| objects[j].parent == parent)
                .map(|j| j as u8 + 1)
                .collect();
            let position = siblings.iter().position(|&s| s == me).unwrap();
            objects[i].sibling = siblings.get(position + 1).copied().unwrap_or(0);
            if position == 0 {
                objects[parent as usize - 1].child = me;
            }
        }
        for o in objects {
            story.object(o);
        }
        let mut code = self.setup.clone();
        code.extend(op::op0(op::QUIT));
        story.main(&code);
        story.build()
    }

    pub fn machine(&self) -> Machine {
        run(self.build())
    }
}
