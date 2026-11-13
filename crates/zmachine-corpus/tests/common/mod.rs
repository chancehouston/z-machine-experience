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
/// Goes through the toy's door if it is open, and otherwise stays.
const DOOR: u8 = 254;
/// The attribute the toy's `open` sets on its door and `close` clears.
pub const OPEN_ATTR: u8 = 9;
/// The attribute a toy's bag has when it is see-through.
pub const CLEAR_ATTR: u8 = 8;

/// The attribute a toy's lit rooms have, and its lamp while it is on.
pub const LIT_ATTR: u8 = 7;

/// What a toy's `look` command does, in a toy with a lamp. The toy's own rule
/// for light: the room has `LIT_ATTR`, or the lamp has it and is carried or in
/// the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Looking {
    /// Names the room when the toy's rule says it is lit, and not otherwise.
    Honest,
    /// Names the room whatever the light.
    AlwaysNames,
    /// Never names the room.
    NeverNames,
}

/// What a toy's `inventory` command prints of the gem in its bag, once the bag
/// and the key are taken. It always names the bag and the key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listing {
    /// As a game would: the gem while the bag is open or see-through.
    Honest,
    /// The gem, whatever the bag's state.
    Always,
    /// Never the gem.
    Never,
    /// Nothing that names anything carried.
    Nonsense,
}

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
    /// With `true` north has a further word, longer than `north`, that is no
    /// compass word: `sunward`.
    pub odd_north_word: bool,
    /// A door: the room it is in, the direction, and the room behind it. The
    /// table gets a 5-byte exit; the code goes through while the door has
    /// `OPEN_ATTR`, which the commands `open` and `close` set and clear.
    pub door: Option<(usize, u8, usize)>,
    /// Global variables that hold 1 from the start.
    pub set: Vec<u8>,
    /// Things to carry, in the first room: a bag with a gem in it, and a key.
    /// `take` takes the bag and the key; `open` and `close` set and clear
    /// `OPEN_ATTR` on the bag; `inventory` lists as this says.
    pub carrying: Option<Listing>,
    /// With `true` the bag has `CLEAR_ATTR` from the start.
    pub clear_bag: bool,
    /// A lamp in the first room, and a `look` command. `take` takes the lamp;
    /// `on` and `off` set and clear `LIT_ATTR` on it.
    pub lamp: Option<Looking>,
    /// Rooms, counted from 0, that have `LIT_ATTR`.
    pub lit_rooms: Vec<usize>,
}

impl Toy {
    /// `rooms` rooms with no exits and code that never moves the player.
    pub fn new(rooms: usize) -> Toy {
        Toy {
            tables: vec![Vec::new(); rooms],
            behaviour: vec![[STAY; 6]; rooms],
            start: 0,
            direction_words: true,
            odd_north_word: false,
            door: None,
            set: Vec::new(),
            carrying: None,
            clear_bag: false,
            lamp: None,
            lit_rooms: Vec::new(),
        }
    }

    /// The lamp's object number, in a toy that has one.
    pub fn lamp_object(&self) -> u16 {
        self.door_object()
    }

    /// The bag's, the gem's and the key's object numbers, in a toy that has
    /// things to carry and no door.
    pub fn things(&self) -> (u16, u16, u16) {
        let first = FIRST as u16 + self.tables.len() as u16 + 1;
        (first, first + 1, first + 2)
    }

    /// The door's object number.
    pub fn door_object(&self) -> u16 {
        FIRST as u16 + self.tables.len() as u16 + 1
    }

    /// A 4-byte exit on global variable `global`, which holds 1: the table says
    /// the way is open. The code goes there only if `honoured`.
    pub fn flag_way(
        &mut self,
        from: usize,
        direction: u8,
        to: usize,
        global: u8,
        honoured: bool,
    ) -> &mut Self {
        let room = Self::room(to) as u8;
        self.set.push(global);
        self.table(from, direction, vec![room, 16 + global, 0, 0])
            .does(from, direction, if honoured { room } else { STAY })
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
        if self.odd_north_word {
            story.word_data("sunward", [flags, NORTH, column(NORTH) as u8 + 1]);
        }
        let look = if self.lamp.is_some() { 13 } else { 0 };
        story.word_data("look", [0x41, 0xF0, look]);
        assert!(
            self.lamp.is_none() || (self.door.is_none() && self.carrying.is_none()),
            "a toy with a lamp has no door and nothing else to carry"
        );
        if self.lamp.is_some() {
            story.word_data("take", [0x41, 0xF3, 9]);
            story.word_data("on", [0x41, 0xF5, 11]);
            story.word_data("off", [0x41, 0xF6, 12]);
        }
        assert!(
            self.door.is_none() || self.carrying.is_none(),
            "a toy has a door or things to carry, not both"
        );
        if self.door.is_some() || self.carrying.is_some() {
            story.word_data("open", [0x41, 0xF1, 7]);
            story.word_data("close", [0x41, 0xF2, 8]);
        }
        if self.carrying.is_some() {
            story.word_data("take", [0x41, 0xF3, 9]);
            story.word_data("inventory", [0x41, 0xF4, 10]);
        }
        for &global in &self.set {
            story.global(global as usize, 1);
        }

        let mut container = Object::new("");
        container.child = FIRST;
        story.object(container);
        // The family is known by its rooms having direction properties. A toy with
        // none at all is given one that says "not an exit" (two bytes: a message).
        let mut tables = self.tables.clone();
        let mut behaviour = self.behaviour.clone();
        let door = self.door_object() as u8;
        let mut behind = 0;
        if let Some((from, direction, to)) = self.door {
            behind = Self::room(to) as u8;
            tables[from].push((direction, vec![behind, door, 0, 0, 0]));
            behaviour[from][column(direction)] = DOOR;
        }
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
            if i == 0 && (self.carrying.is_some() || self.lamp.is_some()) {
                room.child = door;
            }
            if self.lit_rooms.contains(&i) {
                room.attributes |= 0x8000_0000 >> LIT_ATTR;
            }
            story.object(room);
        }
        story.object(Object::new("you"));
        if self.door.is_some() {
            story.object(Object::new("door"));
        }
        if self.lamp.is_some() {
            let mut thing = Object::new("lamp");
            thing.parent = FIRST;
            story.object(thing);
        }
        // The bag takes the door's number: `open` and `close` act on it.
        let (bag, gem, key) = (door, door + 1, door + 2);
        if self.carrying.is_some() {
            let mut thing = Object::new("bag");
            thing.parent = FIRST;
            thing.child = gem;
            thing.sibling = key;
            if self.clear_bag {
                thing.attributes |= 0x8000_0000 >> CLEAR_ATTR;
            }
            story.object(thing);
            let mut thing = Object::new("gem");
            thing.parent = bag;
            story.object(thing);
            let mut thing = Object::new("key");
            thing.parent = FIRST;
            story.object(thing);
        }

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
        let flat: Vec<u8> = behaviour.iter().flatten().copied().collect();
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
        if self.door.is_some() || self.carrying.is_some() {
            a.branch(op::op2(op::JE, V(COLUMN), S(7)), true, "open");
            a.branch(op::op2(op::JE, V(COLUMN), S(8)), true, "close");
        }
        if self.carrying.is_some() {
            a.branch(op::op2(op::JE, V(COLUMN), S(9)), true, "take");
            a.branch(op::op2(op::JE, V(COLUMN), S(10)), true, "inventory");
        }
        if self.lamp.is_some() {
            a.branch(op::op2(op::JE, V(COLUMN), S(9)), true, "take-lamp");
            a.branch(op::op2(op::JE, V(COLUMN), S(11)), true, "on");
            a.branch(op::op2(op::JE, V(COLUMN), S(12)), true, "off");
            a.branch(op::op2(op::JE, V(COLUMN), S(13)), true, "look");
        }
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
        if self.door.is_some() {
            a.branch(op::op2(op::JE, V(ACTION), S(DOOR)), true, "door");
        }
        a.code(op::op2(op::STORE, S(ROOM), V(ACTION)));
        a.code(op::op2(op::INSERT_OBJ, S(player), V(ACTION)));
        a.jump("loop");
        if let Some(looking) = self.lamp {
            let lamp = door;
            a.label("take-lamp");
            a.code(op::op2(op::INSERT_OBJ, S(lamp), S(player)));
            a.jump("loop");
            a.label("on");
            a.code(op::op2(op::SET_ATTR, S(lamp), S(LIT_ATTR)));
            a.jump("loop");
            a.label("off");
            a.code(op::op2(op::CLEAR_ATTR, S(lamp), S(LIT_ATTR)));
            a.jump("loop");
            a.label("look");
            match looking {
                Looking::AlwaysNames => a.jump("named"),
                Looking::NeverNames => a.jump("unnamed"),
                Looking::Honest => {
                    a.branch(op::op2(op::TEST_ATTR, V(ROOM), S(LIT_ATTR)), true, "named");
                    a.branch(
                        op::op2(op::TEST_ATTR, S(lamp), S(LIT_ATTR)),
                        false,
                        "unnamed",
                    );
                    a.branch(op::op2(op::JIN, S(lamp), S(player)), true, "named");
                    a.branch(op::op2(op::JIN, S(lamp), V(ROOM)), true, "named");
                    a.jump("unnamed");
                }
            }
            a.label("named");
            a.code(op::op1(op::PRINT_OBJ, V(ROOM)));
            a.jump("loop");
            a.label("unnamed");
            a.code(print("pitch "));
            a.jump("loop");
        }
        if let Some(listing) = self.carrying {
            a.label("take");
            a.code(op::op2(op::INSERT_OBJ, S(key), S(player)));
            a.code(op::op2(op::INSERT_OBJ, S(bag), S(player)));
            a.jump("loop");
            a.label("inventory");
            // Nothing is listed until the bag is carried.
            a.branch(op::op2(op::JIN, S(bag), S(player)), false, "loop");
            if listing == Listing::Nonsense {
                a.code(print("eh "));
                a.jump("loop");
            }
            a.code(print("bag key "));
            match listing {
                Listing::Always => a.jump("gem"),
                Listing::Never | Listing::Nonsense => a.jump("loop"),
                Listing::Honest => {
                    a.branch(op::op2(op::TEST_ATTR, S(bag), S(OPEN_ATTR)), true, "gem");
                    a.branch(op::op2(op::TEST_ATTR, S(bag), S(CLEAR_ATTR)), true, "gem");
                    a.jump("loop");
                }
            }
            a.label("gem");
            a.code(print("gem "));
            a.jump("loop");
        }
        if self.door.is_some() || self.carrying.is_some() {
            a.label("open");
            a.code(op::op2(op::SET_ATTR, S(door), S(OPEN_ATTR)));
            a.jump("loop");
            a.label("close");
            a.code(op::op2(op::CLEAR_ATTR, S(door), S(OPEN_ATTR)));
            a.jump("loop");
        }
        if self.door.is_some() {
            a.label("door");
            a.branch(op::op2(op::TEST_ATTR, S(door), S(OPEN_ATTR)), false, "loop");
            a.code(op::op2(op::STORE, S(ROOM), S(behind)));
            a.code(op::op2(op::INSERT_OBJ, S(player), S(behind)));
            a.jump("loop");
        }
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

// ---------------------------------------------------------------------------
// A folder laid out like the repository, for tests of the commands
// ---------------------------------------------------------------------------

/// A new, empty folder under the system's temporary folder, removed when the
/// value is dropped.
pub struct Scratch(pub std::path::PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Scratch {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("zmachine-corpus-{}-{name}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("stories/corpus/reports")).unwrap();
        std::fs::write(path.join("stories/corpus/manifest.txt"), "").unwrap();
        std::fs::write(path.join("stories/corpus/ledger.txt"), "").unwrap();
        Scratch(path)
    }

    pub fn path(&self, relative: &str) -> std::path::PathBuf {
        self.0.join(relative)
    }

    pub fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    /// Writes `packs/<id>.pack` for an image, with its identity worked out from
    /// the image and these entries.
    pub fn pack(&self, id: &str, image: &[u8], entries: &str) {
        self.write(
            &format!("packs/{id}.pack"),
            pack_for(image, entries).as_bytes(),
        );
    }

    pub fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.path(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
    }

    /// Adds a story built in memory as a committed story with a license record
    /// beside it, and its entry to the manifest. `extra` is further lines of the
    /// entry, each ending in a line feed.
    pub fn add(&self, id: &str, image: &[u8], extra: &str) {
        use sha2::{Digest, Sha256};
        let digest: String = Sha256::digest(image)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        self.write(&format!("stories/{id}/{id}.z3"), image);
        self.write(&format!("stories/{id}/LICENSE"), b"A test's own.\n");
        self.write(
            &format!("stories/{id}/SOURCE.md"),
            format!("| SHA-256 | `{digest}` |\n").as_bytes(),
        );
        let entry = format!(
            "id: {id}\nsha256: {digest}\nsize: {}\nversion: 3\nfamily: zil\nsource: built in memory\n\
             standing: a test's own\ntier: committed\npath: stories/{id}/{id}.z3\nseed: 1\n{extra}\n",
            image.len()
        );
        let manifest = self.read("stories/corpus/manifest.txt") + &entry;
        self.write("stories/corpus/manifest.txt", manifest.as_bytes());
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A game pack for an image: its identity, then these entries.
pub fn pack_for(image: &[u8], entries: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(image)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!(
        "pack: 1\nsha256: {digest}\nrelease: {}\nserial: {}\n{entries}",
        u16::from_be_bytes([image[2], image[3]]),
        String::from_utf8_lossy(&image[0x12..0x18])
    )
}
