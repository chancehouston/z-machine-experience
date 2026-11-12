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
        Bytes { view: *view, shipped: false }
    }

    pub fn shipped(view: &View<'a>) -> Self {
        Bytes { view: *view, shipped: true }
    }

    pub fn len(&self) -> usize {
        self.view.len()
    }

    pub fn byte(&self, _address: usize) -> Option<u8> {
        None
    }

    pub fn word(&self, _address: usize) -> Option<u16> {
        None
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

/// Where the story file keeps its tables, from the header (§11.1).
#[derive(Clone, Debug)]
pub(crate) struct Layout {
    pub version: u8,
    pub last_object: u16,
    pub high_memory: usize,
}

impl Layout {
    pub fn read(_b: Bytes) -> Layout {
        Layout { version: 0, last_object: 0, high_memory: 0 }
    }

    /// The byte address a packed address stands for (§1.2.3).
    pub fn unpack(&self, _packed: u16) -> usize {
        0
    }

    pub fn object(&self, _b: Bytes, _number: u16) -> Option<Node> {
        None
    }

    /// The numbers of the attributes an object has, in order.
    pub fn attributes(&self, _b: Bytes, _number: u16) -> Option<Vec<u8>> {
        None
    }

    pub fn has_attribute(&self, _b: Bytes, _number: u16, _attribute: u8) -> Option<bool> {
        None
    }

    /// An object's short name as stored; empty if it has none.
    pub fn name(&self, _b: Bytes, _number: u16) -> Option<String> {
        None
    }

    /// An object's properties in table order.
    pub fn properties(&self, _b: Bytes, _number: u16) -> Option<Vec<Property>> {
        None
    }

    /// One property: `Some(None)` if the object does not provide it.
    pub fn property(&self, _b: Bytes, _number: u16, _property: u8) -> Option<Option<Property>> {
        None
    }

    /// The Z-encoded text at a byte address (§3).
    pub fn text(&self, _b: Bytes, _address: usize) -> Option<String> {
        None
    }

    /// Global variable `index`, from 0 (§6.2).
    pub fn global(&self, _b: Bytes, _index: usize) -> Option<u16> {
        None
    }

    pub fn dictionary(&self, _b: Bytes) -> Option<Dictionary> {
        None
    }

    /// The text of the dictionary entry at an address, as a player types it.
    pub fn word(&self, _b: Bytes, _entry: usize) -> Option<String> {
        None
    }
}
