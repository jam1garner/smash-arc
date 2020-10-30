use binread::BinRead;
use crate::{HashToIndex, DirInfo};

#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Hash40(pub u64);

impl Hash40 {
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn len(&self) -> u8 {
        (self.0 >> 32) as u8
    }

    pub fn crc32(&self) -> u32 {
        self.0 as u32
    }
}

impl From<u64> for Hash40 {
    fn from(hash: u64) -> Self {
        Hash40(hash)
    }
}

impl From<&str> for Hash40 {
    fn from(string: &str) -> Self {
        hash40(string)
    }
}

impl From<&HashToIndex> for Hash40 {
    fn from(hash_index: &HashToIndex) -> Self {
        hash_index.hash40()
    }
}

impl From<HashToIndex> for Hash40 {
    fn from(hash_index: HashToIndex) -> Self {
        hash_index.hash40()
    }
}

impl HashToIndex {
    pub fn hash40(&self) -> Hash40 {
        Hash40((self.hash() as u64) + ((self.length() as u64) << 32))
    }
}

impl DirInfo {
    pub fn path_hash40(&self) -> Hash40 {
        Hash40((self.path_hash as u64) + ((self.dir_offset_index as u64) & 0xFF) << 32)
    }
}


// Find the hash40 of a given string
pub const fn hash40(string: &str) -> Hash40 {
    let bytes = string.as_bytes();

    Hash40(((bytes.len() as u64) << 32) + crc32(bytes) as u64)
}

/// const crc32 implementation by leo60288

macro_rules! reflect {
    ($bits:expr, $value:expr) => {{
        let mut reflection = 0;
        let mut value = $value;
        let mut i = 0;

        while i < $bits {
            if (value & 0x01) == 1 {
                reflection |= 1 << (($bits - 1) - i)
            }

            value >>= 1;
            i += 1;
        }

        reflection
    }};
}

const fn make_table(poly: u32) -> [u32; 256] {
    let mut table = [0; 256];
    let top_bit = 1 << 31;
    let mut byte;

    let mut i = 0;
    while i <= 255 {
        byte = reflect!(8, i);

        let mut value = byte << 24;

        let mut j = 0;
        while j < 8 {
            if (value & top_bit) != 0 {
                value = (value << 1) ^ poly
            } else {
                value <<= 1
            }

            j += 1;
        }

        value = reflect!(32, value);

        table[i as usize] = value;

        i += 1;
    }

    table
}

const IEEE_TABLE: [u32; 256] = make_table(0x04C11DB7);

pub const fn crc32(bytes: &[u8]) -> u32 {
    let mut value = !0u32;
    let mut i = 0;
    while i < bytes.len() {
        value = (value >> 8) ^ (IEEE_TABLE[((value ^ (bytes[i] as u32)) & 0xFF) as usize]);
        i += 1;
    }

    !value
}
