#![feature(test)]

use std::{
    fs::File,
    path::Path,
    sync::Mutex,
    io::BufReader,
};

use binread::{
    BinRead,
    FilePtr64,
    BinResult,
    BinReaderExt,
    io::Cursor,
};

mod hash40;
mod lookups;
mod filesystem;
mod hash_labels;

pub use filesystem::*;
pub use hash40::{hash40, Hash40};

pub trait SeekRead: std::io::Read + std::io::Seek {}
impl<R: std::io::Read + std::io::Seek> SeekRead for R {}

#[derive(BinRead)]
#[br(magic = 0xABCDEF9876543210_u64)]
pub struct Arc {
    pub stream_section_offset: u64,
    pub file_section_offset: u64,
    pub shared_section_offset: u64,

    #[br(parse_with = FilePtr64::parse)]
    #[br(map = |x: CompressedFileSystem| x.0)]
    pub file_system: FileSystem,
    pub patch_section: u64,

    #[br(calc = Mutex::new(Box::new(Cursor::new([])) as _))]
    pub reader: Mutex<Box<dyn SeekRead>>,
}

impl Arc {
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }

    pub fn from_reader<R: SeekRead + 'static>(mut reader: R) -> BinResult<Self> {
        let arc: Arc = reader.read_le()?;

        *arc.reader.lock().unwrap() = Box::new(reader);

        Ok(arc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let arc: Arc = Arc::open("/home/jam/re/ult/900/data.arc").unwrap();
    }
}
