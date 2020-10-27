use modular_bitfield::prelude::*;

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

        reader.read_exact(&mut compressed)?;

        let compressed = Cursor::new(compressed);
        let mut decompressed = Cursor::new(zstd::decode_all(compressed)?);

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
    pub stream_hashes: Vec<Hash40>,

    #[br(count = stream_header.stream_hash_count)]
    pub stream_entries: Vec<StreamEntry>,
    
    #[br(count = stream_header.stream_file_index_count)]
    pub stream_file_indices: Vec<u32>,
    
    #[br(count = stream_header.stream_offset_entry_count)]
    pub stream_offset_entries: Vec<StreamOffsetEntry>,

    #[br(temp)]
    pub hash_index_group_count: u32,
    
    #[br(temp)]
    pub bucket_count: u32,

    #[br(count = bucket_count)]
    pub file_info_buckets: Vec<FileInfoBucket>,

    #[br(count = hash_index_group_count)]
    pub hash_index_groups: Vec<HashIndexGroup>,

    #[br(count = fs_header.file_info_path_count)]
    pub file_info_paths: Vec<FileInfoPath>,

    #[br(count = fs_header.file_info_index_count)]
    pub file_info_indices: Vec<FileInfoIndex>,
    
    #[br(count = fs_header.folder_count)]
    pub dir_hash_to_index: Vec<HashIndexGroup>,

    #[br(count = fs_header.folder_count)]
    pub dirs: Vec<DirectoryInfo>,
    
    #[br(count = fs_header.folder_offset_count_1 + fs_header.folder_offset_count_2 + fs_header.extra_folder)]
    pub folder_offsets: Vec<DirectoryOffsets>,

    #[br(count = fs_header.hash_folder_count)]
    pub folder_child_hashes: Vec<HashIndexGroup>,

    #[br(count = fs_header.file_info_count + fs_header.sub_file_count_2 + fs_header.extra_count)]
    pub file_infos: Vec<FileInfo>,

    #[br(count = fs_header.file_info_sub_index_count + fs_header.sub_file_count_2 + fs_header.extra_count_2)]
    pub file_info_sub_index: Vec<FileInfoSubIndex>,

    #[br(count = fs_header.sub_file_count + fs_header.sub_file_count_2 + fs_header.extra_count)]
    pub sub_files: Vec<SubFileInfo>,
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
#[derive(Debug, Clone, Copy)]
pub struct QuickDir {
    pub hash: u32,
    pub name_length: u8,
    pub count: B24,
    pub index: u32,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct StreamEntry {
    pub hash: u32,
    pub name_length: u8,
    pub index: B24,
    pub flags: u32,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct HashIndexGroup {
    pub hash: u32,
    pub length: u8,
    pub index: B24,
}

#[derive_binread]
#[derive(Debug, Clone, Copy)]
pub struct Hash40 {
    pub hash: u32,
    pub length: u8,

    #[br(temp)]
    padding: [u8; 3],
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfoBucket {
    pub start: u32,
    pub count: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfoPath {
    pub path: HashIndexGroup,
    pub ext: HashIndexGroup,
    pub parent: HashIndexGroup,
    pub file_name: HashIndexGroup,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfoIndex {
    pub dir_offset_index: u32,
    pub file_info_index: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct DirectoryInfo {
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
pub struct StreamOffsetEntry {
    pub size: u64,
    pub offset: u64,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct DirectoryOffsets {
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
    pub sub_file_index: u32,
    // Flags
    pub flags: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct FileInfoSubIndex {
    pub folder_offset_index: u32,
    pub sub_file_index: u32,
    pub file_info_index_and_flag: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct SubFileInfo {
    pub offset: u32,
    pub comp_size: u32,
    pub decomp_size: u32,
    pub flags: u32,
}

impl From<[u8; 0xC]> for QuickDir {
    fn from(bytes: [u8; 0xC]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<[u8; 0xC]> for StreamEntry {
    fn from(bytes: [u8; 0xC]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<[u8; 0x8]> for HashIndexGroup {
    fn from(bytes: [u8; 0x8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl BinRead for QuickDir {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        BinRead::read_options(reader, options, args).map(<[u8; 0xC]>::into)
    }
}

impl BinRead for StreamEntry {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        BinRead::read_options(reader, options, args).map(<[u8; 0xC]>::into)
    }
}

impl BinRead for HashIndexGroup {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        BinRead::read_options(reader, options, args).map(<[u8; 0x8]>::into)
    }
}
