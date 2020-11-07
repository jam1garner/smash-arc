#![feature(test)]

mod hash40;
mod lookups;
mod filesystem;
mod hash_labels;
mod zstd_backend;

#[cfg(feature = "smash-runtime")]
mod loaded_arc;
mod arc_file;

pub use arc_file::*;
pub use filesystem::*;
pub use hash40::{hash40, Hash40};
pub use lookups::ArcLookup;

#[repr(C)]
#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
pub enum FileNode {
    Dir(Hash40),
    File(Hash40)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
    }
}
