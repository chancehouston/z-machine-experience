//! The Z-machine's own tables, read from bytes: objects (Standard §12), the
//! dictionary (§13), text (§3) and the globals (§6.2).
//!
//! Every reader returns `None` when what it is asked for is not wholly inside the
//! story file or does not end. None panics and none loops without bound.

use zmachine_core::View;

/// One of the two things a view shows: memory now, or the file as shipped.
#[derive(Clone, Copy)]
pub(crate) struct Bytes<'a> {
    view: View<'a>,
    shipped: bool,
}

impl<'a> Bytes<'a> {
    pub fn now(view: &View<'a>) -> Self {
        Bytes {
            view: *view,
            shipped: false,
        }
    }

    pub fn shipped(view: &View<'a>) -> Self {
        Bytes {
            view: *view,
            shipped: true,
        }
    }

    pub fn len(&self) -> usize {
        self.view.len()
    }

    pub fn byte(&self, address: usize) -> Option<u8> {
        match self.shipped {
            true => self.view.shipped_byte(address),
            false => self.view.byte(address),
        }
    }

    pub fn word(&self, address: usize) -> Option<u16> {
        match self.shipped {
            true => self.view.shipped_word(address),
            false => self.view.word(address),
        }
    }
}

/// An object's links (§12.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    pub parent: u16,
    pub sibling: u16,
    pub child: u16,
    /// Address of its property table.
    pub properties: usize,
}

/// One property of an object (§12.4): where its data is and how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Property {
    pub number: u8,
    pub at: usize,
    pub len: usize,
}

/// The dictionary's shape (§13.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Dictionary {
    /// Address of the first entry.
    pub first: usize,
    pub entry_len: usize,
    pub count: usize,
    /// Bytes of encoded text at the start of each entry: 4 or 6.
    pub text_len: usize,
}

/// The three alphabets of §3.5.3, as the default table has them.
const ALPHABETS: [&[u8; 26]; 3] = [
    b"abcdefghijklmnopqrstuvwxyz",
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    b" \n0123456789.,!?_#'\"/\\-:()",
];

/// The default meanings of ZSCII 155 to 223 (§3.8.5.3).
const EXTRA: &str = "äöüÄÖÜß»«ëïÿËÏáéíóúýÁÉÍÓÚÝàèìòùÀÈÌÒÙâêîôûÂÊÎÔÛåÅøØãñõÃÑÕæÆçÇþðÞÐ£œŒ¡¿";

/// The longest text the readers will decode, in Z-characters. No story file's
/// text approaches it; it bounds the work a damaged one can cause.
const MAX_ZCHARS: usize = 60_000;

/// Where the story file keeps its tables, from the header (§11.1).
#[derive(Clone, Debug)]
pub(crate) struct Layout {
    pub version: u8,
    pub last_object: u16,
    pub high_memory: usize,
    /// Address of the first object entry, after the property defaults.
    tree: usize,
    entry: usize,
    globals: usize,
    dictionary: usize,
    abbreviations: usize,
    /// Address of the game's own alphabet table, if it has one (§3.5.5).
    alphabet: Option<usize>,
    /// The game's own Unicode table, if it has one (§3.8.5.2).
    unicode: Option<Vec<char>>,
}

impl Layout {
    pub fn read(b: Bytes) -> Layout {
        let version = b.byte(0).unwrap_or(0);
        let word = |at: usize| b.word(at).unwrap_or(0) as usize;
        let wide = version > 3;
        let (defaults, entry) = if wide { (63, 14) } else { (31, 9) };
        let tree = word(0x0A) + 2 * defaults;
        let mut layout = Layout {
            version,
            last_object: 0,
            high_memory: word(0x04),
            tree,
            entry,
            globals: word(0x0C),
            dictionary: word(0x08),
            abbreviations: word(0x18),
            alphabet: Some(word(0x34)).filter(|&a| wide && a != 0),
            unicode: None,
        };
        layout.last_object = layout.count_objects(b);
        if wide {
            layout.unicode = read_unicode_table(b, word(0x36));
        }
        layout
    }

    /// §12.3: the entries run up to the first property table.
    fn count_objects(&self, b: Bytes) -> u16 {
        let limit = if self.version > 3 { 0xFFFF } else { 255 };
        let mut lowest = b.len();
        let mut count = 0usize;
        while count < limit {
            let at = self.tree + count * self.entry;
            if at + self.entry > lowest {
                break;
            }
            let Some(table) = b.word(at + self.entry - 2) else {
                break;
            };
            // A property table cannot lie inside the entries already counted.
            if table as usize > at {
                lowest = lowest.min(table as usize);
            }
            count += 1;
        }
        count as u16
    }

    /// The byte address a packed address stands for (§1.2.3).
    pub fn unpack(&self, packed: u16) -> usize {
        packed as usize
            * match self.version {
                8 => 8,
                4..=7 => 4,
                _ => 2,
            }
    }

    fn entry_at(&self, number: u16) -> Option<usize> {
        (number >= 1 && number <= self.last_object)
            .then(|| self.tree + (number as usize - 1) * self.entry)
    }

    pub fn object(&self, b: Bytes, number: u16) -> Option<Node> {
        let at = self.entry_at(number)?;
        if self.version > 3 {
            Some(Node {
                parent: b.word(at + 6)?,
                sibling: b.word(at + 8)?,
                child: b.word(at + 10)?,
                properties: b.word(at + 12)? as usize,
            })
        } else {
            Some(Node {
                parent: b.byte(at + 4)? as u16,
                sibling: b.byte(at + 5)? as u16,
                child: b.byte(at + 6)? as u16,
                properties: b.word(at + 7)? as usize,
            })
        }
    }

    /// An object's children in the tree's order: its child, then each sibling.
    /// `None` if a link is out of range, the chain is longer than there are
    /// objects, or a child names some other parent.
    pub fn children(&self, b: Bytes, number: u16) -> Option<Vec<u16>> {
        let mut children = Vec::new();
        let mut next = self.object(b, number)?.child;
        while next != 0 {
            let child = self.object(b, next)?;
            if child.parent != number || children.len() >= self.last_object as usize {
                return None;
            }
            children.push(next);
            next = child.sibling;
        }
        Some(children)
    }

    /// The bytes of a property.
    pub fn data(&self, b: Bytes, property: &Property) -> Option<Vec<u8>> {
        (0..property.len).map(|i| b.byte(property.at + i)).collect()
    }

    /// The value of a property that is exactly two bytes long.
    pub fn property_word(&self, b: Bytes, number: u16, property: u8) -> Option<u16> {
        let p = self.property(b, number, property)??;
        if p.len != 2 {
            return None;
        }
        b.word(p.at)
    }

    fn attribute_bytes(&self) -> usize {
        if self.version > 3 { 6 } else { 4 }
    }

    /// The numbers of the attributes an object has, in order.
    pub fn attributes(&self, b: Bytes, number: u16) -> Option<Vec<u8>> {
        let at = self.entry_at(number)?;
        let mut set = Vec::new();
        for i in 0..self.attribute_bytes() {
            let byte = b.byte(at + i)?;
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    set.push((i * 8 + bit) as u8);
                }
            }
        }
        Some(set)
    }

    pub fn has_attribute(&self, b: Bytes, number: u16, attribute: u8) -> Option<bool> {
        let at = self.entry_at(number)?;
        let index = attribute as usize / 8;
        if index >= self.attribute_bytes() {
            return None;
        }
        Some(b.byte(at + index)? & (0x80 >> (attribute % 8)) != 0)
    }

    /// An object's short name as stored; empty if it has none.
    pub fn name(&self, b: Bytes, number: u16) -> Option<String> {
        let table = self.object(b, number)?.properties;
        match b.byte(table)? {
            0 => Some(String::new()),
            _ => self.text(b, table + 1),
        }
    }

    /// An object's properties in table order.
    pub fn properties(&self, b: Bytes, number: u16) -> Option<Vec<Property>> {
        let table = self.object(b, number)?.properties;
        let mut at = table + 1 + 2 * b.byte(table)? as usize;
        let mut found = Vec::new();
        // A table holds each property at most once.
        for _ in 0..64 {
            let size = b.byte(at)?;
            if size == 0 {
                return Some(found);
            }
            let (number, len, data) = if self.version <= 3 {
                (size & 0x1F, (size >> 5) as usize + 1, at + 1)
            } else if size & 0x80 != 0 {
                // §12.4.2.1.1: a length of 0 means 64.
                let len = (b.byte(at + 1)? & 0x3F) as usize;
                (size & 0x3F, if len == 0 { 64 } else { len }, at + 2)
            } else {
                (size & 0x3F, if size & 0x40 != 0 { 2 } else { 1 }, at + 1)
            };
            // The data must be inside the file.
            b.byte((data + len).checked_sub(1)?)?;
            found.push(Property {
                number,
                at: data,
                len,
            });
            at = data + len;
        }
        None
    }

    /// One property: `Some(None)` if the object does not provide it.
    pub fn property(&self, b: Bytes, number: u16, property: u8) -> Option<Option<Property>> {
        Some(
            self.properties(b, number)?
                .into_iter()
                .find(|p| p.number == property),
        )
    }

    /// Global variable `index`, from 0 (§6.2).
    pub fn global(&self, b: Bytes, index: usize) -> Option<u16> {
        if index >= 240 {
            return None;
        }
        b.word(self.globals + 2 * index)
    }

    pub fn dictionary(&self, b: Bytes) -> Option<Dictionary> {
        let separators = b.byte(self.dictionary)? as usize;
        let at = self.dictionary + 1 + separators;
        let entry_len = b.byte(at)? as usize;
        // §13.2: a negative count marks an unsorted dictionary of that many entries.
        let count = (b.word(at + 1)? as i16).unsigned_abs() as usize;
        let text_len = if self.version > 3 { 6 } else { 4 };
        if entry_len < text_len {
            return None;
        }
        let first = at + 3;
        // The whole table must be inside the file.
        if count > 0 {
            b.byte(first + count * entry_len - 1)?;
        }
        Some(Dictionary {
            first,
            entry_len,
            count,
            text_len,
        })
    }

    /// The Z-encoded text at a byte address (§3).
    pub fn text(&self, b: Bytes, address: usize) -> Option<String> {
        let mut zchars = Vec::new();
        let mut at = address;
        loop {
            let word = b.word(at)?;
            zchars.extend([(word >> 10) & 31, (word >> 5) & 31, word & 31]);
            if word & 0x8000 != 0 {
                break;
            }
            if zchars.len() > MAX_ZCHARS {
                return None;
            }
            at = at.checked_add(2)?;
        }
        let mut out = String::new();
        self.decode(b, &zchars, true, &mut out)?;
        Some(out)
    }

    /// The text of the dictionary entry at an address, as a player types it.
    pub fn word(&self, b: Bytes, entry: usize) -> Option<String> {
        let text_len = if self.version > 3 { 6 } else { 4 };
        let mut zchars = Vec::new();
        for i in (0..text_len).step_by(2) {
            let word = b.word(entry + i)?;
            zchars.extend([(word >> 10) & 31, (word >> 5) & 31, word & 31]);
        }
        let mut out = String::new();
        self.decode(b, &zchars, false, &mut out)?;
        Some(out)
    }

    fn alphabet_char(&self, b: Bytes, alphabet: usize, zchar: u16) -> Option<char> {
        let index = zchar as usize - 6;
        // §3.5.5.1: in A2, character 7 is a new-line whatever the table says.
        if alphabet == 2 && zchar == 7 {
            return Some('\n');
        }
        let code = match self.alphabet {
            Some(table) => b.byte(table + alphabet * 26 + index)?,
            None => ALPHABETS[alphabet][index],
        };
        self.zscii(code as u16)
    }

    /// The character a ZSCII code prints as (§3.8), or `None` for a code that
    /// prints nothing.
    fn zscii(&self, code: u16) -> Option<char> {
        match code {
            13 => Some('\n'),
            32..=126 => Some(code as u8 as char),
            155..=251 => {
                let index = code as usize - 155;
                Some(match &self.unicode {
                    Some(table) => table.get(index).copied().unwrap_or('?'),
                    None => EXTRA.chars().nth(index).unwrap_or('?'),
                })
            }
            _ => None,
        }
    }

    /// Turns Z-characters into text (§3.2 to §3.5). `abbreviations` is false inside
    /// an abbreviation (§3.3.1) and for dictionary words.
    fn decode(
        &self,
        b: Bytes,
        zchars: &[u16],
        abbreviations: bool,
        out: &mut String,
    ) -> Option<()> {
        let mut alphabet = 0;
        let mut i = 0;
        while i < zchars.len() {
            let z = zchars[i];
            i += 1;
            match z {
                0 => out.push(' '),
                1..=3 => {
                    let Some(&x) = zchars.get(i) else { break };
                    i += 1;
                    if abbreviations {
                        let entry = self.abbreviations + 2 * (32 * (z as usize - 1) + x as usize);
                        // §3.3: abbreviation strings are at word addresses.
                        let address = b.word(entry)? as usize * 2;
                        let mut inner = Vec::new();
                        let mut at = address;
                        loop {
                            let word = b.word(at)?;
                            inner.extend([(word >> 10) & 31, (word >> 5) & 31, word & 31]);
                            if word & 0x8000 != 0 {
                                break;
                            }
                            if inner.len() > MAX_ZCHARS {
                                return None;
                            }
                            at += 2;
                        }
                        self.decode(b, &inner, false, out)?;
                    }
                }
                4 | 5 => {
                    alphabet = z as usize - 3;
                    continue;
                }
                6 if alphabet == 2 => {
                    // §3.4: a ten-bit ZSCII code in the next two Z-characters.
                    let (Some(&high), Some(&low)) = (zchars.get(i), zchars.get(i + 1)) else {
                        break;
                    };
                    i += 2;
                    out.extend(self.zscii(high << 5 | low));
                }
                _ => out.extend(self.alphabet_char(b, alphabet, z)),
            }
            alphabet = 0;
            if out.len() > 4 * MAX_ZCHARS {
                return None;
            }
        }
        Some(())
    }
}

/// §3.8.5.2: the header extension table's third word points at a Unicode table.
fn read_unicode_table(b: Bytes, extension: usize) -> Option<Vec<char>> {
    if extension == 0 || b.word(extension)? < 3 {
        return None;
    }
    let table = b.word(extension + 6)? as usize;
    if table == 0 {
        return None;
    }
    let count = b.byte(table)? as usize;
    (0..count)
        .map(|i| Some(char::from_u32(b.word(table + 1 + 2 * i)? as u32).unwrap_or('?')))
        .collect()
}
