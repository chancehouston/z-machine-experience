//! A read-only view of the machine, for the introspection interface
//! (docs/introspection.md; specs/003-introspection/contracts/rust-api.md).

/// The story file as it is now and as it was loaded. A `View` borrows the machine
/// and has no way to change it.
#[derive(Clone, Copy, Debug)]
pub struct View<'a> {
    pub(crate) version: u8,
    pub(crate) now: &'a [u8],
    pub(crate) shipped: &'a [u8],
}

impl View<'_> {
    /// The Z-machine version of the story file: 3, 5 or 8.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// The number of bytes in the story file.
    pub fn len(&self) -> usize {
        self.now.len()
    }

    pub fn is_empty(&self) -> bool {
        self.now.is_empty()
    }

    /// The byte at an address as memory stands now, or `None` outside the file.
    pub fn byte(&self, address: usize) -> Option<u8> {
        self.now.get(address).copied()
    }

    /// The big-endian word at an address as memory stands now.
    pub fn word(&self, address: usize) -> Option<u16> {
        word(self.now, address)
    }

    /// The byte at an address in the story file as it was loaded.
    pub fn shipped_byte(&self, address: usize) -> Option<u8> {
        self.shipped.get(address).copied()
    }

    /// The word at an address in the story file as it was loaded.
    pub fn shipped_word(&self, address: usize) -> Option<u16> {
        word(self.shipped, address)
    }
}

fn word(bytes: &[u8], address: usize) -> Option<u16> {
    let high = *bytes.get(address)?;
    let low = *bytes.get(address.checked_add(1)?)?;
    Some(u16::from_be_bytes([high, low]))
}
