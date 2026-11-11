//! The object table (§12), for version 3.
//!
//! Callers never pass object 0: there is no such object, and the executor deals
//! with instructions that name it before reaching this module.

use crate::FaultKind;
use crate::memory::Memory;

type R<T> = Result<T, FaultKind>;

const DEFAULTS: u32 = 31 * 2;
const ENTRY: u32 = 9;
const PARENT: u32 = 4;
const SIBLING: u32 = 5;
const CHILD: u32 = 6;
const PROPERTIES: u32 = 7;

pub(crate) struct Objects {
    table: u32,
}

/// One property as found in a property table.
struct Property {
    number: u16,
    /// Address of the first data byte.
    data: u32,
    len: u32,
}

impl Objects {
    pub fn new(table: u16) -> Self {
        Objects {
            table: table as u32,
        }
    }

    /// Address of an object's 9-byte entry (§12.3.1).
    fn entry(&self, object: u16) -> u32 {
        self.table + DEFAULTS + (object as u32).wrapping_sub(1) * ENTRY
    }

    pub fn parent(&self, m: &Memory, object: u16) -> R<u16> {
        Ok(m.read_byte(self.entry(object) + PARENT)? as u16)
    }

    pub fn sibling(&self, m: &Memory, object: u16) -> R<u16> {
        Ok(m.read_byte(self.entry(object) + SIBLING)? as u16)
    }

    pub fn child(&self, m: &Memory, object: u16) -> R<u16> {
        Ok(m.read_byte(self.entry(object) + CHILD)? as u16)
    }

    fn set_link(&self, m: &mut Memory, object: u16, link: u32, value: u16) -> R<()> {
        Ok(m.write_byte(self.entry(object) + link, value as u8)?)
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
            self.set_link(m, parent, CHILD, next)?;
        } else {
            // Walk the chain to the sibling just before this object. The bound
            // stops a malformed chain from looping forever.
            let mut at = first;
            for _ in 0..256 {
                if at == 0 {
                    break;
                }
                let after = self.sibling(m, at)?;
                if after == object {
                    self.set_link(m, at, SIBLING, next)?;
                    break;
                }
                at = after;
            }
        }
        self.set_link(m, object, PARENT, 0)?;
        self.set_link(m, object, SIBLING, 0)
    }

    /// Makes `object` the first child of `destination` (§15 insert_obj).
    pub fn insert(&self, m: &mut Memory, object: u16, destination: u16) -> R<()> {
        self.remove(m, object)?;
        let first = self.child(m, destination)?;
        self.set_link(m, object, SIBLING, first)?;
        self.set_link(m, object, PARENT, destination)?;
        self.set_link(m, destination, CHILD, object)
    }

    /// Address and bit mask of an attribute, or `None` if version 3 has no such
    /// attribute (§12.3.1: 32 of them, numbered from the top bit).
    fn attribute(&self, object: u16, attribute: u16) -> Option<(u32, u8)> {
        (attribute < 32).then(|| {
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
        Ok(m.read_word(self.entry(object) + PROPERTIES)? as u32)
    }

    /// Address of the short name's Z-string, or `None` if the object has no name
    /// (§12.4).
    pub fn name_address(&self, m: &Memory, object: u16) -> R<Option<u32>> {
        let table = self.property_table(m, object)?;
        Ok((m.read_byte(table)? != 0).then_some(table + 1))
    }

    /// The property at `size_byte`, or `None` at the end of the list (§12.4.1).
    fn property_at(m: &Memory, size_byte: u32) -> R<Option<Property>> {
        let size = m.read_byte(size_byte)?;
        if size == 0 {
            return Ok(None);
        }
        Ok(Some(Property {
            number: (size & 0x1F) as u16,
            data: size_byte + 1,
            len: (size >> 5) as u32 + 1,
        }))
    }

    fn first_property(&self, m: &Memory, object: u16) -> R<Option<Property>> {
        let table = self.property_table(m, object)?;
        let name_words = m.read_byte(table)? as u32;
        Self::property_at(m, table + 1 + 2 * name_words)
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
            next = Self::property_at(m, p.data + p.len)?;
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
            None if (1..=31).contains(&number) => {
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
            Self::property_at(m, p.data + p.len)?
        };
        Ok(next.map_or(0, |p| p.number))
    }
}

/// §15 get_prop_len: the length of the property whose data is at `address`.
/// Standard 1.1 defines the answer for address 0 as 0.
pub(crate) fn get_prop_len(m: &Memory, address: u16) -> R<u16> {
    if address == 0 {
        return Ok(0);
    }
    Ok((m.read_byte(address as u32 - 1)? >> 5) as u16 + 1)
}

#[cfg(test)]
#[path = "objects_tests.rs"]
mod tests;
