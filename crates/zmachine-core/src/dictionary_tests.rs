use super::*;
use crate::header;
use crate::memory::Memory;
use zmachine_testkit::{Story, op, zencode_dict};

struct Fixture {
    memory: Memory,
    dictionary: u16,
    text: u16,
    parse: u16,
}

/// A story whose dictionary holds a, lamp, north, northeast, take and the comma,
/// with a text buffer of 40 and a parse buffer for `max_words` words.
fn fixture(max_words: u8) -> Fixture {
    let mut story = Story::new();
    for w in ["take", "north", "a", "lamp", "northeast", ","] {
        story.word(w);
    }
    let mut text = vec![0u8; 42];
    text[0] = 40;
    let text = story.scratch(&text);
    let mut parse = vec![0xEEu8; 2 + 4 * 12];
    parse[0] = max_words;
    let parse = story.scratch(&parse);
    story.main(&op::op0(op::QUIT));
    let image = story.build();
    let header = header::parse(&image).unwrap();
    Fixture {
        memory: Memory::new(image, header.static_base as usize),
        dictionary: header.dictionary,
        text,
        parse,
    }
}

impl Fixture {
    fn find(&self, word: &str) -> u16 {
        lookup(&self.memory, self.dictionary, zencode_dict(word)).unwrap()
    }

    /// Stores the line and tokenises it; returns (dictionary address, length,
    /// position) for each word found.
    fn tokens(&mut self, line: &str) -> Vec<(u16, u8, u8)> {
        store_line(&mut self.memory, self.text, line.as_bytes()).unwrap();
        tokenise(&mut self.memory, self.dictionary, self.text, self.parse).unwrap();
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

// §13.2: the dictionary is a header then sorted entries; a lookup returns the byte
// address of the entry, or 0 (§13.6.3, §15 read).
#[test]
fn lookup_finds_first_middle_last_and_reports_missing_as_zero() {
    let f = fixture(10);
    let all: Vec<u16> = [",", "a", "lamp", "north", "northeast", "take"]
        .iter()
        .map(|w| f.find(w))
        .collect();
    assert!(all.iter().all(|&a| a != 0), "{all:x?}");
    // Entries are 7 bytes apart, in sorted order of their encoded text.
    let mut sorted = all.clone();
    sorted.sort();
    assert_eq!(
        sorted.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>(),
        vec![7; 5]
    );
    assert_eq!(f.find("lantern"), 0);
    assert_eq!(f.find(""), 0);
    assert_eq!(f.find("zzzzzz"), 0);
}

// §13.3: only the first six Z-characters are significant.
#[test]
fn lookup_ignores_characters_past_the_sixth() {
    let f = fixture(10);
    assert_eq!(f.find("northeastern"), f.find("northeast"));
    assert_eq!(f.find("northe"), f.find("northeast"));
    assert_ne!(f.find("north"), f.find("northeast"));
}

// §15 read: the text is stored from byte 1 in lower case with a zero terminator
// (versions 1 to 4).
#[test]
fn store_line_writes_lower_case_text_and_a_terminator() {
    let mut f = fixture(10);
    store_line(&mut f.memory, f.text, b"Take LAMP").unwrap();
    let got: Vec<u8> = (1..=10)
        .map(|i| f.memory.read_byte(f.text as u32 + i).unwrap())
        .collect();
    assert_eq!(&got, b"take lamp\0");
    assert_eq!(
        f.memory.read_byte(f.text as u32).unwrap(),
        40,
        "the size byte is untouched"
    );
}

// §13.6: words are split at spaces. Each parse block is the dictionary address, the
// number of letters, and the position of the first letter in the text buffer.
#[test]
fn tokenise_simple_words() {
    let mut f = fixture(10);
    let (take, lamp) = (f.find("take"), f.find("lamp"));
    assert_eq!(f.tokens("take lamp"), vec![(take, 4, 1), (lamp, 4, 6)]);
    assert_eq!(
        f.tokens("  take   lamp  "),
        vec![(take, 4, 3), (lamp, 4, 10)]
    );
    assert_eq!(f.tokens("take xyzzy"), vec![(take, 4, 1), (0, 5, 6)]);
}

// §13.6.1: a word separator is a word in its own right, and also ends the word
// before it. A separator that is not in the dictionary's word list looks up as 0.
#[test]
fn separators_are_their_own_words() {
    let mut f = fixture(10);
    let (take, lamp, north, comma) = (f.find("take"), f.find("lamp"), f.find("north"), f.find(","));
    assert_eq!(
        f.tokens("take lamp,north."),
        vec![
            (take, 4, 1),
            (lamp, 4, 6),
            (comma, 1, 10),
            (north, 5, 11),
            (0, 1, 16)
        ]
    );
    assert_eq!(f.tokens(",,"), vec![(comma, 1, 1), (comma, 1, 2)]);
}

// §15 read: an empty line gives zero words.
#[test]
fn an_empty_line_has_no_words() {
    let mut f = fixture(10);
    assert_eq!(f.tokens(""), vec![]);
    assert_eq!(f.tokens("     "), vec![]);
}

// §15 read: "The maximum number of words is in byte 0 of the parse buffer"; words
// beyond it are dropped, and nothing is written past the space for them.
#[test]
fn words_beyond_the_parse_buffer_are_dropped() {
    let mut f = fixture(2);
    let (take, a) = (f.find("take"), f.find("a"));
    assert_eq!(f.tokens("take a lamp north"), vec![(take, 4, 1), (a, 1, 6)]);
    let after = f.parse as u32 + 2 + 2 * 4;
    assert_eq!(
        f.memory.read_byte(after).unwrap(),
        0xEE,
        "nothing written past two blocks"
    );
    assert_eq!(
        f.memory.read_byte(f.parse as u32).unwrap(),
        2,
        "the size byte is untouched"
    );
}

// §13.3: a long word is looked up by its first six characters but its true length
// is recorded.
#[test]
fn a_long_word_keeps_its_full_length() {
    let mut f = fixture(10);
    let northeast = f.find("northeast");
    assert_eq!(f.tokens("northeastern"), vec![(northeast, 12, 1)]);
}

// The real thing: Zork I's dictionary finds its own words (stories/zork1/SOURCE.md).
#[test]
fn zork1_dictionary_lookups() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../stories/zork1/zork1.z3");
    let story = std::fs::read(path).unwrap();
    let header = header::parse(&story).unwrap();
    let memory = Memory::new(story, header.static_base as usize);
    for w in [
        "mailbox", "open", "north", "xyzzy", "lantern", "zzmgck", "$ve", "\"", ",",
    ] {
        let key = zencode_dict(w);
        assert_ne!(lookup(&memory, header.dictionary, key).unwrap(), 0, "{w}");
    }
    assert_eq!(
        lookup(&memory, header.dictionary, zencode_dict("qqqqqq")).unwrap(),
        0
    );
}
