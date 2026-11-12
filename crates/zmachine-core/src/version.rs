//! Everything that differs between the versions of the Z-machine this interpreter
//! runs (research R6). A story file's version is fixed when it is loaded, and each
//! difference is a question asked of this one type, so that version 8 is version 5
//! with two numbers changed and nothing about version 3 is left to chance.

/// A Z-machine version this interpreter runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Version {
    V3,
    V5,
    V8,
}

impl Version {
    /// The version for a header's first byte, if it is one this interpreter runs.
    pub fn from_byte(byte: u8) -> Option<Version> {
        match byte {
            3 => Some(Version::V3),
            5 => Some(Version::V5),
            8 => Some(Version::V8),
            _ => None,
        }
    }

    pub fn is_v3(self) -> bool {
        self == Version::V3
    }

    /// What a packed address is multiplied by (§1.2.3).
    pub fn packed_multiplier(self) -> u32 {
        match self {
            Version::V3 => 2,
            Version::V5 => 4,
            Version::V8 => 8,
        }
    }

    /// The byte address a packed address refers to.
    pub fn unpack(self, packed: u16) -> u32 {
        packed as u32 * self.packed_multiplier()
    }

    /// What the header's file-length field is multiplied by (§11.1.6).
    pub fn length_multiplier(self) -> u32 {
        match self {
            Version::V3 => 2,
            Version::V5 => 4,
            Version::V8 => 8,
        }
    }

    /// The largest story file the version allows, in bytes (§1.1.4).
    pub fn max_story_size(self) -> usize {
        match self {
            Version::V3 => 128 * 1024,
            Version::V5 => 256 * 1024,
            Version::V8 => 512 * 1024,
        }
    }

    /// Whether object numbers and links are words (§12.3.2) or bytes (§12.3.1).
    pub fn wide_objects(self) -> bool {
        !self.is_v3()
    }

    /// Size of one entry in the object table.
    pub fn object_entry_size(self) -> u32 {
        if self.wide_objects() { 14 } else { 9 }
    }

    pub fn attribute_count(self) -> u16 {
        if self.wide_objects() { 48 } else { 32 }
    }

    /// How many words of property defaults precede the object entries (§12.2).
    pub fn property_defaults(self) -> u32 {
        if self.wide_objects() { 63 } else { 31 }
    }

    /// Z-characters in a dictionary word's encoded text.
    pub fn dictionary_zchars(self) -> usize {
        if self.is_v3() { 6 } else { 9 }
    }

    /// Whether a routine's header lists initial values for its locals (§5.2.1).
    pub fn routines_have_initial_values(self) -> bool {
        self.is_v3()
    }
}

#[cfg(test)]
#[path = "version_tests.rs"]
mod tests;
