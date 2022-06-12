use std::{fs::File, io::BufReader, slice};

use crate::filesystem::*;
use crate::loaded_arc::{LoadedArc, LoadedSearchSection};
use crate::SeekRead;
use crate::{ArcLookup, SearchLookup};

impl ArcLookup for LoadedArc {
    fn get_file_info_buckets(&self) -> &[FileInfoBucket] {
        unsafe {
            let table_size = (*self.file_info_buckets).count;
            slice::from_raw_parts(self.file_info_buckets.offset(1), table_size as _)
        }
    }

    fn get_file_hash_to_path_index(&self) -> &[HashToIndex] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_path_count;
            slice::from_raw_parts(self.file_hash_to_path_index, table_size as _)
        }
    }

    fn get_dir_hash_to_info_index(&self) -> &[HashToIndex] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.folder_count;
            slice::from_raw_parts(self.dir_hash_to_info_index, table_size as _)
        }
    }

    fn get_dir_infos(&self) -> &[DirInfo] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.folder_count;
            slice::from_raw_parts(self.dir_infos, table_size as _)
        }
    }

    fn get_dir_infos_mut(&mut self) -> &mut [DirInfo] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.folder_count;
            slice::from_raw_parts_mut(self.dir_infos, table_size as _)
        }
    }

    fn get_file_paths(&self) -> &[FilePath] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_path_count;
            slice::from_raw_parts(self.file_paths, table_size as _)
        }
    }

    fn get_file_info_indices(&self) -> &[FileInfoIndex] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_index_count;
            slice::from_raw_parts(self.file_info_indices, table_size as _)
        }
    }

    fn get_file_infos(&self) -> &[FileInfo] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_count + fs.file_data_count_2 + fs.extra_count;
            slice::from_raw_parts(self.file_infos, table_size as _)
        }
    }

    fn get_file_infos_mut(&mut self) -> &mut [FileInfo] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_count + fs.file_data_count_2 + fs.extra_count;
            slice::from_raw_parts_mut(self.file_infos, table_size as _)
        }
    }

    fn get_file_info_to_datas(&self) -> &[FileInfoToFileData] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_sub_index_count + fs.file_data_count_2 + fs.extra_count_2;
            slice::from_raw_parts(self.file_info_to_datas, table_size as _)
        }
    }

    fn get_file_info_to_datas_mut(&mut self) -> &mut [FileInfoToFileData] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_sub_index_count + fs.file_data_count_2 + fs.extra_count_2;
            slice::from_raw_parts_mut(self.file_info_to_datas, table_size as _)
        }
    }

    fn get_file_datas(&self) -> &[FileData] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_data_count + fs.file_data_count_2 + fs.extra_count;
            slice::from_raw_parts(self.file_datas, table_size as _)
        }
    }

    fn get_file_datas_mut(&mut self) -> &mut [FileData] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_data_count + fs.file_data_count_2 + fs.extra_count;
            slice::from_raw_parts_mut(self.file_datas, table_size as _)
        }
    }

    fn get_folder_offsets(&self) -> &[DirectoryOffset] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.folder_offset_count_1 + fs.folder_offset_count_2; // + fs.extra_folder;
            slice::from_raw_parts(self.folder_offsets, table_size as _)
        }
    }

    fn get_folder_offsets_mut(&mut self) -> &mut [DirectoryOffset] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.folder_offset_count_1 + fs.folder_offset_count_2; // + fs.extra_folder;
            slice::from_raw_parts_mut(self.folder_offsets, table_size as _)
        }
    }

    fn get_stream_entries(&self) -> &[StreamEntry] {
        unsafe {
            let stream = &*self.stream_header;
            let table_size = stream.stream_hash_count;
            slice::from_raw_parts(self.stream_entries, table_size as _)
        }
    }

    fn get_stream_file_indices(&self) -> &[u32] {
        unsafe {
            let stream = &*self.stream_header;
            let table_size = stream.stream_file_index_count;
            slice::from_raw_parts(self.stream_file_indices, table_size as _)
        }
    }

    fn get_stream_datas(&self) -> &[StreamData] {
        unsafe {
            let stream = &*self.stream_header;
            let table_size = stream.stream_offset_entry_count;
            slice::from_raw_parts(self.stream_datas, table_size as _)
        }
    }

    fn get_stream_hash_to_entries(&self) -> &[HashToIndex] {
        unsafe {
            let stream = &*self.stream_header;
            let table_size = stream.stream_hash_count;
            slice::from_raw_parts(self.stream_hash_to_entries, table_size as _)
        }
    }

    fn get_quick_dirs(&self) -> &[QuickDir] {
        unsafe {
            let stream = &*self.stream_header;
            let table_size = stream.quick_dir_count;
            slice::from_raw_parts(self.quick_dirs, table_size as _)
        }
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
        Box::new(BufReader::new(File::open("rom:/data.arc").unwrap()))
    }
}

impl SearchLookup for LoadedArc {
    fn get_folder_path_to_index(&self) -> &[HashToIndex] {
        unsafe { (*self.loaded_file_system_search).get_folder_path_to_index() }
    }

    fn get_folder_path_list(&self) -> &[FolderPathListEntry] {
        unsafe { (*self.loaded_file_system_search).get_folder_path_list() }
    }

    fn get_path_to_index(&self) -> &[HashToIndex] {
        unsafe { (*self.loaded_file_system_search).get_path_to_index() }
    }

    fn get_path_list_indices(&self) -> &[u32] {
        unsafe { (*self.loaded_file_system_search).get_path_list_indices() }
    }

    fn get_path_list(&self) -> &[PathListEntry] {
        unsafe { (*self.loaded_file_system_search).get_path_list() }
    }
}

impl SearchLookup for LoadedSearchSection {
    fn get_folder_path_to_index(&self) -> &[HashToIndex] {
        unsafe {
            let table_size = (*self.body).folder_path_count;
            std::slice::from_raw_parts(self.folder_path_index, table_size as usize)
        }
    }

    fn get_folder_path_list(&self) -> &[FolderPathListEntry] {
        unsafe {
            let table_size = (*self.body).folder_path_count;
            std::slice::from_raw_parts(self.folder_path_list, table_size as usize)
        }
    }

    fn get_path_to_index(&self) -> &[HashToIndex] {
        unsafe {
            let table_size = (*self.body).path_indices_count;
            std::slice::from_raw_parts(self.path_index, table_size as usize)
        }
    }

    fn get_path_list_indices(&self) -> &[u32] {
        unsafe {
            let table_size = (*self.body).path_indices_count;
            std::slice::from_raw_parts(self.path_list_indices, table_size as usize)
        }
    }

    fn get_path_list(&self) -> &[PathListEntry] {
        unsafe {
            let table_size = (*self.body).path_count;
            std::slice::from_raw_parts(self.path_list, table_size as usize)
        }
    }
}
