//! Text: ZSCII and the Z-character encoding (§3), for version 3.

use crate::FaultKind;
use crate::memory::Memory;

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

/// The character a ZSCII code prints as (§3.8). `None` means it prints nothing;
/// a code with no meaning on output prints as a question mark.
pub(crate) fn zscii_to_char(code: u16) -> Option<char> {
    match code {
        0 => None,
        13 => Some('\n'),
        32..=126 => Some(code as u8 as char),
        155..=223 => EXTRA.chars().nth((code - EXTRA_FIRST) as usize),
        _ => Some('?'),
    }
}

/// The ZSCII code for a character the player typed, in lower case (§3.8, §15 read).
/// A character with no code becomes a question mark.
pub(crate) fn char_to_zscii(ch: char) -> u8 {
    let ch = ch.to_ascii_lowercase();
    match ch {
        ' '..='~' => ch as u8,
        _ => match EXTRA.chars().position(|c| c == ch) {
            Some(i) => (EXTRA_FIRST as usize + i) as u8,
            None => b'?',
        },
    }
}

fn decode_into(
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
                        // §3.3: entry 32(z-1)+x holds a word address.
                        let index = 32 * (table as u32 - 1) + z as u32;
                        let entry = abbreviations as u32 + 2 * index;
                        let target = memory
                            .read_word(entry)
                            .map_err(|_| FaultKind::TextOverrun)?;
                        decode_into(memory, target as u32 * 2, abbreviations, depth + 1, out)?;
                    }
                    continue;
                }
                Pending::ZsciiHigh => {
                    pending = Pending::ZsciiLow(z);
                    continue;
                }
                Pending::ZsciiLow(high) => {
                    pending = Pending::None;
                    out.extend(zscii_to_char((high as u16) << 5 | z as u16));
                    continue;
                }
                Pending::None => {}
            }
            let current = alphabet;
            // §3.2.3: in version 3 a shift lasts for one character.
            alphabet = 0;
            match z {
                0 => out.push(' '),
                1..=3 => pending = Pending::Abbreviation(z),
                4 => alphabet = 1,
                5 => alphabet = 2,
                _ => match current {
                    0 => out.push((b'a' + z - 6) as char),
                    1 => out.push((b'A' + z - 6) as char),
                    _ => match z {
                        6 => pending = Pending::ZsciiHigh,
                        7 => out.push('\n'),
                        _ => out.push(A2[(z - 8) as usize] as char),
                    },
                },
            }
        }
        if word & 0x8000 != 0 {
            return Ok(at);
        }
    }
}

/// Decodes the Z-string at `address`. Returns the text and the address just past
/// the string. A string that runs off the end of the file is a fault.
pub(crate) fn decode_string(
    memory: &Memory,
    address: u32,
    abbreviations: u16,
) -> Result<(String, u32), FaultKind> {
    let mut out = String::new();
    let end = decode_into(memory, address, abbreviations, 0, &mut out)?;
    Ok((out, end))
}

/// Encodes a word of ZSCII text for dictionary lookup: six Z-characters in four
/// bytes, truncated or padded with 5s (§13.3, §13.4). The word should already be in
/// lower case.
pub(crate) fn encode_dict(word: &[u8]) -> [u8; 4] {
    let mut z: Vec<u8> = Vec::with_capacity(12);
    for &c in word {
        if z.len() >= 6 {
            break;
        }
        match c {
            b' ' => z.push(0),
            b'a'..=b'z' => z.push(c - b'a' + 6),
            b'A'..=b'Z' => z.extend([4, c - b'A' + 6]),
            _ => match A2.iter().position(|&a| a == c) {
                Some(i) => z.extend([5, i as u8 + 8]),
                None => z.extend([5, 6, c >> 5, c & 0x1F]),
            },
        }
    }
    z.resize(6, 5);
    let first = (z[0] as u16) << 10 | (z[1] as u16) << 5 | z[2] as u16;
    let second = 0x8000 | (z[3] as u16) << 10 | (z[4] as u16) << 5 | z[5] as u16;
    let (a, b) = (first.to_be_bytes(), second.to_be_bytes());
    [a[0], a[1], b[0], b[1]]
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
