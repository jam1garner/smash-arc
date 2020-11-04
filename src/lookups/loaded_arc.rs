use std::io;
use std::io::BufReader;
use std::slice;
use crate::*;

impl ArcLookup for LoadedArc {
    /// Requires testing
    fn get_file_info_buckets(&self) -> &[FileInfoBucket] {
        let table_size = (*self.file_info_buckets).count;
        unsafe { slice::from_raw_parts(self.file_info_buckets.offset(1), table_size as _) }
    }

    /// Most likely incorrect
    fn get_file_hash_to_path_index(&self) -> &[HashToIndex] {
        let fs_search = *(*self.loaded_file_system_search).body;
        let table_size = fs_search.path_group_count;
        unsafe { slice::from_raw_parts(self.file_hash_to_path_index, table_size as _) }
    }

    fn get_dir_hash_to_info_index(&self) -> &[HashToIndex] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.folder_count;
        unsafe { slice::from_raw_parts(self.dir_hash_to_info_index, table_size as _) }
    }

    fn get_dir_infos(&self) -> &[DirInfo] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.folder_count;
        unsafe { slice::from_raw_parts(self.dir_infos, table_size as _) }
    }

    fn get_file_paths(&self) -> &[FilePath] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.file_info_path_count;
        unsafe { slice::from_raw_parts(self.file_paths, table_size as _) }
    }

    fn get_file_info_indices(&self) -> &[FileInfoIndex] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.file_info_index_count;
        unsafe { slice::from_raw_parts(self.file_info_indices, table_size as _) }
    }

    fn get_file_infos(&self) -> &[FileInfo] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.file_info_count + fs.sub_file_count_2 + fs.extra_count ;
        unsafe { slice::from_raw_parts(self.file_infos, table_size as _) }
    }

    fn get_file_info_to_datas(&self) -> &[FileInfoToFileData] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.file_info_sub_index_count  + fs.sub_file_count_2 + fs.extra_count_2;
        unsafe { slice::from_raw_parts(self.file_info_to_datas, table_size as _) }
    }

    fn get_file_datas(&self) -> &[FileData] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.sub_file_count + fs.sub_file_count_2 + fs.extra_count;
        unsafe { slice::from_raw_parts(self.file_datas, table_size as _) }
    }

    fn get_folder_offsets(&self) -> &[DirectoryOffset] {
        let fs = *self.loaded_filesystem;
        let table_size = fs.folder_offset_count_1 + fs.folder_offset_count_2 + fs.extra_folder;
        unsafe { slice::from_raw_parts(self.folder_offsets, table_size as _) }
    }

    fn get_file_section_offset(&self) -> u64 {
        self.file_section_offset
    }

    fn get_file_reader<'a>(&'a self) -> Box<dyn SeekRead + 'a> {
        Box::new(MutexReader(Mutex::new(Box::new(BufReader::new(File::open("rom:/data.arc").unwrap()))).lock().unwrap()))
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