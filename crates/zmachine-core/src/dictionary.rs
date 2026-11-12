//! The dictionary and lexical analysis (§13).

use crate::FaultKind;
use crate::memory::Memory;
use crate::text::Charset;
use crate::version::Version;

struct Layout {
    separators: Vec<u8>,
    entry_len: u32,
    /// Number of entries.
    count: u32,
    /// Entries are in numerical order of their encoded text and can be searched by
    /// halving. A negative count in the file means they are not (§13.2, `tokenise`).
    sorted: bool,
    first: u32,
}

fn layout(memory: &Memory, dictionary: u16) -> Result<Layout, FaultKind> {
    // §13.2: n, n separator codes, entry length, number of entries, entries.
    let base = dictionary as u32;
    let n = memory.read_byte(base)? as u32;
    let mut separators = Vec::with_capacity(n as usize);
    for i in 0..n {
        separators.push(memory.read_byte(base + 1 + i)?);
    }
    let count = memory.read_word(base + 2 + n)? as i16;
    Ok(Layout {
        separators,
        entry_len: memory.read_byte(base + 1 + n)? as u32,
        count: count.unsigned_abs() as u32,
        sorted: count >= 0,
        first: base + 4 + n,
    })
}

fn entry_cmp(memory: &Memory, at: u32, key: &[u8]) -> Result<std::cmp::Ordering, FaultKind> {
    for (i, &k) in key.iter().enumerate() {
        let byte = memory.read_byte(at + i as u32)?;
        if byte != k {
            return Ok(byte.cmp(&k));
        }
    }
    Ok(std::cmp::Ordering::Equal)
}

fn find(memory: &Memory, l: &Layout, key: &[u8]) -> Result<u16, FaultKind> {
    if !l.sorted {
        for i in 0..l.count {
            let at = l.first + i * l.entry_len;
            if entry_cmp(memory, at, key)?.is_eq() {
                return Ok(at as u16);
            }
        }
        return Ok(0);
    }
    let (mut low, mut high) = (0u32, l.count);
    while low < high {
        let mid = (low + high) / 2;
        let at = l.first + mid * l.entry_len;
        match entry_cmp(memory, at, key)? {
            std::cmp::Ordering::Equal => return Ok(at as u16),
            std::cmp::Ordering::Less => low = mid + 1,
            std::cmp::Ordering::Greater => high = mid,
        }
    }
    Ok(0)
}

/// The byte address of the dictionary entry with this encoded text, or 0.
#[cfg(test)]
pub(crate) fn lookup(memory: &Memory, dictionary: u16, key: [u8; 4]) -> Result<u16, FaultKind> {
    find(memory, &layout(memory, dictionary)?, &key)
}

/// As [`lookup`], for a key of any version's length.
#[cfg(test)]
pub(crate) fn lookup_key(memory: &Memory, dictionary: u16, key: &[u8]) -> Result<u16, FaultKind> {
    find(memory, &layout(memory, dictionary)?, key)
}

/// Writes a line of ZSCII text into a version 3 text buffer: from byte 1, in lower
/// case, followed by a zero byte (§15 read). The caller has already cut the line to
/// the length the buffer allows.
pub(crate) fn store_line(memory: &mut Memory, text: u16, line: &[u8]) -> Result<(), FaultKind> {
    let mut at = text as u32 + 1;
    for &c in line {
        memory.write_byte(at, c.to_ascii_lowercase())?;
        at += 1;
    }
    memory.write_byte(at, 0)?;
    Ok(())
}

/// Appends a line of ZSCII text to a version 5 text buffer: byte 1 holds the number
/// of characters, which start at byte 2; there is no terminator. Characters already
/// in the buffer are kept and the new ones follow them (§15 read). The caller has
/// already cut the line to the room the buffer has left.
pub(crate) fn append_line(memory: &mut Memory, text: u16, line: &[u8]) -> Result<(), FaultKind> {
    let base = text as u32;
    let present = memory.read_byte(base + 1)? as u32;
    for (i, &c) in line.iter().enumerate() {
        memory.write_byte(base + 2 + present + i as u32, c.to_ascii_lowercase())?;
    }
    memory.write_byte(base + 1, (present + line.len() as u32).min(255) as u8)?;
    Ok(())
}

/// What `tokenise` needs to know beyond the buffers.
pub(crate) struct Lexer<'a> {
    pub version: Version,
    pub charset: &'a Charset,
    pub dictionary: u16,
    /// Leave the parse-buffer entries of unknown words untouched (§15 `tokenise`).
    pub skip_unknown: bool,
}

impl Lexer<'_> {
    /// Splits the text buffer into words and fills in the parse buffer (§13.6, §15
    /// read). Each block is the dictionary address of the word (0 if unknown), its
    /// length in letters, and the position of its first letter in the text buffer.
    pub fn tokenise(&self, memory: &mut Memory, text: u16, parse: u16) -> Result<(), FaultKind> {
        let l = layout(memory, self.dictionary)?;

        // Version 3 text starts at byte 1 and ends with a zero; from version 5 it
        // starts at byte 2 and byte 1 holds its length.
        let first = if self.version.is_v3() { 1 } else { 2 };
        let mut line = Vec::new();
        if self.version.is_v3() {
            loop {
                let c = memory.read_byte(text as u32 + 1 + line.len() as u32)?;
                if c == 0 {
                    break;
                }
                line.push(c);
            }
        } else {
            let len = memory.read_byte(text as u32 + 1)? as u32;
            for i in 0..len {
                line.push(memory.read_byte(text as u32 + 2 + i)?);
            }
        }

        // (start index in `line`, length)
        let mut words: Vec<(usize, usize)> = Vec::new();
        let mut start = None;
        for (i, &c) in line.iter().enumerate() {
            let separator = l.separators.contains(&c);
            if c == b' ' || separator {
                if let Some(s) = start.take() {
                    words.push((s, i - s));
                }
                if separator {
                    words.push((i, 1));
                }
            } else if start.is_none() {
                start = Some(i);
            }
        }
        if let Some(s) = start {
            words.push((s, line.len() - s));
        }

        let max = memory.read_byte(parse as u32)? as usize;
        words.truncate(max);
        memory.write_byte(parse as u32 + 1, words.len() as u8)?;
        for (n, &(s, len)) in words.iter().enumerate() {
            let key = self
                .charset
                .encode(&line[s..s + len], self.version.dictionary_zchars());
            let entry = find(memory, &l, &key)?;
            if entry == 0 && self.skip_unknown {
                continue;
            }
            let at = parse as u32 + 2 + 4 * n as u32;
            memory.write_word(at, entry)?;
            memory.write_byte(at + 2, len.min(255) as u8)?;
            memory.write_byte(at + 3, (s + first).min(255) as u8)?;
        }
        Ok(())
    }
}

/// Tokenises a version 3 text buffer against a dictionary, with the default
/// character tables.
#[cfg(test)]
pub(crate) fn tokenise(
    memory: &mut Memory,
    dictionary: u16,
    text: u16,
    parse: u16,
) -> Result<(), FaultKind> {
    let lexer = Lexer {
        version: Version::V3,
        charset: &Charset::default(),
        dictionary,
        skip_unknown: false,
    };
    lexer.tokenise(memory, text, parse)
}

#[cfg(test)]
#[path = "dictionary_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "dictionary5_tests.rs"]
mod tests5;
