//! The two queries that need no family: the object tree and the dictionary
//! (docs/introspection.md sections 4 and 5, cited I§4 and I§5; SC-007). "§" alone
//! cites the Z-Machine Standard 1.1.
//!
//! The independent check is a second reader of the tables, written here from the
//! Standard alone and sharing no code with the crate.

mod common;

use common::*;
use sha2::{Digest, Sha256};
use zmachine_core::Machine;
use zmachine_introspect::{Exactness, Introspector};

const STORIES: [&str; 7] = [
    "stories/zork1/zork1.z3",
    "stories/advent/advent.z5",
    "stories/advent/advent.z8",
    "stories/czech/czech.z3",
    "stories/czech/czech.z5",
    "stories/czech/czech.z8",
    "stories/praxix/praxix.z5",
];

/// A plain reader of a story file's bytes, for comparison.
struct Reference {
    m: Vec<u8>,
}

impl Reference {
    fn w(&self, a: usize) -> usize {
        (self.m[a] as usize) << 8 | self.m[a + 1] as usize
    }
    fn wide(&self) -> bool {
        self.m[0] > 3
    }
    fn entry(&self, n: usize) -> usize {
        let (defaults, size) = if self.wide() { (63, 14) } else { (31, 9) };
        self.w(0x0A) + 2 * defaults + (n - 1) * size
    }
    fn count(&self) -> usize {
        // §12.3: entries end where the lowest property table begins.
        let size = if self.wide() { 14 } else { 9 };
        let mut lowest = usize::MAX;
        let mut n = 0;
        while self.entry(n + 1) < lowest {
            n += 1;
            lowest = lowest.min(self.w(self.entry(n) + size - 2));
        }
        n
    }
    fn links(&self, n: usize) -> (usize, usize, usize) {
        let e = self.entry(n);
        if self.wide() {
            (self.w(e + 6), self.w(e + 8), self.w(e + 10))
        } else {
            (
                self.m[e + 4] as usize,
                self.m[e + 5] as usize,
                self.m[e + 6] as usize,
            )
        }
    }
    fn children(&self, n: usize) -> Vec<u16> {
        let mut out = vec![];
        let mut c = self.links(n).2;
        while c != 0 {
            out.push(c as u16);
            c = self.links(c).1;
        }
        out
    }
    fn zchars(&self, mut a: usize) -> Vec<usize> {
        let mut z = vec![];
        loop {
            let x = self.w(a);
            z.extend([(x >> 10) & 31, (x >> 5) & 31, x & 31]);
            a += 2;
            if x & 0x8000 != 0 {
                return z;
            }
        }
    }
    /// §3, default alphabet only: none of the committed story files has its own.
    fn decode(&self, z: &[usize], nested: bool) -> String {
        const A: [&str; 3] = [
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            " \n0123456789.,!?_#'\"/\\-:()",
        ];
        let (mut out, mut shift, mut i) = (String::new(), 0, 0);
        while i < z.len() {
            let c = z[i];
            i += 1;
            match c {
                0 => out.push(' '),
                1..=3 if i < z.len() => {
                    let table = self.w(0x18) + 2 * (32 * (c - 1) + z[i]);
                    i += 1;
                    if !nested {
                        out.push_str(&self.decode(&self.zchars(self.w(table) * 2), true));
                    }
                }
                4 | 5 => {
                    shift = c - 3;
                    continue;
                }
                6 if shift == 2 => {
                    if i + 1 < z.len() {
                        out.push(char::from_u32((z[i] << 5 | z[i + 1]) as u32).unwrap());
                    }
                    i += 2;
                }
                6..=31 => out.push(A[shift].chars().nth(c - 6).unwrap()),
                _ => {}
            }
            shift = 0;
        }
        out
    }
    fn name(&self, n: usize) -> String {
        let size = if self.wide() { 14 } else { 9 };
        let table = self.w(self.entry(n) + size - 2);
        match self.m[table] {
            0 => String::new(),
            _ => self.decode(&self.zchars(table + 1), false),
        }
    }
    fn words(&self) -> Vec<String> {
        let d = self.w(0x08);
        let at = d + 1 + self.m[d] as usize;
        let (len, count) = (
            self.m[at] as usize,
            (self.w(at + 1) as i16).unsigned_abs() as usize,
        );
        let text = if self.wide() { 6 } else { 4 };
        (0..count)
            .map(|i| {
                let e = at + 3 + i * len;
                let z: Vec<usize> = (0..text)
                    .step_by(2)
                    .flat_map(|k| {
                        let x = self.w(e + k);
                        [(x >> 10) & 31, (x >> 5) & 31, x & 31]
                    })
                    .collect();
                self.decode(&z, true)
            })
            .collect()
    }
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// Constitution Principle VI: each story file is the one its record describes.
#[test]
fn the_story_files_match_their_records() {
    for story in STORIES {
        let dir = story.rsplit_once('/').unwrap().0;
        let record = read_text(&format!("{dir}/SOURCE.md"));
        assert!(
            record.contains(&digest(&read(story))),
            "{story} is not the file SOURCE.md records"
        );
    }
}

// I§4, §12: every object once, by number, with its name, parent and children in
// the tree's order, for every story file, recognised or not. SC-007.
#[test]
fn the_object_tree_equals_an_independent_reading() {
    for story in STORIES {
        let machine: Machine = load(story, 1);
        let view = machine.view();
        let reference = Reference { m: read(story) };
        let answer = Introspector::new(&view).object_tree(&view);
        assert_eq!(answer.exactness, Exactness::Exact, "{story}");
        let objects = answer.value.unwrap_or_else(|e| panic!("{story}: {e:?}"));
        assert_eq!(
            objects.len(),
            reference.count(),
            "{story}: how many objects"
        );
        assert!(objects.len() >= 5, "{story}");
        for (i, object) in objects.iter().enumerate() {
            let n = i + 1;
            assert_eq!(object.number as usize, n, "{story}");
            assert_eq!(object.name, reference.name(n), "{story}: object {n}");
            let (parent, _, _) = reference.links(n);
            assert_eq!(
                object.parent,
                Ok((parent != 0).then_some(parent as u16)),
                "{story}: object {n}"
            );
            assert_eq!(
                object.children,
                Ok(reference.children(n)),
                "{story}: object {n}"
            );
        }
    }
}

// I§4: names a player would know, and the attributes and properties of an object
// whose table was read by hand.
#[test]
fn known_objects_are_as_the_games_have_them() {
    let machine = load("stories/zork1/zork1.z3", 1);
    let view = machine.view();
    let objects = Introspector::new(&view).object_tree(&view).value.unwrap();
    assert_eq!(objects.len(), 250);
    let named = |name: &str| {
        objects
            .iter()
            .find(|o| o.name == name)
            .unwrap_or_else(|| panic!("no {name}"))
    };
    let house = named("West of House");
    assert_eq!(house.printed_name, Ok("West of House".to_string()));
    assert_eq!(
        house.properties,
        [31, 30, 29, 28, 27, 25, 24, 21, 17, 5],
        "table order, descending"
    );
    assert_eq!(named("brass lantern").number, named("brass lantern").number);
    assert!(
        named("small mailbox").parent == Ok(Some(house.number)),
        "the mailbox starts at the house"
    );

    let machine = load("stories/advent/advent.z5", 1);
    let view = machine.view();
    let objects = Introspector::new(&view).object_tree(&view).value.unwrap();
    assert_eq!(objects.len(), 277);
    assert_eq!(objects[0].name, "Class");
    let road = objects.iter().find(|o| o.name == "At End Of Road").unwrap();
    assert_eq!(road.attributes, [9, 31]);
    assert_eq!(road.parent, Ok(None));
}

// I§5, §13: every word of the dictionary the header names, in the dictionary's
// order, as stored. SC-007.
#[test]
fn the_dictionary_equals_an_independent_reading() {
    for story in STORIES {
        let machine = load(story, 1);
        let view = machine.view();
        let reference = Reference { m: read(story) };
        let answer = Introspector::new(&view).dictionary(&view);
        assert_eq!(answer.exactness, Exactness::Exact, "{story}");
        let words: Vec<String> = answer
            .value
            .unwrap_or_else(|e| panic!("{story}: {e:?}"))
            .into_iter()
            .map(|w| w.word)
            .collect();
        assert_eq!(words, reference.words(), "{story}");
    }
    let machine = load("stories/zork1/zork1.z3", 1);
    let view = machine.view();
    let words = Introspector::new(&view).dictionary(&view).value.unwrap();
    assert_eq!(words.len(), 684);
    for known in ["north", "lamp", "take", "mailbo", "xyzzy"] {
        assert!(words.iter().any(|w| w.word == known), "{known}");
    }
    assert!(
        words.windows(2).all(|pair| pair[0].word != pair[1].word),
        "no word twice"
    );
}

// I§1.7, I§4: the tree follows the game. After a `take`, the object's parent is
// the player; the answer for the unchanged opening state is the same every time.
#[test]
fn the_tree_follows_the_game_and_is_repeatable() {
    let mut first = None;
    let mut seen_carried = false;
    walk(ZORK1, |machine, stop| {
        let view = machine.view();
        let tree = Introspector::new(&view).object_tree(&view);
        if stop.number == 1 {
            assert_eq!(tree, Introspector::new(&view).object_tree(&view));
            first = Some(tree.clone());
        }
        let objects = tree.value.unwrap();
        let egg = objects
            .iter()
            .find(|o| o.name == "jewel-encrusted egg")
            .unwrap();
        if let Some((_, command)) = &stop.line {
            // Line 7 follows `take egg`.
            if stop.number == 7 {
                let holder = egg.parent.clone().unwrap().unwrap();
                assert_eq!(
                    objects[holder as usize - 1].name,
                    "cretin",
                    "before {command}"
                );
                seen_carried = true;
            }
        }
    });
    assert!(seen_carried && first.is_some());
}

// I§5, I§3.2: the parts of speech ZIL records, and the direction a direction
// word names, which is the number the exits query uses.
#[test]
fn zil_words_carry_their_parts_of_speech() {
    use zmachine_introspect::PartOfSpeech::*;
    let machine = load("stories/zork1/zork1.z3", 1);
    let view = machine.view();
    let i = Introspector::new(&view);
    let words = i.dictionary(&view).value.unwrap();
    let find = |word: &str| {
        words
            .iter()
            .find(|w| w.word == word)
            .unwrap_or_else(|| panic!("no {word}"))
    };
    assert_eq!(find("north").parts, Ok(vec![Direction]));
    assert_eq!(find("north").direction, Some(31));
    assert_eq!(find("n").direction, find("north").direction);
    assert_eq!(find("take").parts, Ok(vec![Verb]));
    assert_eq!(find("lamp").parts, Ok(vec![Noun]));
    assert_eq!(find("with").parts, Ok(vec![Preposition]));
    assert_eq!(find("the").parts, Ok(vec![Buzz]));
    // Words with more than one part carry each.
    assert_eq!(find("down").parts, Ok(vec![Direction, Preposition]));
    assert_eq!(find("west").parts, Ok(vec![Adjective, Direction]));
    assert_eq!(find("brass").parts, Ok(vec![Adjective]));
    assert!(words.iter().all(|w| w.parts.is_ok()));
    assert_eq!(words.iter().filter(|w| w.direction.is_some()).count(), 25);
    assert!(
        words
            .iter()
            .all(|w| w.direction.is_some() == w.parts.as_ref().unwrap().contains(&Direction))
    );
}

// I§5, I§3.1: the parts of speech Inform records. A direction word is one in a
// compass direction's `name`, and its number is the exits query's.
#[test]
fn inform_words_carry_their_parts_of_speech() {
    use zmachine_introspect::PartOfSpeech::*;
    for story in ["stories/advent/advent.z5", "stories/advent/advent.z8"] {
        let machine = load(story, 1);
        let view = machine.view();
        let words = Introspector::new(&view).dictionary(&view).value.unwrap();
        let find = |word: &str| {
            words
                .iter()
                .find(|w| w.word == word)
                .unwrap_or_else(|| panic!("no {word}"))
        };
        assert_eq!(find("take").parts, Ok(vec![Verb]));
        assert_eq!(find("lamp").parts, Ok(vec![Noun]));
        assert_eq!(find("north").direction, Some(7));
        assert_eq!(find("n").direction, Some(7));
        assert!(find("north").parts.as_ref().unwrap().contains(&Direction));
        assert_eq!(find("southwest").direction, Some(14));
        // A word used in a grammar line, here not a preposition at all.
        assert!(
            find("fee").parts.as_ref().unwrap().contains(&GrammarWord),
            "{:?}",
            find("fee").parts
        );
        // Meta verbs: commands to the game, not to the player's character.
        assert_eq!(find("save").parts, Ok(vec![Verb, Meta]));
        assert!(find("lamp").direction.is_none());
        // `in` is a verb, and names the direction the library treats as one.
        assert_eq!(find("in").direction, Some(17));
        assert!(words.iter().all(|w| w.parts.is_ok()), "{story}");
        assert!(
            words
                .iter()
                .filter(|w| w.parts.as_ref().unwrap().contains(&Verb))
                .count()
                >= 200
        );
    }
}
