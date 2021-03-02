//! A library for working with the data.arc file from Smash Ultimate.
//!
//! ```rust
//! use smash_arc::{ArcFile, ArcLookup, FileNode, Hash40, Region};
//!
//! // Load the hashes needed to list directories (file format restriction)
//! Hash40::set_global_labels_file("hash_labels.txt").unwrap();
//!
//! // Parse the arc from a file
//! let arc = ArcFile::open("data.arc").unwrap();
//!
//! // loop over every file in the root
//! for node in arc.get_dir_listing("/").unwrap() {
//!     match node {
//!         FileNode::Dir(dir) => {
//!             // print out name of directory
//!             println!("directory: {}", dir.global_label().unwrap());
//!         }
//!         FileNode::File(file) => {
//!             // extract file
//!             let path = file.global_label().unwrap();
//!             std::fs::write(path, arc.get_file_contents(file, Region::UsEnglish).unwrap()).unwrap();
//!         }
//!     }
//! }
//! ```
//!
//! ## Cargo Features
//!
//! * `network` (enabled by default) = Ability to parse the file over the network
//! * `dir-listing` (enabled by default) = List directories 
//! * `global-hashes` (enabled by default) = Enable a global table for cracking hashes
//! * `smash-runtime` = Enables features for running under the context of Smash Ultimate itself
//! (enable Aarch64 crc32 hardware acceleration, enable parsing the Arc from the game's memory
//! layout)
//! 
//! * ZSTD backends
//!   * `libzstd` - Recommended for use on platforms it builds for
//!   * `rust-zstd` - Increased portability (Recommended for use on switch)
//!   * `nozstd` - Provide no zstd backend, panic on ZSTD decompression

mod hash40;
mod region;
mod lookups;
mod filesystem;
mod hash_labels;
mod zstd_backend;
mod ffi_bindings;
mod table_indices;

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
pub use table_indices::*;

/// A node in the file tree, the hash of which can be used to handle lookups.
#[repr(C, u64)]
#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
pub enum FileNode {
    Dir(Hash40),
    File(Hash40),
}

/// An enum representing a Region (country and associated language) supported by Smash Ultimate.
pub use region::Region;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
    }
}
