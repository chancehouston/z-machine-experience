//! The memory map (§1).
//!
//! A story file is one array of bytes. The part below the static memory base is
//! dynamic memory, the only part a game may write. Everything may be read.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MemoryError {
    /// A read at this address is not wholly inside the file.
    ReadOutsideFile(u32),
    /// A write at this address is not wholly inside dynamic memory.
    WriteOutsideDynamicMemory(u32),
}

#[derive(Clone, Debug)]
pub(crate) struct Memory {
    bytes: Vec<u8>,
    static_base: usize,
}

impl Memory {
    /// `static_base` must not exceed the length of `bytes`; the header check at
    /// load guarantees it.
    pub fn new(bytes: Vec<u8>, static_base: usize) -> Self {
        debug_assert!(static_base <= bytes.len());
        Memory { bytes, static_base }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// The whole of memory as it stands.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn static_base(&self) -> usize {
        self.static_base
    }

    pub fn read_byte(&self, address: u32) -> Result<u8, MemoryError> {
        self.bytes
            .get(address as usize)
            .copied()
            .ok_or(MemoryError::ReadOutsideFile(address))
    }

    /// Reads a big-endian word (§2.1).
    pub fn read_word(&self, address: u32) -> Result<u16, MemoryError> {
        let at = address as usize;
        match (
            self.bytes.get(at),
            at.checked_add(1).and_then(|n| self.bytes.get(n)),
        ) {
            (Some(&hi), Some(&lo)) => Ok(u16::from_be_bytes([hi, lo])),
            _ => Err(MemoryError::ReadOutsideFile(address)),
        }
    }

    pub fn write_byte(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        let at = address as usize;
        if at >= self.static_base {
            return Err(MemoryError::WriteOutsideDynamicMemory(address));
        }
        self.bytes[at] = value;
        Ok(())
    }

    /// Writes a big-endian word. Both bytes must be in dynamic memory, or neither
    /// is written.
    pub fn write_word(&mut self, address: u32, value: u16) -> Result<(), MemoryError> {
        let at = address as usize;
        match at.checked_add(1) {
            Some(last) if last < self.static_base => {
                self.bytes[at..=last].copy_from_slice(&value.to_be_bytes());
                Ok(())
            }
            _ => Err(MemoryError::WriteOutsideDynamicMemory(address)),
        }
    }

    /// Dynamic memory: the bytes a save records.
    pub fn dynamic(&self) -> &[u8] {
        &self.bytes[..self.static_base]
    }

    pub fn dynamic_mut(&mut self) -> &mut [u8] {
        &mut self.bytes[..self.static_base]
    }
}

#[cfg(test)]
#[path = "memory_tests.rs"]
mod tests;
