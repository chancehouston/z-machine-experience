//! Assembles a complete version 5 or version 8 story image.
//!
//! The layout follows [`crate::Story`], with the object table and dictionary in
//! their version 4+ forms (§12.3.2, §13.4), routines without initial local values
//! (§5.2.1), and packed addresses divided by 4 or 8 (§1.2.3):
//!
//! ```text
//! 0x0000  header (64 bytes)
//! 0x0040  abbreviations table (96 words)
//! 0x0100  global variables (240 words)            GLOBALS
//! 0x02E0  scratch area for buffers and arrays     SCRATCH, SCRATCH_LEN bytes
//! 0x04E0  object table: 63 defaults, 14-byte objects, property tables
//! ......  static memory: abbreviation strings, dictionary
//! ......  high memory: routines, strings, main code
//! ```
//!
//! Objects, dictionary words and abbreviations must be added before the first
//! routine, string or main code, as with version 3.

use crate::story::{GLOBALS, SCRATCH, SCRATCH_LEN};
use crate::zenc::{zencode, zencode_dict9};

const ABBREVIATIONS: usize = 0x0040;
const OBJECTS: usize = 0x04E0;

/// One object for a version 5 or 8 object table (§12.3.2).
#[derive(Clone, Debug)]
pub struct Object5 {
    pub name: String,
    /// The 48 attributes in the low 48 bits; attribute 0 is bit 47.
    pub attributes: u64,
    pub parent: u16,
    pub sibling: u16,
    pub child: u16,
    /// Properties as (number 1 to 63, data of 1 to 64 bytes).
    pub props: Vec<(u8, Vec<u8>)>,
}

impl Object5 {
    pub fn new(name: &str) -> Self {
        Object5 {
            name: name.to_string(),
            attributes: 0,
            parent: 0,
            sibling: 0,
            child: 0,
            props: Vec::new(),
        }
    }
}

/// Builder for a version 5 or version 8 story image.
pub struct Story5 {
    version: u8,
    flags1: u8,
    release: u16,
    serial: [u8; 6],
    globals: [u16; 240],
    prop_defaults: [u16; 63],
    objects: Vec<Object5>,
    words: Vec<String>,
    unsorted_dictionary: bool,
    separators: Vec<u8>,
    abbreviations: Vec<(usize, String)>,
    scratch: Vec<u8>,
    /// (header offset, value) pairs written at build time.
    header_words: Vec<(usize, u16)>,
    low: Option<Vec<u8>>,
    dictionary_at: usize,
    static_base: usize,
    high: Vec<u8>,
    main_at: Option<usize>,
}

impl Story5 {
    /// A story of version 5 or 8.
    pub fn new(version: u8) -> Self {
        assert!(
            version == 5 || version == 8,
            "Story5 builds versions 5 and 8"
        );
        Story5 {
            version,
            flags1: 0,
            release: 1,
            serial: *b"000000",
            globals: [0; 240],
            prop_defaults: [0; 63],
            objects: Vec::new(),
            words: Vec::new(),
            unsorted_dictionary: false,
            separators: b".,\"".to_vec(),
            abbreviations: Vec::new(),
            scratch: Vec::new(),
            header_words: Vec::new(),
            low: None,
            dictionary_at: 0,
            static_base: 0,
            high: Vec::new(),
            main_at: None,
        }
    }

    /// What a packed address is multiplied by in this version (§1.2.3).
    pub fn packed_multiplier(&self) -> usize {
        if self.version == 8 { 8 } else { 4 }
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

    pub fn serial(&mut self, serial: &str) -> &mut Self {
        self.serial.copy_from_slice(serial.as_bytes());
        self
    }

    pub fn flags1(&mut self, flags: u8) -> &mut Self {
        self.flags1 = flags;
        self
    }

    /// Sets a header word, such as the terminating characters table (0x2E), the
    /// alphabet table (0x34) or the header extension table (0x36).
    pub fn header_word(&mut self, offset: usize, value: u16) -> &mut Self {
        self.header_words.push((offset, value));
        self
    }

    pub fn global(&mut self, index: usize, value: u16) -> &mut Self {
        self.data_still_open("globals");
        self.globals[index] = value;
        self
    }

    /// Default value of property `number` (1 to 63) (§12.2).
    pub fn prop_default(&mut self, number: u8, value: u16) -> &mut Self {
        self.data_still_open("property defaults");
        self.prop_defaults[number as usize - 1] = value;
        self
    }

    /// Adds an object and returns its number, counting from 1.
    pub fn object(&mut self, object: Object5) -> u16 {
        self.data_still_open("objects");
        self.objects.push(object);
        self.objects.len() as u16
    }

    pub fn word(&mut self, word: &str) -> &mut Self {
        self.data_still_open("dictionary words");
        self.words.push(word.to_string());
        self
    }

    /// Writes the dictionary in the order words were added, with a negative entry
    /// count, which marks it as unsorted (§13.2, `tokenise`).
    pub fn unsorted_dictionary(&mut self) -> &mut Self {
        self.data_still_open("the dictionary's order");
        self.unsorted_dictionary = true;
        self
    }

    pub fn separators(&mut self, separators: &str) -> &mut Self {
        self.data_still_open("separators");
        self.separators = separators.as_bytes().to_vec();
        self
    }

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
        assert!(
            self.scratch.len() <= SCRATCH_LEN as usize,
            "scratch area is full"
        );
        at as u16
    }

    pub fn reserve(&mut self, len: usize) -> u16 {
        self.scratch(&vec![0; len])
    }

    /// A version 5 text buffer for `aread`: byte 0 the capacity, byte 1 the number
    /// of characters already present, then those characters (§15 `read`).
    pub fn text_buffer(&mut self, capacity: u8, preloaded: &str) -> u16 {
        let mut bytes = vec![0u8; capacity as usize + 2];
        bytes[0] = capacity;
        bytes[1] = preloaded.len() as u8;
        bytes[2..2 + preloaded.len()].copy_from_slice(preloaded.as_bytes());
        self.scratch(&bytes)
    }

    /// A parse buffer with room for `max_words` words.
    pub fn parse_buffer(&mut self, max_words: u8) -> u16 {
        let mut bytes = vec![0u8; 2 + 4 * max_words as usize];
        bytes[0] = max_words;
        self.scratch(&bytes)
    }

    /// The bytes of a dictionary (§13.2) holding `words`, for placing in the
    /// scratch area as a dictionary of the game's own.
    pub fn dictionary_bytes(separators: &str, words: &[&str], sorted: bool) -> Vec<u8> {
        let mut out = vec![separators.len() as u8];
        out.extend(separators.as_bytes());
        out.push(9);
        let mut keys: Vec<[u8; 6]> = words.iter().map(|w| zencode_dict9(w)).collect();
        if sorted {
            keys.sort();
            keys.dedup();
            out.extend((keys.len() as u16).to_be_bytes());
        } else {
            out.extend((-(keys.len() as i16)).to_be_bytes());
        }
        for key in keys {
            out.extend(key);
            out.extend([0, 0, 0]);
        }
        out
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

        // Object table (§12.3.2): 63 default words, 14-byte entries, property tables.
        for d in self.prop_defaults {
            low.extend(d.to_be_bytes());
        }
        let entries = low.len();
        low.resize(entries + self.objects.len() * 14, 0);
        for (i, o) in self.objects.iter().enumerate() {
            let table = low.len();
            let name = if o.name.is_empty() {
                Vec::new()
            } else {
                zencode(&o.name)
            };
            low.push((name.len() / 2) as u8);
            low.extend(&name);
            let mut props = o.props.clone();
            props.sort_by_key(|p| std::cmp::Reverse(p.0));
            for (number, data) in &props {
                assert!((1..=63).contains(number), "property numbers are 1 to 63");
                assert!(
                    (1..=64).contains(&data.len()),
                    "property data is 1 to 64 bytes"
                );
                // §12.4.2: one size byte for lengths 1 and 2 (bit 6 set for 2); two
                // for longer, the second holding the length, 0 meaning 64.
                match data.len() {
                    1 => low.push(*number),
                    2 => low.push(0x40 | number),
                    len => low.extend([0x80 | number, 0x80 | (len as u8 & 0x3F)]),
                }
                low.extend(data);
            }
            low.push(0);
            let e = entries + i * 14;
            low[e..e + 6].copy_from_slice(&o.attributes.to_be_bytes()[2..]);
            low[e + 6..e + 8].copy_from_slice(&o.parent.to_be_bytes());
            low[e + 8..e + 10].copy_from_slice(&o.sibling.to_be_bytes());
            low[e + 10..e + 12].copy_from_slice(&o.child.to_be_bytes());
            low[e + 12..e + 14].copy_from_slice(&(table as u16).to_be_bytes());
        }
        if !low.len().is_multiple_of(2) {
            low.push(0);
        }
        self.static_base = low.len();

        // Abbreviation strings; the table holds word addresses in every version
        // (§3.3).
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

        self.dictionary_at = low.len();
        let separators = String::from_utf8(self.separators.clone()).unwrap();
        let words: Vec<&str> = self.words.iter().map(String::as_str).collect();
        low.extend(Self::dictionary_bytes(
            &separators,
            &words,
            !self.unsorted_dictionary,
        ));
        let multiple = self.packed_multiplier();
        while !low.len().is_multiple_of(multiple) {
            low.push(0);
        }
        self.low = Some(low);
    }

    fn high_base(&mut self) -> usize {
        self.freeze();
        self.low.as_ref().unwrap().len()
    }

    /// Pads high memory so the next item has a packed address.
    fn align(&mut self) {
        let multiple = self.packed_multiplier();
        while !self.high.len().is_multiple_of(multiple) {
            self.high.push(0);
        }
    }

    pub fn dictionary_address(&mut self) -> u16 {
        self.freeze();
        self.dictionary_at as u16
    }

    pub fn object_table_address(&self) -> u16 {
        OBJECTS as u16
    }

    pub fn static_base(&mut self) -> u16 {
        self.freeze();
        self.static_base as u16
    }

    /// The byte address at which the next unaligned code would be placed.
    pub fn next_code_address(&mut self) -> u32 {
        (self.high_base() + self.high.len()) as u32
    }

    /// Adds `len` zero bytes to high memory, to push what follows to a high address.
    pub fn filler(&mut self, len: usize) -> &mut Self {
        self.high_base();
        self.high.resize(self.high.len() + len, 0);
        self
    }

    /// Adds a routine with `locals` local variables, all starting at zero (§5.2.1),
    /// and returns its packed address.
    pub fn routine(&mut self, locals: usize, code: &[u8]) -> u16 {
        assert!(locals <= 15, "at most 15 locals");
        let base = self.high_base();
        self.align();
        let at = base + self.high.len();
        self.high.push(locals as u8);
        self.high.extend_from_slice(code);
        let packed = at / self.packed_multiplier();
        assert!(
            packed <= 0xFFFF,
            "the routine is beyond the reach of a packed address"
        );
        packed as u16
    }

    pub fn string(&mut self, text: &str) -> u16 {
        self.raw_string(&zencode(text))
    }

    pub fn raw_string(&mut self, bytes: &[u8]) -> u16 {
        let base = self.high_base();
        self.align();
        let at = base + self.high.len();
        self.high.extend_from_slice(bytes);
        (at / self.packed_multiplier()) as u16
    }

    /// Adds the code execution starts at. In versions 1 to 5 and 8 the initial
    /// program counter is the byte address of its first instruction (§5.5), which
    /// must be within the first 64K.
    pub fn main(&mut self, code: &[u8]) -> &mut Self {
        assert!(self.main_at.is_none(), "main code was already added");
        let at = self.next_code_address() as usize;
        assert!(
            at <= 0xFFFF,
            "main code must start within the first 64K; add it before any filler"
        );
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
        // §11.1.6: the length is stored divided by 4 or 8, so the file is padded.
        let multiple = self.packed_multiplier();
        while !image.len().is_multiple_of(multiple) {
            image.push(0);
        }
        let limit = if self.version == 8 {
            512 * 1024
        } else {
            256 * 1024
        };
        assert!(
            image.len() <= limit,
            "the story is larger than its version allows"
        );

        let put = |image: &mut Vec<u8>, at: usize, value: usize| {
            image[at..at + 2].copy_from_slice(&(value as u16).to_be_bytes());
        };
        image[0x00] = self.version;
        image[0x01] = self.flags1;
        put(&mut image, 0x02, self.release as usize);
        put(&mut image, 0x04, high_base.min(0xFFFF));
        put(&mut image, 0x06, main_at);
        put(&mut image, 0x08, self.dictionary_at);
        put(&mut image, 0x0A, OBJECTS);
        put(&mut image, 0x0C, GLOBALS as usize);
        put(&mut image, 0x0E, self.static_base);
        image[0x12..0x18].copy_from_slice(&self.serial);
        put(&mut image, 0x18, ABBREVIATIONS);
        let len = image.len();
        put(&mut image, 0x1A, len / multiple);
        for (offset, value) in &self.header_words {
            put(&mut image, *offset, *value as usize);
        }
        let sum = image[0x40..].iter().fold(0u32, |a, &b| a + b as u32) & 0xFFFF;
        put(&mut image, 0x1C, sum as usize);
        image
    }
}
