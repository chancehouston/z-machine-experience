//! The dictionary and lexical analysis for versions 5 and 8 (feature 002, T012).

use super::*;
use crate::header;
use crate::memory::Memory;
use zmachine_testkit::{op, zencode_dict9, Story5};

struct Fixture {
    memory: Memory,
    dictionary: u16,
    own_sorted: u16,
    own_unsorted: u16,
    text: u16,
    parse: u16,
}

fn fixture(max_words: u8) -> Fixture {
    let mut story = Story5::new(5);
    for w in ["take", "north", "a", "lantern", "inventory", ","] {
        story.word(w);
    }
    let text = story.text_buffer(60, "");
    let mut parse = vec![0xEEu8; 2 + 4 * 12];
    parse[0] = max_words;
    let parse = story.scratch(&parse);
    let own_sorted = story.scratch(&Story5::dictionary_bytes("!", &["zebra", "apple", "mango"], true));
    let own_unsorted =
        story.scratch(&Story5::dictionary_bytes("!", &["zebra", "apple", "mango"], false));
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let h = header::parse(&image).unwrap();
    Fixture {
        memory: Memory::new(image, h.static_base as usize),
        dictionary: h.dictionary,
        own_sorted,
        own_unsorted,
        text,
        parse,
    }
}

impl Fixture {
    fn find(&self, dictionary: u16, word: &str) -> u16 {
        lookup_key(&self.memory, dictionary, &zencode_dict9(word)).unwrap()
    }

    fn lexer(&self, dictionary: u16, skip_unknown: bool) -> Lexer<'static> {
        static DEFAULT: std::sync::OnceLock<Charset> = std::sync::OnceLock::new();
        Lexer {
            version: Version::V5,
            charset: DEFAULT.get_or_init(Charset::default),
            dictionary,
            skip_unknown,
        }
    }

    /// Stores the line and tokenises it; returns (address, length, position).
    fn tokens(&mut self, dictionary: u16, line: &str) -> Vec<(u16, u8, u8)> {
        self.memory.write_byte(self.text as u32 + 1, 0).unwrap();
        append_line(&mut self.memory, self.text, line.as_bytes()).unwrap();
        self.lexer(dictionary, false).tokenise(&mut self.memory, self.text, self.parse).unwrap();
        self.blocks()
    }

    fn blocks(&self) -> Vec<(u16, u8, u8)> {
        let count = self.memory.read_byte(self.parse as u32 + 1).unwrap();
        (0..count as u32)
            .map(|i| {
                let at = self.parse as u32 + 2 + i * 4;
                (
                    self.memory.read_word(at).unwrap(),
                    self.memory.read_byte(at + 2).unwrap(),
                    self.memory.read_byte(at + 3).unwrap(),
                )
            })
            .collect()
    }
}

// §13.4: entries are found by their six-byte encoded text; only the first nine
// Z-characters count.
#[test]
fn lookup_by_six_byte_key() {
    let f = fixture(10);
    let d = f.dictionary;
    for w in [",", "a", "inventory", "lantern", "north", "take"] {
        assert_ne!(f.find(d, w), 0, "{w}");
    }
    assert_eq!(f.find(d, "lamp"), 0);
    // "inventory" is exactly nine letters; anything after them is not looked at.
    assert_eq!(f.find(d, "inventoryish"), f.find(d, "inventory"), "nine characters are significant");
    assert_ne!(f.find(d, "inventor"), f.find(d, "inventory"));
}

// §15 read, version 5: byte 1 of the text buffer holds the number of characters and
// the text starts at byte 2, in lower case, with no terminator.
#[test]
fn append_line_writes_the_version_5_layout() {
    let mut f = fixture(10);
    append_line(&mut f.memory, f.text, b"Take LANTERN").unwrap();
    let byte = |i: u32| f.memory.read_byte(f.text as u32 + i).unwrap();
    assert_eq!((byte(0), byte(1)), (60, 12));
    let got: Vec<u8> = (2..14).map(byte).collect();
    assert_eq!(&got, b"take lantern");
    assert_eq!(byte(14), 0, "nothing is written after the text");
}

// §15 read: "the interpreter writes the new characters after those already there".
#[test]
fn append_line_keeps_text_already_in_the_buffer() {
    let mut f = fixture(10);
    append_line(&mut f.memory, f.text, b"take ").unwrap();
    append_line(&mut f.memory, f.text, b"Lantern").unwrap();
    let byte = |i: u32| f.memory.read_byte(f.text as u32 + i).unwrap();
    assert_eq!(byte(1), 12);
    let got: Vec<u8> = (2..14).map(byte).collect();
    assert_eq!(&got, b"take lantern");
}

// §13.6: positions in the parse buffer count from the start of the text buffer, so
// the first letter is at 2 in version 5 (it is at 1 in version 3).
#[test]
fn tokenise_positions_start_at_two() {
    let mut f = fixture(10);
    let d = f.dictionary;
    let (take, lantern, comma, north) =
        (f.find(d, "take"), f.find(d, "lantern"), f.find(d, ","), f.find(d, "north"));
    assert_eq!(
        f.tokens(d, "take lantern,north xyzzy"),
        vec![(take, 4, 2), (lantern, 7, 7), (comma, 1, 14), (north, 5, 15), (0, 5, 21)]
    );
    assert_eq!(f.tokens(d, ""), vec![]);
}

// §15 read: words beyond the parse buffer's capacity are dropped.
#[test]
fn words_beyond_the_parse_buffer_are_dropped() {
    let mut f = fixture(2);
    let d = f.dictionary;
    assert_eq!(f.tokens(d, "take a lantern north").len(), 2);
    let after = f.parse as u32 + 2 + 2 * 4;
    assert_eq!(f.memory.read_byte(after).unwrap(), 0xEE);
}

// §15 tokenise: a dictionary of the game's own may be given. §13.2: a negative
// entry count means its entries are unsorted, so it is searched in order.
#[test]
fn tokenise_against_a_games_own_dictionary_sorted_or_not() {
    let mut f = fixture(10);
    for d in [f.own_sorted, f.own_unsorted] {
        let (apple, mango, zebra) = (f.find(d, "apple"), f.find(d, "mango"), f.find(d, "zebra"));
        assert!(apple != 0 && mango != 0 && zebra != 0);
        assert_eq!(f.find(d, "take"), 0, "the main dictionary's words are not in this one");
        // This dictionary's separator is '!', not ','.
        assert_eq!(
            f.tokens(d, "zebra!apple,mango"),
            vec![(zebra, 5, 2), (0, 1, 7), (0, 11, 8)]
        );
        assert_eq!(f.tokens(d, "mango apple"), vec![(mango, 5, 2), (apple, 5, 8)]);
    }
    // The unsorted one really is in a different order.
    let order = |d: u16| [f.find(d, "apple"), f.find(d, "zebra")];
    assert!(order(f.own_sorted)[0] < order(f.own_sorted)[1]);
    assert!(order(f.own_unsorted)[0] > order(f.own_unsorted)[1]);
}

// §15 tokenise: with its flag set, "unrecognised words are not written into the
// parse buffer and their slots are left unchanged", so a second pass can fill in
// what a first one did not know.
#[test]
fn the_flag_leaves_unknown_words_untouched() {
    let mut f = fixture(10);
    let (main, own) = (f.dictionary, f.own_sorted);
    let take = f.find(main, "take");
    let zebra = f.find(own, "zebra");
    assert_eq!(f.tokens(main, "take zebra"), vec![(take, 4, 2), (0, 5, 7)]);
    f.lexer(own, true).tokenise(&mut f.memory, f.text, f.parse).unwrap();
    assert_eq!(f.blocks(), vec![(take, 4, 2), (zebra, 5, 7)], "take was left as the first pass wrote it");
}

// The real thing: Advent's dictionary, in both builds.
#[test]
fn advents_dictionary_lookups() {
    for version in [5u8, 8] {
        let path = format!("{}/../../stories/advent/advent.z{version}", env!("CARGO_MANIFEST_DIR"));
        let story = std::fs::read(path).unwrap();
        let h = header::parse(&story).unwrap();
        let m = Memory::new(story, h.static_base as usize);
        for w in ["lantern", "xyzzy", "plugh", "inventory", "north", "take", "undo", "script"] {
            let at = lookup_key(&m, h.dictionary, &zencode_dict9(w)).unwrap();
            assert_ne!(at, 0, "version {version}: {w}");
        }
        assert_eq!(lookup_key(&m, h.dictionary, &zencode_dict9("qqqqqqqqq")).unwrap(), 0);
    }
}
