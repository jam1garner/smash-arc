use binread::BinRead;
use crc32fast::Hasher;
use crate::{HashToIndex, StreamEntry, QuickDir};

#[repr(transparent)]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Hash40(pub u64);

impl Hash40 {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn len(self) -> u8 {
        (self.0 >> 32) as u8
    }

    pub fn crc32(self) -> u32 {
        self.0 as u32
    }
}

impl From<&Hash40> for Hash40 {
    fn from(hash: &Hash40) -> Self {
        *hash
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

impl From<&StreamEntry> for Hash40 {
    fn from(hash_index: &StreamEntry) -> Self {
        hash_index.hash40()
    }
}

impl From<StreamEntry> for Hash40 {
    fn from(hash_index: StreamEntry) -> Self {
        hash_index.hash40()
    }
}

impl HashToIndex {
    pub fn hash40(&self) -> Hash40 {
        Hash40((self.hash() as u64) + ((self.length() as u64) << 32))
    }
}

impl StreamEntry {
    pub fn hash40(&self) -> Hash40 {
        Hash40((self.hash() as u64) + ((self.name_length() as u64) <<  32))
    }
}

impl QuickDir {
    pub fn hash40(&self) -> Hash40 {
        Hash40((self.hash() as u64) + ((self.name_length() as u64) <<  32))
    }
}


// Find the hash40 of a given string
pub fn hash40(string: &str) -> Hash40 {
    hash40_from_bytes(string.as_bytes())
}

// TODO: Is this worth adding to the public API?
pub(crate) fn hash40_from_bytes(bytes: &[u8]) -> Hash40 {
    Hash40(((bytes.len() as u64) << 32) + crc32(bytes) as u64)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use crate::{Hash40, hash40::{hash40_from_bytes, hash40}};

    #[test]
    fn hash40_path_string() {
        assert_eq!(Hash40(0x29954022ed), hash40("fighter/mario/model/body/c00/model.numatb"));
    }

    #[test]
    fn hash40_path_bytes() {
        assert_eq!(Hash40(0x29954022ed), hash40_from_bytes("fighter/mario/model/body/c00/model.numatb".as_bytes()));
    }
}
