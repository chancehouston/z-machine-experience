//! The dictionary and lexical analysis (§13), for version 3.

use crate::FaultKind;
use crate::memory::Memory;
use crate::text;

struct Layout {
    separators: Vec<u8>,
    entry_len: u32,
    count: u32,
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
    Ok(Layout {
        separators,
        entry_len: memory.read_byte(base + 1 + n)? as u32,
        count: memory.read_word(base + 2 + n)? as u32,
        first: base + 4 + n,
    })
}

fn find(memory: &Memory, l: &Layout, key: [u8; 4]) -> Result<u16, FaultKind> {
    // §13.2: entries are in numerical order of their encoded text, so a binary
    // search finds a word.
    let (mut low, mut high) = (0u32, l.count);
    while low < high {
        let mid = (low + high) / 2;
        let at = l.first + mid * l.entry_len;
        let mut entry = [0u8; 4];
        for (i, b) in entry.iter_mut().enumerate() {
            *b = memory.read_byte(at + i as u32)?;
        }
        match entry.cmp(&key) {
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

/// Splits the text buffer into words and fills in the parse buffer (§13.6, §15
/// read). Each block is the dictionary address of the word (0 if unknown), its
/// length in letters, and the position of its first letter in the text buffer.
pub(crate) fn tokenise(
    memory: &mut Memory,
    dictionary: u16,
    text: u16,
    parse: u16,
) -> Result<(), FaultKind> {
    let l = layout(memory, dictionary)?;

    let mut line = Vec::new();
    loop {
        let c = memory.read_byte(text as u32 + 1 + line.len() as u32)?;
        if c == 0 {
            break;
        }
        line.push(c);
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
        let entry = find(memory, &l, text::encode_dict(&line[s..s + len]))?;
        let at = parse as u32 + 2 + 4 * n as u32;
        memory.write_word(at, entry)?;
        memory.write_byte(at + 2, len.min(255) as u8)?;
        memory.write_byte(at + 3, (s + 1).min(255) as u8)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "dictionary_tests.rs"]
mod tests;
