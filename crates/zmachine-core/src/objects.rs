//! The object table (§12).
//!
//! Callers never pass object 0: there is no such object, and the executor deals
//! with instructions that name it before reaching this module.

use crate::FaultKind;
use crate::memory::Memory;
use crate::version::Version;

type R<T> = Result<T, FaultKind>;

#[derive(Clone, Copy)]
enum Link {
    Parent,
    Sibling,
    Child,
}

pub(crate) struct Objects {
    table: u32,
    v: Version,
}

/// One property as found in a property table.
struct Property {
    number: u16,
    /// Address of the first data byte.
    data: u32,
    len: u32,
}

impl Objects {
    /// The object table of a version 3 story.
    #[cfg(test)]
    pub fn new(table: u16) -> Self {
        Self::for_version(table, Version::V3)
    }

    pub fn for_version(table: u16, v: Version) -> Self {
        Objects {
            table: table as u32,
            v,
        }
    }

    /// Address of an object's entry: 9 bytes in versions 1 to 3 (§12.3.1), 14 from
    /// version 4 (§12.3.2), after the property defaults (§12.2).
    fn entry(&self, object: u16) -> u32 {
        self.table
            + self.v.property_defaults() * 2
            + (object as u32)
                .wrapping_sub(1)
                .wrapping_mul(self.v.object_entry_size())
    }

    /// Offset of a link within an entry. The attributes come first: 4 bytes, or 6.
    fn link_offset(&self, link: Link) -> u32 {
        let n = link as u32;
        if self.v.wide_objects() {
            6 + 2 * n
        } else {
            4 + n
        }
    }

    fn link(&self, m: &Memory, object: u16, link: Link) -> R<u16> {
        let at = self.entry(object) + self.link_offset(link);
        Ok(if self.v.wide_objects() {
            m.read_word(at)?
        } else {
            m.read_byte(at)? as u16
        })
    }

    fn set_link(&self, m: &mut Memory, object: u16, link: Link, value: u16) -> R<()> {
        let at = self.entry(object) + self.link_offset(link);
        if self.v.wide_objects() {
            Ok(m.write_word(at, value)?)
        } else {
            Ok(m.write_byte(at, value as u8)?)
        }
    }

    pub fn parent(&self, m: &Memory, object: u16) -> R<u16> {
        self.link(m, object, Link::Parent)
    }

    pub fn sibling(&self, m: &Memory, object: u16) -> R<u16> {
        self.link(m, object, Link::Sibling)
    }

    pub fn child(&self, m: &Memory, object: u16) -> R<u16> {
        self.link(m, object, Link::Child)
    }

    /// Detaches an object from its parent, keeping its own children (§15
    /// remove_obj).
    pub fn remove(&self, m: &mut Memory, object: u16) -> R<()> {
        let parent = self.parent(m, object)?;
        if parent == 0 {
            return Ok(());
        }
        let next = self.sibling(m, object)?;
        let first = self.child(m, parent)?;
        if first == object {
            self.set_link(m, parent, Link::Child, next)?;
        } else {
            // Walk the chain to the sibling just before this object. The bound
            // stops a malformed chain from looping forever.
            let bound = if self.v.wide_objects() { 65536 } else { 256 };
            let mut at = first;
            for _ in 0..bound {
                if at == 0 {
                    break;
                }
                let after = self.sibling(m, at)?;
                if after == object {
                    self.set_link(m, at, Link::Sibling, next)?;
                    break;
                }
                at = after;
            }
        }
        self.set_link(m, object, Link::Parent, 0)?;
        self.set_link(m, object, Link::Sibling, 0)
    }

    /// Makes `object` the first child of `destination` (§15 insert_obj).
    pub fn insert(&self, m: &mut Memory, object: u16, destination: u16) -> R<()> {
        self.remove(m, object)?;
        let first = self.child(m, destination)?;
        self.set_link(m, object, Link::Sibling, first)?;
        self.set_link(m, object, Link::Parent, destination)?;
        self.set_link(m, destination, Link::Child, object)
    }

    /// Address and bit mask of an attribute, or `None` if the version has no such
    /// attribute: 32 of them in versions 1 to 3, 48 from version 4, numbered from
    /// the top bit (§12.3.1, §12.3.2).
    fn attribute(&self, object: u16, attribute: u16) -> Option<(u32, u8)> {
        (attribute < self.v.attribute_count()).then(|| {
            (
                self.entry(object) + attribute as u32 / 8,
                0x80 >> (attribute % 8),
            )
        })
    }

    pub fn test_attr(&self, m: &Memory, object: u16, attribute: u16) -> R<bool> {
        match self.attribute(object, attribute) {
            Some((at, mask)) => Ok(m.read_byte(at)? & mask != 0),
            None => Ok(false),
        }
    }

    pub fn set_attr(&self, m: &mut Memory, object: u16, attribute: u16) -> R<()> {
        if let Some((at, mask)) = self.attribute(object, attribute) {
            let byte = m.read_byte(at)?;
            m.write_byte(at, byte | mask)?;
        }
        Ok(())
    }

    pub fn clear_attr(&self, m: &mut Memory, object: u16, attribute: u16) -> R<()> {
        if let Some((at, mask)) = self.attribute(object, attribute) {
            let byte = m.read_byte(at)?;
            m.write_byte(at, byte & !mask)?;
        }
        Ok(())
    }

    fn property_table(&self, m: &Memory, object: u16) -> R<u32> {
        let offset = if self.v.wide_objects() { 12 } else { 7 };
        Ok(m.read_word(self.entry(object) + offset)? as u32)
    }

    /// Address of the short name's Z-string, or `None` if the object has no name
    /// (§12.4).
    pub fn name_address(&self, m: &Memory, object: u16) -> R<Option<u32>> {
        let table = self.property_table(m, object)?;
        Ok((m.read_byte(table)? != 0).then_some(table + 1))
    }

    /// The property whose size byte is at `at`, or `None` at the end of the list.
    ///
    /// Versions 1 to 3 (§12.4.1): one size byte, 32 times (length - 1) plus the
    /// number. From version 4 (§12.4.2): the number is the low six bits; if bit 7 is
    /// set a second byte holds the length in its low six bits, 0 meaning 64
    /// (§12.4.2.1.1); otherwise bit 6 says the length is 2, not 1.
    fn property_at(&self, m: &Memory, at: u32) -> R<Option<Property>> {
        let size = m.read_byte(at)?;
        if size == 0 {
            return Ok(None);
        }
        if !self.v.wide_objects() {
            return Ok(Some(Property {
                number: (size & 0x1F) as u16,
                data: at + 1,
                len: (size >> 5) as u32 + 1,
            }));
        }
        let number = (size & 0x3F) as u16;
        if size & 0x80 != 0 {
            let len = (m.read_byte(at + 1)? & 0x3F) as u32;
            Ok(Some(Property {
                number,
                data: at + 2,
                len: if len == 0 { 64 } else { len },
            }))
        } else {
            Ok(Some(Property {
                number,
                data: at + 1,
                len: if size & 0x40 != 0 { 2 } else { 1 },
            }))
        }
    }

    fn first_property(&self, m: &Memory, object: u16) -> R<Option<Property>> {
        let table = self.property_table(m, object)?;
        let name_words = m.read_byte(table)? as u32;
        self.property_at(m, table + 1 + 2 * name_words)
    }

    /// Finds a property. Properties are stored in descending numerical order.
    fn find(&self, m: &Memory, object: u16, number: u16) -> R<Option<Property>> {
        let mut next = self.first_property(m, object)?;
        while let Some(p) = next {
            if p.number == number {
                return Ok(Some(p));
            }
            if p.number < number {
                break;
            }
            next = self.property_at(m, p.data + p.len)?;
        }
        Ok(None)
    }

    /// §15 get_prop: the value, or the default if the object lacks the property.
    pub fn get_prop(&self, m: &Memory, object: u16, number: u16) -> R<u16> {
        match self.find(m, object, number)? {
            Some(p) if p.len == 1 => Ok(m.read_byte(p.data)? as u16),
            // Longer than two bytes is illegal for get_prop; like other
            // interpreters, read the first word.
            Some(p) => Ok(m.read_word(p.data)?),
            None if (1..=self.v.property_defaults()).contains(&(number as u32)) => {
                Ok(m.read_word(self.table + 2 * (number as u32 - 1))?)
            }
            None => Ok(0),
        }
    }

    /// §15 put_prop: writes the value; the object must have the property.
    pub fn put_prop(&self, m: &mut Memory, object: u16, number: u16, value: u16) -> R<()> {
        match self.find(m, object, number)? {
            Some(p) if p.len == 1 => Ok(m.write_byte(p.data, value as u8)?),
            Some(p) => Ok(m.write_word(p.data, value)?),
            None => Err(FaultKind::MissingProperty),
        }
    }

    /// §15 get_prop_addr: the address of the property data, or 0.
    pub fn get_prop_addr(&self, m: &Memory, object: u16, number: u16) -> R<u16> {
        Ok(self.find(m, object, number)?.map_or(0, |p| p.data as u16))
    }

    /// §15 get_next_prop: the property after `number`, or the first if `number` is
    /// 0; 0 when there are no more.
    pub fn get_next_prop(&self, m: &Memory, object: u16, number: u16) -> R<u16> {
        let next = if number == 0 {
            self.first_property(m, object)?
        } else {
            let p = self
                .find(m, object, number)?
                .ok_or(FaultKind::MissingProperty)?;
            self.property_at(m, p.data + p.len)?
        };
        Ok(next.map_or(0, |p| p.number))
    }

    /// §15 get_prop_len: the length of the property whose data is at `address`,
    /// worked out from the size byte just before it. Standard 1.1 defines the answer
    /// for address 0 as 0.
    pub fn prop_len(&self, m: &Memory, address: u16) -> R<u16> {
        if address == 0 {
            return Ok(0);
        }
        let size = m.read_byte(address as u32 - 1)?;
        if !self.v.wide_objects() {
            return Ok((size >> 5) as u16 + 1);
        }
        // §12.4.2.1: with bit 7 set this byte is the second size byte, whose low
        // six bits are the length.
        Ok(if size & 0x80 != 0 {
            let len = (size & 0x3F) as u16;
            if len == 0 { 64 } else { len }
        } else if size & 0x40 != 0 {
            2
        } else {
            1
        })
    }
}

/// §15 get_prop_len for a version 3 story.
#[cfg(test)]
pub(crate) fn get_prop_len(m: &Memory, address: u16) -> R<u16> {
    Objects::for_version(0, Version::V3).prop_len(m, address)
}

#[cfg(test)]
#[path = "objects_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "objects5_tests.rs"]
mod tests5;
