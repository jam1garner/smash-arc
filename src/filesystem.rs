use modular_bitfield::prelude::*;
use crate::Hash40;

use binread::{
    BinRead,
    derive_binread,
    ReadOptions,
    io::*,
    BinResult
};

#[derive(BinRead, Debug, Clone, Copy)]
#[br(magic = 0x10_u32)]
pub struct CompTableHeader {
    pub decomp_size: u32,
    pub comp_size: u32,
    pub section_size: u32,
}

pub(crate) struct CompressedFileSystem(pub FileSystem);

impl BinRead for CompressedFileSystem {
    type Args = ();

    fn read_options<R>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self>
        where R: Read + Seek,
    {
        let header = CompTableHeader::read_options(reader, options, args)?;

        let mut compressed = vec![0; header.comp_size as usize];

        dbg!(header.comp_size);

        reader.read_exact(&mut compressed)?;

        let compressed = Cursor::new(compressed);
        let mut decompressed = Cursor::new(crate::zstd_backend::decode_all(compressed)?);

        FileSystem::read_options(&mut decompressed, options, args)
            .map(CompressedFileSystem)
    }
}

#[derive_binread]
#[derive(Debug)]
pub struct FileSystem {
    pub fs_header: FileSystemHeader,

    #[br(align_before = 0x100)]
    pub stream_header: StreamHeader,

    #[br(count = stream_header.quick_dir_count)]
    pub quick_dirs: Vec<QuickDir>,

    #[br(count = stream_header.stream_hash_count)]
    pub stream_hash_to_entries: Vec<HashToIndex>,

    #[br(count = stream_header.stream_hash_count)]
    pub stream_entries: Vec<StreamEntry>,
    
    #[br(count = stream_header.stream_file_index_count)]
    pub stream_file_indices: Vec<u32>,
    
    #[br(count = stream_header.stream_offset_entry_count)]
    pub stream_datas: Vec<StreamData>,

    #[br(temp)]
    pub hash_index_group_count: u32,
    
    #[br(temp)]
    pub bucket_count: u32,

    #[br(count = bucket_count)]
    pub file_info_buckets: Vec<FileInfoBucket>,

    #[br(count = hash_index_group_count)]
    pub file_hash_to_path_index: Vec<HashToIndex>,

    #[br(count = fs_header.file_info_path_count)]
    pub file_paths: Vec<FilePath>,

    #[br(count = fs_header.file_info_index_count)]
    pub file_info_indices: Vec<FileInfoIndex>,
    
    #[br(count = fs_header.folder_count)]
    pub dir_hash_to_info_index: Vec<HashToIndex>,

    #[br(count = fs_header.folder_count)]
    pub dir_infos: Vec<DirInfo>,
    
    #[br(count = fs_header.folder_offset_count_1 + fs_header.folder_offset_count_2 + fs_header.extra_folder)]
    pub folder_offsets: Vec<DirectoryOffset>,

    #[br(count = fs_header.hash_folder_count)]
    pub folder_child_hashes: Vec<HashToIndex>,

    #[br(count = fs_header.file_info_count + fs_header.sub_file_count_2 + fs_header.extra_count)]
    pub file_infos: Vec<FileInfo>,

    #[br(count = fs_header.file_info_sub_index_count + fs_header.sub_file_count_2 + fs_header.extra_count_2)]
    pub file_info_to_datas: Vec<FileInfoToFileData>,

    #[br(count = fs_header.sub_file_count + fs_header.sub_file_count_2 + fs_header.extra_count)]
    pub file_datas: Vec<FileData>,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileSystemHeader {
    pub table_filesize: u32,
    pub file_info_path_count: u32,
    pub file_info_index_count: u32,
    pub folder_count: u32,

    pub folder_offset_count_1: u32,

    pub hash_folder_count: u32,
    pub file_info_count: u32,
    pub file_info_sub_index_count: u32,
    pub sub_file_count: u32,

    pub folder_offset_count_2: u32,
    pub sub_file_count_2: u32,
    pub padding: u32,

    pub unk1_10: u32, // always 0x10
    pub unk2_10: u32, // always 0x10

    pub regional_count_1: u8,
    pub regional_count_2: u8,
    pub padding2: u16,
    
    pub version: u32,
    pub extra_folder: u32,
    pub extra_count: u32,

    pub unk: [u32; 2],

    pub extra_count_2: u32,
    pub extra_sub_count: u32,
}

#[derive(BinRead, Debug)]
pub struct StreamHeader {
    pub quick_dir_count: u32,
    pub stream_hash_count: u32,
    pub stream_file_index_count: u32,
    pub stream_offset_entry_count: u32,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Copy)]
#[br(map = Self::from_bytes)]
pub struct QuickDir {
    pub hash: u32,
    pub name_length: u8,
    pub count: B24,
    pub index: u32,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Copy)]
#[br(map = Self::from_bytes)]
pub struct StreamEntry {
    pub hash: u32,
    pub name_length: u8,
    pub index: B24,
    pub flags: u32,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Copy)]
#[br(map = Self::from_bytes)]
pub struct HashToIndex {
    pub hash: u32,
    pub length: u8,
    pub index: B24,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfoBucket {
    pub start: u32,
    pub count: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FilePath {
    pub path: HashToIndex,
    pub ext: HashToIndex,
    pub parent: HashToIndex,
    pub file_name: HashToIndex,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfoIndex {
    pub dir_offset_index: u32,
    pub file_info_index: u32,
}

#[derive(BinRead, Debug, Clone)]
pub struct DirInfo {
    pub path_hash: u32,
    pub dir_offset_index: u32,
    pub name: Hash40,
    pub parent: Hash40,
    pub extra_dis_re: u32,
    pub extra_dis_re_length: u32,
    pub file_name_start_index: u32,
    pub file_info_count: u32,
    pub child_dir_start_index: u32,
    pub child_dir_count: u32,
    pub flags: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct StreamData {
    pub size: u64,
    pub offset: u64,
}

#[derive(BinRead, Debug, Clone)]
pub struct DirectoryOffset {
    pub offset: u64,
    pub decomp_size: u32,
    pub size: u32,
    pub sub_data_start_index: u32,
    pub sub_data_count: u32,
    pub resource_index: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfo {
    // PathIndex
    pub hash_index: u32,
    // IndexIndex
    pub hash_index_2: u32,
    // SubIndexIndex
    pub info_to_data_index: u32,
    // Flags
    pub flags: FileInfoFlags,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[br(map = Self::from_bytes)]
pub struct FileInfoFlags {
    pub unused: B4,
    pub is_redirect: bool,
    pub unused2: B7,
    pub unknown1: bool,
    pub padding3: B2,
    pub is_regional: bool,
    pub is_localized: bool,
    pub unused3: B3,
    pub unknown2: bool,
    pub unknown3: bool,
    pub unused4: B10,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfoToFileData {
    pub folder_offset_index: u32,
    pub file_data_index: u32,
    pub file_info_index_and_flag: u32,
}

#[repr(C)]
#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileData {
    pub offset_in_folder: u32,
    pub comp_size: u32,
    pub decomp_size: u32,
    pub flags: FileDataFlags,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[br(map = Self::from_bytes)]
pub struct FileDataFlags {
    pub compressed: bool,
    pub use_zstd: bool,
    pub unk: B30,
}
