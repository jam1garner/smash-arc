use std::convert::{TryInto, TryFrom};

use binread::BinRead;

#[repr(transparent)]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilePathIdx(u32);

impl From<FilePathIdx> for usize {
    fn from(index: FilePathIdx) -> Self {
        index.0 as usize
    }
}

impl From<u32> for FilePathIdx {
    fn from(index: u32) -> Self {
        FilePathIdx(index)
    }
}

impl From<usize> for FilePathIdx {
    fn from(index: usize) -> Self {
        FilePathIdx(index as u32)
    }
}

#[repr(transparent)]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileInfoIdx(u32);

impl From<FileInfoIdx> for usize {
    fn from(index: FileInfoIdx) -> Self {
        index.0 as usize
    }
}

impl From<u32> for FileInfoIdx {
    fn from(index: u32) -> Self {
        FileInfoIdx(index)
    }
}

impl From<usize> for FileInfoIdx {
    fn from(index: usize) -> Self {
        FileInfoIdx(index as u32)
    }
}

#[repr(transparent)]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileInfoIndiceIdx(pub u32);

impl From<FileInfoIndiceIdx> for usize {
    fn from(index: FileInfoIndiceIdx) -> Self {
        index.0 as usize
    }
}

impl From<u32> for FileInfoIndiceIdx {
    fn from(index: u32) -> Self {
        FileInfoIndiceIdx(index)
    }
}

impl From<usize> for FileInfoIndiceIdx {
    fn from(index: usize) -> Self {
        FileInfoIndiceIdx(index as u32)
    }
}

#[repr(transparent)]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InfoToDataIdx(u32);

impl From<InfoToDataIdx> for usize {
    fn from(index: InfoToDataIdx) -> Self {
        index.0 as usize
    }
}

impl From<u32> for InfoToDataIdx {
    fn from(index: u32) -> Self {
        InfoToDataIdx(index)
    }
}

impl From<usize> for InfoToDataIdx {
    fn from(index: usize) -> Self {
        InfoToDataIdx(index as u32)
    }
}

#[repr(transparent)]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDataIdx(pub u32);

impl From<FileDataIdx> for usize {
    fn from(index: FileDataIdx) -> Self {
        index.0 as usize
    }
}

impl From<u32> for FileDataIdx {
    fn from(index: u32) -> Self {
        FileDataIdx(index)
    }
}

impl From<usize> for FileDataIdx {
    fn from(index: usize) -> Self {
        FileDataIdx(index as u32)
    }
}