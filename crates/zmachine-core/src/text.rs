//! Text: ZSCII and the Z-character encoding (§3).

use crate::memory::Memory;
use crate::FaultKind;

/// Alphabet A2 from Z-character 8 on (§3.5.3). Characters 6 and 7 are the ZSCII
/// escape and new-line.
const A2: &[u8] = b"0123456789.,!?_#'\"/\\-:()";

/// The default extra characters, ZSCII 155 to 223 (§3.8.7).
const EXTRA: &str = "\u{e4}\u{f6}\u{fc}\u{c4}\u{d6}\u{dc}\u{df}\u{bb}\u{ab}\u{eb}\u{ef}\u{ff}\u{cb}\u{cf}\
\u{e1}\u{e9}\u{ed}\u{f3}\u{fa}\u{fd}\u{c1}\u{c9}\u{cd}\u{d3}\u{da}\u{dd}\
\u{e0}\u{e8}\u{ec}\u{f2}\u{f9}\u{c0}\u{c8}\u{cc}\u{d2}\u{d9}\
\u{e2}\u{ea}\u{ee}\u{f4}\u{fb}\u{c2}\u{ca}\u{ce}\u{d4}\u{db}\
\u{e5}\u{c5}\u{f8}\u{d8}\u{e3}\u{f1}\u{f5}\u{c3}\u{d1}\u{d5}\u{e6}\u{c6}\u{e7}\u{c7}\
\u{fe}\u{f0}\u{de}\u{d0}\u{a3}\u{153}\u{152}\u{a1}\u{bf}";

const EXTRA_FIRST: u16 = 155;

/// How deep abbreviations may nest before the rest is dropped. The Standard forbids
/// an abbreviation inside an abbreviation (§3.3.1); the limit only stops a
/// malformed file from recursing forever.
const MAX_ABBREVIATION_DEPTH: u8 = 3;

/// The character tables in force for a story: the defaults, or the game's own from
/// version 5 (§3.5.5, §3.8.5.2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Charset {
    /// The game's alphabet table: three rows of 26 ZSCII codes, for A0, A1 and A2.
    alphabet: Option<Vec<u8>>,
    /// The game's extra characters, for ZSCII 155 upwards.
    unicode: Option<Vec<char>>,
}

impl Charset {
    /// Reads a story's own tables. Either address may be 0, meaning the default. A
    /// table that runs outside the file is ignored.
    pub fn read(memory: &Memory, alphabet_table: u16, unicode_table: u16) -> Charset {
        let alphabet = (alphabet_table != 0)
            .then(|| (0..78).map(|i| memory.read_byte(alphabet_table as u32 + i).ok()).collect())
            .flatten();
        let unicode = (unicode_table != 0)
            .then(|| {
                let count = memory.read_byte(unicode_table as u32).ok()? as u32;
                (0..count)
                    .map(|i| {
                        let code = memory.read_word(unicode_table as u32 + 1 + 2 * i).ok()?;
                        Some(char::from_u32(code as u32).unwrap_or('?'))
                    })
                    .collect()
            })
            .flatten();
        Charset { alphabet, unicode }
    }

    /// The ZSCII code of Z-character `z` (6 to 31) in alphabet 0, 1 or 2. In A2,
    /// Z-characters 6 and 7 are not looked up: they are the escape and new-line
    /// whatever the table says (§3.5.5.1).
    fn alphabet_char(&self, alphabet: u8, z: u8) -> u16 {
        match &self.alphabet {
            Some(table) => table[alphabet as usize * 26 + (z as usize - 6)] as u16,
            None => match alphabet {
                0 => (b'a' + z - 6) as u16,
                1 => (b'A' + z - 6) as u16,
                _ => A2[(z - 8) as usize] as u16,
            },
        }
    }

    /// Where a ZSCII code is found in the alphabets: (alphabet, Z-character).
    fn find(&self, zscii: u8) -> Option<(u8, u8)> {
        match &self.alphabet {
            Some(table) => table.iter().enumerate().find_map(|(i, &c)| {
                let (alphabet, z) = ((i / 26) as u8, (i % 26) as u8 + 6);
                // The escape and new-line slots of A2 never match.
                (c == zscii && !(alphabet == 2 && z < 8)).then_some((alphabet, z))
            }),
            None => match zscii {
                b'a'..=b'z' => Some((0, zscii - b'a' + 6)),
                b'A'..=b'Z' => Some((1, zscii - b'A' + 6)),
                _ => A2.iter().position(|&a| a == zscii).map(|i| (2, i as u8 + 8)),
            },
        }
    }

    fn extra(&self, code: u16) -> Option<char> {
        let index = (code - EXTRA_FIRST) as usize;
        match &self.unicode {
            Some(table) => table.get(index).copied(),
            None => EXTRA.chars().nth(index),
        }
    }

    /// The character a ZSCII code prints as (§3.8). `None` means it prints nothing;
    /// a code with no meaning on output prints as a question mark.
    pub fn zscii_to_char(&self, code: u16) -> Option<char> {
        match code {
            0 => None,
            13 => Some('\n'),
            32..=126 => Some(code as u8 as char),
            155..=251 => Some(self.extra(code).unwrap_or('?')),
            _ => Some('?'),
        }
    }

    /// The ZSCII code for a character, if it has one (§3.8). Case is kept.
    pub fn zscii_for(&self, ch: char) -> Option<u8> {
        match ch {
            ' '..='~' => Some(ch as u8),
            '\n' => Some(13),
            _ => {
                let index = match &self.unicode {
                    Some(table) => table.iter().position(|&c| c == ch),
                    None => EXTRA.chars().position(|c| c == ch),
                }?;
                u8::try_from(EXTRA_FIRST as usize + index).ok()
            }
        }
    }

    /// The ZSCII code for a character the player typed, in lower case (§15 read). A
    /// character with no code becomes a question mark.
    pub fn input_zscii(&self, ch: char) -> u8 {
        let ch = ch.to_ascii_lowercase();
        match ch {
            ' '..='~' => ch as u8,
            // A line of input has no new-line inside it; one that somehow does is
            // not a character the game can be given.
            '\n' => b'?',
            _ => self.zscii_for(ch).unwrap_or(b'?'),
        }
    }

    fn decode_into(
        &self,
        memory: &Memory,
        address: u32,
        abbreviations: u16,
        depth: u8,
        out: &mut String,
    ) -> Result<u32, FaultKind> {
        enum Pending {
            None,
            /// The next Z-character picks an abbreviation from table 1, 2 or 3.
            Abbreviation(u8),
            /// The next two Z-characters are the halves of a ZSCII code.
            ZsciiHigh,
            ZsciiLow(u8),
        }

        let mut at = address;
        let mut alphabet = 0u8;
        let mut pending = Pending::None;
        loop {
            let word = memory.read_word(at).map_err(|_| FaultKind::TextOverrun)?;
            at += 2;
            for shift in [10, 5, 0] {
                let z = (word >> shift & 0x1F) as u8;
                match pending {
                    Pending::Abbreviation(table) => {
                        pending = Pending::None;
                        if depth < MAX_ABBREVIATION_DEPTH {
                            // §3.3: entry 32(z-1)+x holds a word address, in every
                            // version.
                            let index = 32 * (table as u32 - 1) + z as u32;
                            let entry = abbreviations as u32 + 2 * index;
                            let target =
                                memory.read_word(entry).map_err(|_| FaultKind::TextOverrun)?;
                            self.decode_into(memory, target as u32 * 2, abbreviations, depth + 1, out)?;
                        }
                        continue;
                    }
                    Pending::ZsciiHigh => {
                        pending = Pending::ZsciiLow(z);
                        continue;
                    }
                    Pending::ZsciiLow(high) => {
                        pending = Pending::None;
                        out.extend(self.zscii_to_char((high as u16) << 5 | z as u16));
                        continue;
                    }
                    Pending::None => {}
                }
                let current = alphabet;
                // §3.2.3: from version 3 a shift lasts for one character.
                alphabet = 0;
                match z {
                    0 => out.push(' '),
                    1..=3 => pending = Pending::Abbreviation(z),
                    4 => alphabet = 1,
                    5 => alphabet = 2,
                    6 if current == 2 => pending = Pending::ZsciiHigh,
                    7 if current == 2 => out.push('\n'),
                    _ => out.extend(self.zscii_to_char(self.alphabet_char(current, z))),
                }
            }
            if word & 0x8000 != 0 {
                return Ok(at);
            }
        }
    }

    /// Decodes the Z-string at `address`. Returns the text and the address just
    /// past the string. A string that runs off the end of the file is a fault.
    pub fn decode(
        &self,
        memory: &Memory,
        address: u32,
        abbreviations: u16,
    ) -> Result<(String, u32), FaultKind> {
        let mut out = String::new();
        let end = self.decode_into(memory, address, abbreviations, 0, &mut out)?;
        Ok((out, end))
    }

    /// Encodes ZSCII text as exactly `zchars` Z-characters, truncated or padded
    /// with 5s, packed three to a word with the end bit on the last (§3.7). Used for
    /// dictionary lookup (§13.3, §13.4) and by `encode_text`. `zchars` is a
    /// multiple of 3.
    pub fn encode(&self, word: &[u8], zchars: usize) -> Vec<u8> {
        let mut z: Vec<u8> = Vec::with_capacity(zchars + 4);
        for &c in word {
            if z.len() >= zchars {
                break;
            }
            match (c, self.find(c)) {
                (b' ', _) => z.push(0),
                (_, Some((0, zc))) => z.push(zc),
                (_, Some((alphabet, zc))) => z.extend([3 + alphabet, zc]),
                // §3.7.1: any other code is the A2 escape and two halves.
                (_, None) => z.extend([5, 6, c >> 5, c & 0x1F]),
            }
        }
        z.resize(zchars, 5);
        let words = zchars / 3;
        let mut out = Vec::with_capacity(words * 2);
        for (i, c) in z.chunks(3).enumerate() {
            let mut w = (c[0] as u16) << 10 | (c[1] as u16) << 5 | c[2] as u16;
            if i == words - 1 {
                w |= 0x8000;
            }
            out.extend(w.to_be_bytes());
        }
        out
    }
}

/// The character a ZSCII code prints as with the default tables.
#[cfg(test)]
pub(crate) fn zscii_to_char(code: u16) -> Option<char> {
    // The default table stops at 223; the codes above have no meaning by default.
    Charset::default().zscii_to_char(code)
}

/// The ZSCII code for a character the player typed, with the default tables.
#[cfg(test)]
pub(crate) fn char_to_zscii(ch: char) -> u8 {
    Charset::default().input_zscii(ch)
}

/// Decodes the Z-string at `address` with the default tables.
#[cfg(test)]
pub(crate) fn decode_string(
    memory: &Memory,
    address: u32,
    abbreviations: u16,
) -> Result<(String, u32), FaultKind> {
    Charset::default().decode(memory, address, abbreviations)
}

/// Encodes a word for a version 3 dictionary with the default tables: six
/// Z-characters in four bytes (§13.3).
#[cfg(test)]
pub(crate) fn encode_dict(word: &[u8]) -> [u8; 4] {
    Charset::default().encode(word, 6).try_into().expect("six Z-characters are four bytes")
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "text5_tests.rs"]
mod tests5;
