use std::io::BufReader;
use std::slice;
use crate::*;

impl ArcLookup for LoadedArc {
    /// Requires testing
    fn get_file_info_buckets(&self) -> &[FileInfoBucket] {
        unsafe {
            let table_size = (*self.file_info_buckets).count;
            slice::from_raw_parts(self.file_info_buckets.offset(1), table_size as _)
        }
    }

    /// Most likely incorrect
    fn get_file_hash_to_path_index(&self) -> &[HashToIndex] {
        unsafe {
            let fs_search = &(*self.loaded_file_system_search).body;
            let table_size = (**fs_search).path_group_count;
            slice::from_raw_parts(self.file_hash_to_path_index, table_size as _)
        }
    }

    fn get_dir_hash_to_info_index(&self) -> &[HashToIndex] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.folder_count;
            slice::from_raw_parts(self.dir_hash_to_info_index, table_size as _)
        }
    }

    fn get_dir_infos(&self) -> &[DirInfo] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.folder_count;
            slice::from_raw_parts(self.dir_infos, table_size as _)
        }
    }

    fn get_file_paths(&self) -> &[FilePath] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.file_info_path_count;
            slice::from_raw_parts(self.file_paths, table_size as _)
        }
    }

    fn get_file_info_indices(&self) -> &[FileInfoIndex] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.file_info_index_count;
            slice::from_raw_parts(self.file_info_indices, table_size as _)
        }
    }

    fn get_file_infos(&self) -> &[FileInfo] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.file_info_count + fs.sub_file_count_2 + fs.extra_count ;
            slice::from_raw_parts(self.file_infos, table_size as _)
        }
    }

    fn get_file_info_to_datas(&self) -> &[FileInfoToFileData] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.file_info_sub_index_count  + fs.sub_file_count_2 + fs.extra_count_2;
            slice::from_raw_parts(self.file_info_to_datas, table_size as _)
        }
    }

    fn get_file_datas(&self) -> &[FileData] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.sub_file_count + fs.sub_file_count_2 + fs.extra_count;
            slice::from_raw_parts(self.file_datas, table_size as _)
        }
    }

    fn get_folder_offsets(&self) -> &[DirectoryOffset] {
        unsafe {
            let fs = *self.loaded_filesystem;
            let table_size = fs.folder_offset_count_1 + fs.folder_offset_count_2 + fs.extra_folder;
            slice::from_raw_parts(self.folder_offsets, table_size as _)
        }
    }

    fn get_file_section_offset(&self) -> u64 {
        self.file_section_offset
    }

    fn get_file_reader<'a>(&'a self) -> Box<dyn SeekRead + 'a> {
        Box::new(BufReader::new(File::open("rom:/data.arc").unwrap()))
    }
}
