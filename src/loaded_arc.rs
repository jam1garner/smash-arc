use std::{
    fs::File,
    io::BufReader,
};

use binread::{
    BinResult,
    BinReaderExt
};

use crate::ArcFile;
use crate::SeekRead;
pub use crate::filesystem::*;

pub struct LoadedArc {
    pub magic: u64,
    pub stream_section_offset: u64,
    pub file_section_offset: u64,
    pub shared_section_offset: u64,
    pub file_system_offset: u64,
    /// Not too sure about that one
    pub patch_section_offset: u64,
    /// Should be a FileSystem instead?
    pub loaded_filesystem: *const FileSystemHeader,
    pub loaded_filesystem_2: u64,
    /// Not too sure about that one
    pub region_entry: u64,
    pub file_info_buckets: *const FileInfoBucket,
    pub file_hash_to_path_index: *const HashToIndex,
    pub file_paths: *const FilePath,
    pub file_info_indices: *const FileInfoIndex,
    pub dir_hash_to_info_index: *const HashToIndex,
    pub dir_infos: *const DirInfo,
    pub folder_offsets: *const DirectoryOffset,
    pub folder_child_hashes: *const HashToIndex,
    pub file_infos: *const FileInfo,
    pub file_info_to_datas: *const FileInfoToFileData,
    pub file_datas: *const FileData,
    pub unk_section: u64,
    pub stream_header: *const StreamHeader,
    pub stream_unk: u64,
    pub stream_hash_to_name: u64,
    pub stream_name_to_hash: u64,
    pub stream_index_to_offset: u64,
    pub stream_offset: *const StreamOffsetEntry,
    pub extra_buckets: u64,
    pub extra_entries: u64,
    pub extra_folder_offsets: *const DirectoryOffset,
    pub extra_entry_vector: u64,
    pub version: u32,
    pub extra_count: u32,
    pub loaded_file_system_search: *const LoadedSearchSection,
    // ...
}

impl LoadedArc {
    pub fn open() -> BinResult<ArcFile> {
        Self::from_reader(BufReader::new(File::open("rom:/data.arc")?))
    }

    pub fn from_reader<R: SeekRead + 'static>(mut reader: R) -> BinResult<ArcFile> {
        let arc: ArcFile = reader.read_le()?;

        *arc.reader.lock().unwrap() = Box::new(reader);

        Ok(arc)
    }
}

pub struct SearchSectionHeader {
    pub section_size: u32,
    // ..
}

pub struct SearchSectionBody {
    pub file_info_count: u32,
    pub file_info_indices_count: u32,
    pub path_group_count: u32,
}

pub struct LoadedSearchSection {
    pub search_header: *const SearchSectionHeader,
    pub body: *const SearchSectionBody,
    // ...
}
