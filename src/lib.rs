#![feature(test)]

mod hash40;
mod region;
mod lookups;
mod filesystem;
mod hash_labels;
mod zstd_backend;
mod ffi_bindings;

#[cfg(feature = "smash-runtime")]
mod loaded_arc;
mod arc_file;

#[cfg(feature = "smash-runtime")]
pub use loaded_arc::*;

pub use arc_file::*;
pub use filesystem::*;
pub use lookups::{ArcLookup, LookupError};
pub use hash40::{hash40, Hash40};
pub use hash_labels::{GLOBAL_LABELS, HashLabels};

#[repr(C, u64)]
#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
pub enum FileNode {
    Dir(Hash40),
    File(Hash40),
}

pub use region::Region;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
    }
}
