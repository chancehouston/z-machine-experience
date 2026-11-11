//! Assembles a complete version 3 story image.
//!
//! Layout, low to high:
//!
//! ```text
//! 0x0000  header (64 bytes)
//! 0x0040  abbreviations table (96 words)
//! 0x0100  global variables (240 words)            GLOBALS
//! 0x02E0  scratch area for buffers and arrays     SCRATCH, SCRATCH_LEN bytes
//! 0x04E0  object table: defaults, objects, property tables
//! ......  static memory: abbreviation strings, dictionary
//! ......  high memory: routines, strings, main code
//! ```
//!
//! Objects, dictionary words and abbreviations fix the addresses of everything
//! above them, so they must all be added before the first routine, string or main
//! code. The builder panics if that order is broken.

use crate::zenc::{zencode, zencode_dict};

/// Address of the global variables table.
pub const GLOBALS: u16 = 0x0100;
/// Address of a writable scratch area in dynamic memory.
pub const SCRATCH: u16 = 0x02E0;
/// Size of the scratch area in bytes.
pub const SCRATCH_LEN: u16 = 0x0200;

const ABBREVIATIONS: usize = 0x0040;
const OBJECTS: usize = 0x04E0;

/// One object for the object table (§12.3.1).
#[derive(Clone, Debug)]
pub struct Object {
    /// Short name; may be empty.
    pub name: String,
    /// Attributes; attribute 0 is the top bit (§12.3.1).
    pub attributes: u32,
    pub parent: u8,
    pub sibling: u8,
    pub child: u8,
    /// Properties as (number, data). Written in descending number order.
    pub props: Vec<(u8, Vec<u8>)>,
}

impl Object {
    pub fn new(name: &str) -> Self {
        Object {
            name: name.to_string(),
            attributes: 0,
            parent: 0,
            sibling: 0,
            child: 0,
            props: Vec::new(),
        }
    }
}

/// Builder for a version 3 story image.
pub struct Story {
    flags1: u8,
    release: u16,
    serial: [u8; 6],
    globals: [u16; 240],
    prop_defaults: [u16; 31],
    objects: Vec<Object>,
    words: Vec<String>,
    separators: Vec<u8>,
    abbreviations: Vec<(usize, String)>,
    scratch: Vec<u8>,
    /// Everything below high memory, once the data is frozen.
    low: Option<Vec<u8>>,
    dictionary_at: usize,
    static_base: usize,
    high: Vec<u8>,
    main_at: Option<usize>,
}

impl Default for Story {
    fn default() -> Self {
        Self::new()
    }
}

impl Story {
    pub fn new() -> Self {
        Story {
            flags1: 0,
            release: 1,
            serial: *b"000000",
            globals: [0; 240],
            prop_defaults: [0; 31],
            objects: Vec::new(),
            words: Vec::new(),
            separators: b".,\"".to_vec(),
            abbreviations: Vec::new(),
            scratch: Vec::new(),
            low: None,
            dictionary_at: 0,
            static_base: 0,
            high: Vec::new(),
            main_at: None,
        }
    }

    fn data_still_open(&self, what: &str) {
        assert!(
            self.low.is_none(),
            "{what} must be added before any routine, string or main code"
        );
    }

    pub fn release(&mut self, release: u16) -> &mut Self {
        self.release = release;
        self
    }

    /// Six ASCII characters (§11.1).
    pub fn serial(&mut self, serial: &str) -> &mut Self {
        self.serial.copy_from_slice(serial.as_bytes());
        self
    }

    /// The Flags 1 byte as stored in the file (§11.1). Bit 1 set makes a time game.
    pub fn flags1(&mut self, flags: u8) -> &mut Self {
        self.flags1 = flags;
        self
    }

    /// Initial value of global variable `index` (0 to 239). Global 0 is variable 16.
    pub fn global(&mut self, index: usize, value: u16) -> &mut Self {
        self.data_still_open("globals");
        self.globals[index] = value;
        self
    }

    /// Default value of property `number` (1 to 31) (§12.2).
    pub fn prop_default(&mut self, number: u8, value: u16) -> &mut Self {
        self.data_still_open("property defaults");
        self.prop_defaults[number as usize - 1] = value;
        self
    }

    /// Adds an object and returns its number, counting from 1.
    pub fn object(&mut self, object: Object) -> u8 {
        self.data_still_open("objects");
        self.objects.push(object);
        self.objects.len() as u8
    }

    /// Adds a dictionary word.
    pub fn word(&mut self, word: &str) -> &mut Self {
        self.data_still_open("dictionary words");
        self.words.push(word.to_string());
        self
    }

    /// Replaces the dictionary's word-separator characters (§13.2).
    pub fn separators(&mut self, separators: &str) -> &mut Self {
        self.data_still_open("separators");
        self.separators = separators.as_bytes().to_vec();
        self
    }

    /// Sets abbreviation `index` (0 to 95); 32 * (z - 1) + x in the terms of §3.3.
    pub fn abbreviation(&mut self, index: usize, text: &str) -> &mut Self {
        self.data_still_open("abbreviations");
        assert!(index < 96);
        self.abbreviations.push((index, text.to_string()));
        self
    }

    /// Places `bytes` in the scratch area of dynamic memory and returns the address.
    pub fn scratch(&mut self, bytes: &[u8]) -> u16 {
        self.data_still_open("scratch data");
        let at = SCRATCH as usize + self.scratch.len();
        self.scratch.extend_from_slice(bytes);
        assert!(self.scratch.len() <= SCRATCH_LEN as usize, "scratch area is full");
        at as u16
    }

    /// Reserves `len` zero bytes in the scratch area and returns the address.
    pub fn reserve(&mut self, len: usize) -> u16 {
        self.scratch(&vec![0; len])
    }

    fn freeze(&mut self) {
        if self.low.is_some() {
            return;
        }
        let mut low = vec![0u8; OBJECTS];

        for (i, g) in self.globals.iter().enumerate() {
            let at = GLOBALS as usize + i * 2;
            low[at..at + 2].copy_from_slice(&g.to_be_bytes());
        }
        let s = SCRATCH as usize;
        low[s..s + self.scratch.len()].copy_from_slice(&self.scratch);

        // Object table (§12): 31 default words, 9-byte entries, property tables.
        for d in self.prop_defaults {
            low.extend(d.to_be_bytes());
        }
        let entries = low.len();
        low.resize(entries + self.objects.len() * 9, 0);
        for (i, o) in self.objects.iter().enumerate() {
            let table = low.len();
            let name = if o.name.is_empty() { Vec::new() } else { zencode(&o.name) };
            low.push((name.len() / 2) as u8);
            low.extend(&name);
            let mut props = o.props.clone();
            props.sort_by(|a, b| b.0.cmp(&a.0));
            for (number, data) in &props {
                assert!((1..=31).contains(number), "property numbers are 1 to 31");
                assert!((1..=8).contains(&data.len()), "property data is 1 to 8 bytes");
                low.push(32 * (data.len() as u8 - 1) + number);
                low.extend(data);
            }
            low.push(0);
            let e = entries + i * 9;
            low[e..e + 4].copy_from_slice(&o.attributes.to_be_bytes());
            low[e + 4] = o.parent;
            low[e + 5] = o.sibling;
            low[e + 6] = o.child;
            low[e + 7..e + 9].copy_from_slice(&(table as u16).to_be_bytes());
        }
        if low.len() % 2 != 0 {
            low.push(0);
        }
        self.static_base = low.len();

        // Abbreviation strings, with their word addresses in the table (§3.3).
        // Every unset entry points at one shared empty string.
        let empty = low.len();
        low.extend(zencode(""));
        for i in 0..96 {
            let at = ABBREVIATIONS + i * 2;
            low[at..at + 2].copy_from_slice(&((empty / 2) as u16).to_be_bytes());
        }
        for (index, text) in &self.abbreviations {
            let addr = low.len();
            low.extend(zencode(text));
            let at = ABBREVIATIONS + index * 2;
            low[at..at + 2].copy_from_slice(&((addr / 2) as u16).to_be_bytes());
        }

        // Dictionary (§13.2): separators, entry length, count, sorted entries.
        self.dictionary_at = low.len();
        low.push(self.separators.len() as u8);
        low.extend(&self.separators);
        low.push(7);
        let mut keys: Vec<[u8; 4]> = self.words.iter().map(|w| zencode_dict(w)).collect();
        keys.sort();
        keys.dedup();
        low.extend((keys.len() as u16).to_be_bytes());
        for key in keys {
            low.extend(key);
            low.extend([0, 0, 0]);
        }
        if low.len() % 2 != 0 {
            low.push(0);
        }
        self.low = Some(low);
    }

    fn high_base(&mut self) -> usize {
        self.freeze();
        self.low.as_ref().unwrap().len()
    }

    fn align(&mut self) {
        if self.high.len() % 2 != 0 {
            self.high.push(0);
        }
    }

    /// Address of the dictionary. Freezes the data layout.
    pub fn dictionary_address(&mut self) -> u16 {
        self.freeze();
        self.dictionary_at as u16
    }

    /// Address of the object table. Does not depend on the data added.
    pub fn object_table_address(&self) -> u16 {
        OBJECTS as u16
    }

    /// Address of the first byte above dynamic memory. Freezes the data layout.
    pub fn static_base(&mut self) -> u16 {
        self.freeze();
        self.static_base as u16
    }

    /// The byte address at which the next unaligned code would be placed, which is
    /// where [`Story::main`] will put its code. Freezes the data layout.
    pub fn next_code_address(&mut self) -> u32 {
        (self.high_base() + self.high.len()) as u32
    }

    /// Adds a routine with the given initial local values and returns its packed
    /// address (§5.2, §1.2.3). `code` starts with the routine's first instruction.
    pub fn routine(&mut self, locals: &[u16], code: &[u8]) -> u16 {
        assert!(locals.len() <= 15, "at most 15 locals");
        let base = self.high_base();
        self.align();
        let at = base + self.high.len();
        self.high.push(locals.len() as u8);
        for l in locals {
            self.high.extend(l.to_be_bytes());
        }
        self.high.extend_from_slice(code);
        (at / 2) as u16
    }

    /// Adds a Z-encoded string in high memory and returns its packed address.
    pub fn string(&mut self, text: &str) -> u16 {
        self.raw_string(&zencode(text))
    }

    /// Adds already-encoded string bytes in high memory and returns the packed
    /// address. Useful for strings no encoder would produce.
    pub fn raw_string(&mut self, bytes: &[u8]) -> u16 {
        let base = self.high_base();
        self.align();
        let at = base + self.high.len();
        self.high.extend_from_slice(bytes);
        (at / 2) as u16
    }

    /// Adds the code execution starts at. It is not a routine: it has no header of
    /// locals, and the initial program counter points at its first byte (§5.5).
    pub fn main(&mut self, code: &[u8]) -> &mut Self {
        assert!(self.main_at.is_none(), "main code was already added");
        let at = self.next_code_address() as usize;
        self.high.extend_from_slice(code);
        self.main_at = Some(at);
        self
    }

    /// Produces the finished image, with length and checksum filled in (§11.1).
    pub fn build(&mut self) -> Vec<u8> {
        let high_base = self.high_base();
        let main_at = self.main_at.expect("a story needs main code");
        let mut image = self.low.clone().unwrap();
        image.extend(&self.high);
        if image.len() % 2 != 0 {
            image.push(0);
        }
        assert!(image.len() <= 0x1_FFFE, "a version 3 story is at most 128K");

        let put = |image: &mut Vec<u8>, at: usize, value: usize| {
            image[at..at + 2].copy_from_slice(&(value as u16).to_be_bytes());
        };
        image[0x00] = 3;
        image[0x01] = self.flags1;
        put(&mut image, 0x02, self.release as usize);
        put(&mut image, 0x04, high_base);
        put(&mut image, 0x06, main_at);
        put(&mut image, 0x08, self.dictionary_at);
        put(&mut image, 0x0A, OBJECTS);
        put(&mut image, 0x0C, GLOBALS as usize);
        put(&mut image, 0x0E, self.static_base);
        image[0x12..0x18].copy_from_slice(&self.serial);
        put(&mut image, 0x18, ABBREVIATIONS);
        let len = image.len();
        put(&mut image, 0x1A, len / 2);
        let sum = image[0x40..].iter().fold(0u32, |a, &b| a + b as u32) & 0xFFFF;
        put(&mut image, 0x1C, sum as usize);
        image
    }
}
