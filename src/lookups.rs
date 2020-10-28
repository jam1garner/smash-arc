use std::ops::Range;
use std::io::{self, SeekFrom, Read};
use crate::{Arc, Hash40, FileInfoBucket, FileData, FileInfo, HashIndexGroup};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LookupError {
    #[error("failed to read the file")]
    FileRead(#[from] std::io::Error),

    #[error("unsupported compression type, only zstd is supported")]
    UnsupportedCompression,

    #[error("the requested resource could not be found")]
    Missing,
}

impl Arc {
    pub fn get_file_contents<Hash: Into<Hash40>>(&self, hash: Hash) -> Result<Vec<u8>, LookupError> {
        fn inner(arc: &Arc, hash: Hash40) -> Result<Vec<u8>, LookupError> {
            let bucket = arc.get_bucket_for_hash(hash);
            
            let index_in_bucket = bucket.binary_search_by_key(&hash, |group| group.hash40())
                .map_err(|_| LookupError::Missing)?;

            let path_index = bucket[index_in_bucket].index();
            let file_info = arc.get_file_info_from_path_index(path_index);

            let folder_offset = arc.get_folder_offset(file_info);
            let file_data = arc.get_file_data(file_info);

            arc.read_file_data(&file_data, folder_offset)
        }

        inner(self, hash.into())
    }

    pub fn get_bucket_for_hash(&self, hash: Hash40) -> &[HashIndexGroup] {
        let fs = &self.file_system;
        let bucket_index = (hash.as_u64() % (fs.file_info_buckets.len() as u64)) as usize;
        let bucket = &fs.file_info_buckets[bucket_index];
        let bucket = &fs.hash_index_groups[bucket.range()];

        bucket
    }

    pub fn get_file_info_from_path_index(&self, path_index: u32) -> &FileInfo {
        let fs = &self.file_system;
        let index = fs.file_paths[path_index as usize].path.index() as usize;
        let index = fs.file_info_indices[index].file_info_index as usize;
        let file_info = &fs.file_infos[index];

        file_info
    }

    pub fn get_file_data(&self, file_info: &FileInfo) -> &FileData {
        let fs = &self.file_system;
        let file_in_folder = fs.file_info_to_datas[file_info.info_to_data_index as usize];
        
        &fs.file_datas[file_in_folder.file_data_index as usize]
    }

    pub fn get_folder_offset(&self, file_info: &FileInfo) -> u64 {
        let fs = &self.file_system;
        let file_in_folder = fs.file_info_to_datas[file_info.info_to_data_index as usize];

        let folder_offset = fs.folder_offsets[file_in_folder.folder_offset_index as usize].offset;

        folder_offset
    }

    pub fn read_file_data(&self, file_data: &FileData, folder_offset: u64) -> Result<Vec<u8>, LookupError> {
        let offset = folder_offset + self.file_section_offset + ((file_data.offset_in_folder as u64) <<  2);
        
        if file_data.flags.compressed() {
            if file_data.flags.use_zstd() {
                let mut data = Vec::with_capacity(file_data.decomp_size as usize);
                let mut reader = self.reader.lock().unwrap();
                reader.seek(SeekFrom::Start(offset))?;
                let reader = Read::take(&mut **reader, file_data.comp_size as u64);
                zstd::stream::copy_decode(reader, &mut data)?;

                Ok(data)
            } else {
                Err(LookupError::UnsupportedCompression)
            }
        } else {
            let mut data = Vec::with_capacity(file_data.decomp_size as usize);
            let mut reader = self.reader.lock().unwrap();
            reader.seek(SeekFrom::Start(offset))?;
            let mut reader = Read::take(&mut **reader, file_data.comp_size as u64);
            
            io::copy(&mut reader, &mut data)?;

            Ok(data)
        }
    }
}

impl FileInfoBucket {
    fn range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start + self.count as usize;

        start..end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_data() {
        let arc = Arc::open("/home/jam/re/ult/900/data.arc").unwrap();
        let data = arc.get_file_contents("fighter/pickel/model/body/c00/def_pickel_001_col.nutexb").unwrap();

        //dbg!(arc.file_system.dirs.len());
        arc.file_system.dirs.iter()
            .for_each(|dir| {
                
            });
    }
}
