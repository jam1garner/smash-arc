#![feature(test)]

use std::{
    fs::File,
    path::Path,
    io::BufReader,
};

use binread::{BinRead, FilePtr64, BinResult, BinReaderExt};

mod filesystem;
pub use filesystem::*;

#[derive(BinRead, Debug)]
#[br(magic = 0xABCDEF9876543210_u64)]
pub struct Arc {
    pub music_section_offset: u64,
    pub file_section_offset: u64,
    pub shared_section_offset: u64,

    #[br(parse_with = FilePtr64::parse)]
    #[br(map = |x: CompressedFileSystem| x.0)]
    pub file_system: FileSystem,
    pub patch_section: u64,
}

impl Arc {
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        let mut file = BufReader::new(File::open(path)?);
        file.read_le()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let arc: Arc = Arc::open("/home/jam/re/ult/900/data.arc").unwrap();

        dbg!(arc.file_system.quick_dirs);
    }
}
