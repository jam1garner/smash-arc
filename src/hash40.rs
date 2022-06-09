use crate::{HashToIndex, QuickDir, StreamEntry};
use binrw::BinRead;
use crc32fast::Hasher;

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

impl HashToIndex {
    pub fn hash40(&self) -> Hash40 {
        Hash40((self.hash() as u64) + ((self.length() as u64) << 32))
    }
}

impl QuickDir {
    pub fn hash40(&self) -> Hash40 {
        Hash40((self.hash() as u64) + ((self.name_length() as u64) << 32))
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

#[cfg(feature = "serialize")]
pub mod serde {
    use serde::{
        de::{Error, Unexpected, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };

    use crate::Hash40;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
    pub struct Hash40String(pub Hash40);

    struct Hash40Visitor;

    impl<'de> Visitor<'de> for Hash40Visitor {
        type Value = Hash40;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A string or u64")
        }

        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_i32(v as i32)
        }

        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_i32(v as i32)
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Err(Error::invalid_type(
                Unexpected::Signed(v as i64),
                &Hash40Visitor,
            ))
        }

        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u32(v as u32)
        }

        fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u32(v as u32)
        }

        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Err(Error::invalid_type(
                Unexpected::Unsigned(v as u64),
                &Hash40Visitor,
            ))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u64(v as u64)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Hash40(v))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v.starts_with("0x") {
                Ok(u64::from_str_radix(v.trim_start_matches("0x"), 16)
                    .map_or_else(|_| Hash40::from(v), |val| Hash40(val)))
            } else {
                Ok(Hash40::from(v))
            }
        }
    }

    impl Serialize for Hash40 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_u64(self.0)
        }
    }

    impl<'de> Deserialize<'de> for Hash40 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_u64(Hash40Visitor)
        }
    }

    impl Serialize for Hash40String {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Hash40::serialize(&self.0, serializer)
        }
    }

    impl<'de> Deserialize<'de> for Hash40String {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Self(deserializer.deserialize_any(Hash40Visitor)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        hash40::{hash40, hash40_from_bytes},
        Hash40,
    };

    #[test]
    fn hash40_path_string() {
        assert_eq!(
            Hash40(0x29954022ed),
            hash40("fighter/mario/model/body/c00/model.numatb")
        );
    }

    #[test]
    fn hash40_path_bytes() {
        assert_eq!(
            Hash40(0x29954022ed),
            hash40_from_bytes("fighter/mario/model/body/c00/model.numatb".as_bytes())
        );
    }
}

