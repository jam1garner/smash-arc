use std::{
    fs::File,
    slice,
    io::BufReader,
};

use crate::{Hash40, LoadedDirInfo, LookupError, loaded_arc::LoadedArc};
use crate::ArcLookup;
use crate::SeekRead;
use crate::filesystem::*;

impl ArcLookup for LoadedArc {
    type DirInfoType = LoadedDirInfo;

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

    fn get_dir_infos(&self) -> &[LoadedDirInfo] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.folder_count;
            slice::from_raw_parts(self.dir_infos, table_size as _)
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
            let table_size = fs.file_info_count + fs.file_data_count_2 + fs.extra_count ;
            slice::from_raw_parts(self.file_infos, table_size as _)
        }
    }

    fn get_file_infos_mut(&mut self) -> &mut [FileInfo] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_count + fs.file_data_count_2 + fs.extra_count ;
            slice::from_raw_parts_mut(self.file_infos, table_size as _)
        }
    }

    fn get_file_info_to_datas(&self) -> &[FileInfoToFileData] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_sub_index_count  + fs.file_data_count_2 + fs.extra_count_2;
            slice::from_raw_parts(self.file_info_to_datas, table_size as _)
        }
    }

    fn get_file_info_to_datas_mut(&mut self) -> &mut [FileInfoToFileData] {
        unsafe {
            let fs = *self.fs_header;
            let table_size = fs.file_info_sub_index_count  + fs.file_data_count_2 + fs.extra_count_2;
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

    fn get_dir_info_from_hash<Hash: Into<Hash40> + ?Sized>(&self, hash: Hash) -> Result<&LoadedDirInfo, LookupError> {
        fn inner(arc: &LoadedArc, hash: Hash40) -> Result<&LoadedDirInfo, LookupError> {
            let dir_hash_to_info_index = arc.get_dir_hash_to_info_index();

            let index = dir_hash_to_info_index.binary_search_by_key(&hash, |dir| dir.hash40())
                .map(|index| dir_hash_to_info_index[index].index() as usize)
                .map_err(|_| LookupError::Missing)?;

            Ok(&arc.get_dir_infos()[index])
        }

        inner(self, hash.into())
    }

    fn get_file_reader<'a>(&'a self) -> Box<dyn SeekRead + 'a> {
        Box::new(BufReader::new(File::open("rom:/data.arc").unwrap()))
    }
}
