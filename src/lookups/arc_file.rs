use std::io;
use crate::*;

impl ArcLookup for ArcFile {
    fn get_file_info_buckets(&self) -> &[FileInfoBucket] {
        &self.file_system.file_info_buckets
    }

    fn get_file_hash_to_path_index(&self) -> &[HashToIndex] {
        &self.file_system.file_hash_to_path_index
    }

    fn get_dir_hash_to_info_index(&self) -> &[HashToIndex] {
        &self.file_system.dir_hash_to_info_index
    }

    fn get_dir_infos(&self) -> &[DirInfo] {
        &self.file_system.dir_infos
    }

    fn get_file_paths(&self) -> &[FilePath] {
        &self.file_system.file_paths
    }

    fn get_file_info_indices(&self) -> &[FileInfoIndex] {
        &self.file_system.file_info_indices
    }

    fn get_file_infos(&self) -> &[FileInfo] {
        &self.file_system.file_infos
    }

    fn get_file_info_to_datas(&self) -> &[FileInfoToFileData] {
        &self.file_system.file_info_to_datas
    }

    fn get_file_datas(&self) -> &[FileData] {
        &self.file_system.file_datas
    }

    fn get_folder_offsets(&self) -> &[DirectoryOffset] {
        &self.file_system.folder_offsets
    }

    fn get_stream_entries(&self) -> &[StreamEntry] {
        &self.file_system.stream_entries
    }

    fn get_stream_file_indices(&self) -> &[u32] {
        &self.file_system.stream_file_indices
    }

    fn get_stream_datas(&self) -> &[StreamData] {
        &self.file_system.stream_datas
    }

    fn get_stream_hash_to_entries(&self) -> &[HashToIndex] {
        &self.file_system.stream_hash_to_entries
    }

    fn get_quick_dirs(&self) -> &[QuickDir] {
        &self.file_system.quick_dirs
    }

    fn get_file_section_offset(&self) -> u64 {
        self.file_section_offset
    }

    fn get_stream_section_offset(&self) -> u64 {
        self.stream_section_offset
    }

    fn get_shared_section_offset(&self) -> u64 {
        self.shared_section_offset
    }

    fn get_file_reader<'a>(&'a self) -> Box<dyn SeekRead + 'a> {
        Box::new(MutexReader(self.reader.lock().unwrap()))
    }
}

use std::sync::MutexGuard;


// Wrapper type for implementing Read + Seek for MutexGuard
#[repr(transparent)]
struct MutexReader<'a>(MutexGuard<'a, Box<dyn SeekRead + 'static>>);

impl<'a> io::Read for MutexReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<'a> io::Seek for MutexReader<'a> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}
