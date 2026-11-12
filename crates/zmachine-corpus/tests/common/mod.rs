//! Shared by the corpus crate's tests: paths, loading, and small stories built
//! in memory whose exit tables and behaviour can be made to disagree on purpose
//! (specs/005-corpus-testing/tasks.md T004).
#![allow(dead_code)]

use zmachine_core::{LoadOptions, Machine, Settings};
use zmachine_testkit::op::{self, Arg::*};
use zmachine_testkit::{Object, Story, zencode};

pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// A path from the repository's root.
pub fn repo(path: &str) -> String {
    format!("{ROOT}/{path}")
}

pub fn read(path: &str) -> Vec<u8> {
    std::fs::read(repo(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

pub fn read_text(path: &str) -> String {
    String::from_utf8(read(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A machine as the reference transcripts were made with: a fixed seed, and an
/// interpreter that calls itself `F`.
pub fn load_bytes(story: Vec<u8>, seed: u32) -> Machine {
    let options = LoadOptions {
        seed: Some(seed),
        entropy: 0,
    };
    let settings = Settings {
        interpreter_version: b'F',
        ..Settings::default()
    };
    Machine::load_with(story, options, settings).expect("the story loads")
}

pub fn load(path: &str, seed: u32) -> Machine {
    load_bytes(read(path), seed)
}

// ---------------------------------------------------------------------------
// A toy game of the ZIL family's shape
// ---------------------------------------------------------------------------

/// The object every room is a child of.
pub const ROOMS: u8 = 1;
/// The first room's object number; room `i` (from 0) is object `FIRST + i`.
pub const FIRST: u8 = 2;

/// Direction properties, as feature 003's fixture numbers them.
pub const NORTH: u8 = 31;
pub const SOUTH: u8 = 30;
pub const EAST: u8 = 29;
pub const WEST: u8 = 28;
pub const UP: u8 = 23;
pub const DOWN: u8 = 22;
pub const DIRECTIONS: [u8; 6] = [NORTH, SOUTH, EAST, WEST, UP, DOWN];

/// What the toy's code does with a direction, whatever the room's table says.
pub const STAY: u8 = 0;
pub const QUIT: u8 = 250;
/// Asks a second question, at another address, then goes on.
pub const ASK: u8 = 251;
/// Loops for ever.
pub const SPIN: u8 = 252;
/// Performs an illegal operation.
pub const BOOM: u8 = 253;

fn column(direction: u8) -> usize {
    DIRECTIONS
        .iter()
        .position(|&d| d == direction)
        .expect("a direction of the toy")
}

/// A game of a few rooms. Each room has an exit table (what introspection
/// reads) and a row of behaviour (what the code does): a room's object number
/// to go there, or one of the constants above.
#[derive(Clone)]
pub struct Toy {
    tables: Vec<Vec<(u8, Vec<u8>)>>,
    behaviour: Vec<[u8; 6]>,
    /// The room the player starts in, counted from 0.
    pub start: usize,
    /// With `false` the dictionary has no direction words, so the story is of
    /// no family; the game plays the same.
    pub direction_words: bool,
}

impl Toy {
    /// `rooms` rooms with no exits and code that never moves the player.
    pub fn new(rooms: usize) -> Toy {
        Toy {
            tables: vec![Vec::new(); rooms],
            behaviour: vec![[STAY; 6]; rooms],
            start: 0,
            direction_words: true,
        }
    }

    pub fn room(index: usize) -> u16 {
        FIRST as u16 + index as u16
    }

    /// Gives room `from` an exit property with these bytes.
    pub fn table(&mut self, from: usize, direction: u8, data: Vec<u8>) -> &mut Self {
        self.tables[from].push((direction, data));
        self
    }

    /// Makes the code do `action` on `direction` in room `from`.
    pub fn does(&mut self, from: usize, direction: u8, action: u8) -> &mut Self {
        self.behaviour[from][column(direction)] = action;
        self
    }

    /// A one-byte exit in the table, and code that honours it.
    pub fn way(&mut self, from: usize, direction: u8, to: usize) -> &mut Self {
        let to = Self::room(to) as u8;
        self.table(from, direction, vec![to])
            .does(from, direction, to)
    }

    pub fn build(&self) -> Vec<u8> {
        let rooms = self.tables.len();
        let player = FIRST + rooms as u8;
        let mut story = Story::new();
        // The third data byte is the column of the behaviour table, plus one.
        let flags = if self.direction_words { 0x13 } else { 0x41 };
        for (word, direction) in [
            ("north", NORTH),
            ("n", NORTH),
            ("south", SOUTH),
            ("east", EAST),
            ("west", WEST),
            ("up", UP),
            ("down", DOWN),
        ] {
            story.word_data(word, [flags, direction, column(direction) as u8 + 1]);
        }
        story.word_data("look", [0x41, 0xF0, 0]);

        let mut container = Object::new("");
        container.child = FIRST;
        story.object(container);
        // The family is known by its rooms having direction properties. A toy with
        // none at all is given one that says "not an exit" (two bytes: a message).
        let mut tables = self.tables.clone();
        if tables.iter().all(Vec::is_empty) {
            tables[0].push((DOWN, vec![0, 0]));
        }
        for (i, table) in tables.iter().enumerate() {
            let mut room = Object::new(&format!("Room {i}"));
            room.parent = ROOMS;
            room.sibling = if i + 1 < rooms {
                FIRST + i as u8 + 1
            } else {
                0
            };
            room.props = table.clone();
            story.object(room);
        }
        story.object(Object::new("you"));

        let text = story.scratch(&{
            let mut b = vec![0u8; 40];
            b[0] = 30;
            b
        });
        let parse = story.scratch(&{
            let mut b = vec![0u8; 20];
            b[0] = 4;
            b
        });
        let flat: Vec<u8> = self.behaviour.iter().flatten().copied().collect();
        let table = story.scratch(&flat);

        // Variables: global 0 is the room; 12 the player, as ZIL's WINNER.
        const ROOM: u8 = 16;
        const WINNER: u8 = 16 + 12;
        const WORD: u8 = 16 + 20;
        const COLUMN: u8 = 16 + 21;
        const ACTION: u8 = 16 + 22;
        const SPARE: u8 = 16 + 23;
        let start = Self::room(self.start) as u8;
        let sread = || op::var(op::SREAD, &[L(text), L(parse)]);
        let print = |s: &str| [op::op0(op::PRINT), zencode(s)].concat();
        let mut a = Assembler::default();
        a.code(op::op2(op::STORE, S(ROOM), S(start)));
        a.code(op::op2(op::STORE, S(WINNER), S(player)));
        a.code(op::op2(op::INSERT_OBJ, S(player), S(start)));
        a.label("loop");
        a.code(print(">"));
        a.code(sread());
        a.code([op::var(op::RANDOM, &[S(100)]), op::store(SPARE)].concat());
        a.code([op::op2(op::LOADW, L(parse), S(1)), op::store(WORD)].concat());
        a.branch(op::op1(op::JZ, V(WORD)), true, "loop");
        a.code([op::op2(op::LOADB, V(WORD), S(6)), op::store(COLUMN)].concat());
        a.branch(op::op1(op::JZ, V(COLUMN)), true, "loop");
        a.code([op::op2(op::SUB, V(ROOM), S(FIRST)), op::store(ACTION)].concat());
        a.code([op::op2(op::MUL, V(ACTION), S(6)), op::store(ACTION)].concat());
        a.code([op::op2(op::ADD, V(ACTION), V(COLUMN)), op::store(ACTION)].concat());
        a.code([op::op2(op::SUB, V(ACTION), S(1)), op::store(ACTION)].concat());
        a.code([op::op2(op::LOADB, L(table), V(ACTION)), op::store(ACTION)].concat());
        a.branch(op::op1(op::JZ, V(ACTION)), true, "loop");
        a.branch(op::op2(op::JE, V(ACTION), S(QUIT)), true, "quit");
        a.branch(op::op2(op::JE, V(ACTION), S(ASK)), true, "ask");
        a.branch(op::op2(op::JE, V(ACTION), S(SPIN)), true, "spin");
        a.branch(op::op2(op::JE, V(ACTION), S(BOOM)), true, "boom");
        a.code(op::op2(op::STORE, S(ROOM), V(ACTION)));
        a.code(op::op2(op::INSERT_OBJ, S(player), V(ACTION)));
        a.jump("loop");
        a.label("quit");
        a.code(op::op0(op::QUIT));
        a.label("ask");
        a.code(print("?"));
        a.code(sread());
        a.jump("loop");
        a.label("spin");
        a.jump("spin");
        a.label("boom");
        a.code([op::op2(op::DIV, S(1), S(0)), op::store(SPARE)].concat());
        a.code(op::op0(op::QUIT));
        story.main(&a.finish());
        story.build()
    }

    pub fn machine(&self) -> Machine {
        load_bytes(self.build(), 1)
    }
}

/// Lays out code with branches and jumps to named places. Every branch is of
/// the two-byte form, so sizes do not depend on distances.
#[derive(Default)]
struct Assembler {
    items: Vec<Item>,
}

enum Item {
    Code(Vec<u8>),
    Label(&'static str),
    Branch(Vec<u8>, bool, &'static str),
    Jump(&'static str),
}

impl Assembler {
    fn code(&mut self, bytes: Vec<u8>) {
        self.items.push(Item::Code(bytes));
    }

    fn label(&mut self, name: &'static str) {
        self.items.push(Item::Label(name));
    }

    fn branch(&mut self, instruction: Vec<u8>, on_true: bool, to: &'static str) {
        self.items.push(Item::Branch(instruction, on_true, to));
    }

    fn jump(&mut self, to: &'static str) {
        self.items.push(Item::Jump(to));
    }

    fn finish(&self) -> Vec<u8> {
        let size = |item: &Item| match item {
            Item::Code(bytes) => bytes.len(),
            Item::Label(_) => 0,
            Item::Branch(instruction, _, _) => instruction.len() + 2,
            Item::Jump(_) => 3,
        };
        let mut labels = std::collections::HashMap::new();
        let mut at = 0usize;
        for item in &self.items {
            if let Item::Label(name) = item {
                labels.insert(*name, at);
            }
            at += size(item);
        }
        let mut out = Vec::new();
        for item in &self.items {
            let after = (out.len() + size(item)) as i32;
            let offset = |to: &str| (labels[to] as i32 - after + 2) as i16;
            match item {
                Item::Code(bytes) => out.extend(bytes),
                Item::Label(_) => {}
                Item::Branch(instruction, on_true, to) => {
                    out.extend(instruction);
                    out.extend(op::branch_long(*on_true, offset(to)));
                }
                Item::Jump(to) => out.extend(op::op1(op::JUMP, L(offset(to) as u16))),
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// The stories the repository holds a walkthrough for
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Game {
    pub name: &'static str,
    pub story: &'static str,
    pub walkthrough: &'static str,
    pub reference: &'static str,
    pub seed: u32,
}

/// As feature 003's tests have them (`crates/zmachine-introspect/tests/common`).
pub const GAMES: [Game; 3] = [
    Game {
        name: "zork1",
        story: "stories/zork1/zork1.z3",
        walkthrough: "stories/zork1/fidelity/walkthrough.txt",
        reference: "stories/zork1/fidelity/reference.scr",
        seed: 11,
    },
    Game {
        name: "advent.z5",
        story: "stories/advent/advent.z5",
        walkthrough: "stories/advent/fidelity/walkthrough.txt",
        reference: "stories/advent/fidelity/reference_z5.scr",
        seed: 1,
    },
    Game {
        name: "advent.z8",
        story: "stories/advent/advent.z8",
        walkthrough: "stories/advent/fidelity/walkthrough.txt",
        reference: "stories/advent/fidelity/reference_z8.scr",
        seed: 1,
    },
];
