use std::ops::Range;
use std::io::{self, SeekFrom, Read, Seek};
use crate::*;

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

mod arc_file;
#[cfg(feature = "smash-runtime")]
mod loaded_arc;

pub trait ArcLookup {
    fn get_file_info_buckets(&self) -> &[FileInfoBucket];
    fn get_file_hash_to_path_index(&self) -> &[HashToIndex];
    fn get_dir_hash_to_info_index(&self) -> &[HashToIndex];
    fn get_dir_infos(&self) -> &[DirInfo];
    fn get_file_paths(&self) -> &[FilePath];
    fn get_file_info_indices(&self) -> &[FileInfoIndex];
    fn get_file_infos(&self) -> &[FileInfo];
    fn get_file_info_to_datas(&self) -> &[FileInfoToFileData];
    fn get_file_datas(&self) -> &[FileData];
    fn get_folder_offsets(&self) -> &[DirectoryOffset];

    fn get_stream_entries(&self) -> &[StreamEntry];
    fn get_stream_file_indices(&self) -> &[u32];
    fn get_stream_datas(&self) -> &[StreamData];

    fn get_file_reader<'a>(&'a self) -> Box<dyn SeekRead + 'a>;
    fn get_file_section_offset(&self) -> u64;
    fn get_stream_section_offset(&self) -> u64;
    
    fn get_file_contents<Hash: Into<Hash40>>(&self, hash: Hash) -> Result<Vec<u8>, LookupError> {
        let hash = hash.into();

        self.get_nonstream_file_contents(hash)
            .or_else(|err| match err {
                LookupError::Missing => self.get_stream_file_contents(hash),
                err => Err(err),
            })
    }

    fn get_nonstream_file_contents<Hash: Into<Hash40>>(&self, hash: Hash) -> Result<Vec<u8>, LookupError> {
        fn inner<Arc: ArcLookup + ?Sized>(arc: &Arc, hash: Hash40) -> Result<Vec<u8>, LookupError> {
            let file_info = arc.get_file_info_from_hash(hash)?;
            let folder_offset = arc.get_folder_offset(file_info);
            let file_data = arc.get_file_data(file_info);

            arc.read_file_data(&file_data, folder_offset)
        }

        inner(self, hash.into())
    }

    fn get_stream_file_contents<Hash: Into<Hash40>>(&self, hash: Hash) -> Result<Vec<u8>, LookupError> {
        fn inner<Arc: ArcLookup + ?Sized>(arc: &Arc, hash: Hash40) -> Result<Vec<u8>, LookupError> {
            let stream_entries = arc.get_stream_entries();

            let index = stream_entries.iter()
                .find(|entry| entry.hash40() == hash)
                .map(|entry| entry.index() as usize)
                .ok_or(LookupError::Missing)?;
            
            let index = arc.get_stream_file_indices()[index] as usize;
            let file_data = &arc.get_stream_datas()[index];
            
            arc.get_stream_file_data(file_data)
        }

        inner(self, hash.into())
    }

    fn get_stream_file_data(&self, file_data: &StreamData) -> Result<Vec<u8>, LookupError> {
        let offset = file_data.offset + self.get_stream_section_offset();

        let mut reader = self.get_file_reader();
        reader.seek(SeekFrom::Start(offset))?;
        
        let mut data = Vec::with_capacity(file_data.size as usize);
        let mut reader = Read::take(&mut reader, file_data.size as u64);
        
        if reader.read_to_end(&mut data)? as u64 == file_data.size {
            Ok(data)
        } else {
            Err(LookupError::FileRead(io::Error::new(io::ErrorKind::UnexpectedEof, "Failed to read data")))
        }
    }

    fn get_bucket_for_hash(&self, hash: Hash40) -> &[HashToIndex] {
        let file_info_buckets = self.get_file_info_buckets();
        let bucket_index = (hash.as_u64() % (file_info_buckets.len() as u64)) as usize;
        let bucket = &file_info_buckets[bucket_index];
        let bucket = &self.get_file_hash_to_path_index()[bucket.range()];

        bucket
    }

    fn get_file_info_from_hash(&self, hash: Hash40) -> Result<&FileInfo, LookupError> {
        let bucket = self.get_bucket_for_hash(hash);
        
        let index_in_bucket = bucket.binary_search_by_key(&hash, |group| group.hash40())
            .map_err(|_| LookupError::Missing)?;

        let path_index = bucket[index_in_bucket].index();
        let file_info = self.get_file_info_from_path_index(path_index);
        
        Ok(file_info)
    }

    fn get_dir_info_from_hash<Hash: Into<Hash40>>(&self, hash: Hash) -> Result<&DirInfo, LookupError> {
        fn inner<Arc: ArcLookup + ?Sized>(arc: &Arc, hash: Hash40) -> Result<&DirInfo, LookupError> {
            let dir_hash_to_info_index = arc.get_dir_hash_to_info_index();

            let index = dir_hash_to_info_index.binary_search_by_key(&hash, |dir| dir.hash40())
                .map(|index| dir_hash_to_info_index[index].index() as usize)
                .map_err(|_| LookupError::Missing)?;

            Ok(&arc.get_dir_infos()[index])
        }

        inner(self, hash.into())
    }

    fn get_file_info_from_path_index(&self, path_index: u32) -> &FileInfo {
        let index = self.get_file_paths()[path_index as usize].path.index() as usize;
        let index = self.get_file_info_indices()[index].file_info_index as usize;
        let file_info = &self.get_file_infos()[index];

        file_info
    }

    fn get_file_in_folder(&self, file_info: &FileInfo) -> FileInfoToFileData {
        if file_info.flags.is_regional() {
            self.get_file_info_to_datas()[file_info.info_to_data_index as usize + 2]
        } else {
            self.get_file_info_to_datas()[file_info.info_to_data_index as usize]
        }
    }

    fn get_file_data(&self, file_info: &FileInfo) -> &FileData {
        let file_in_folder = self.get_file_in_folder(file_info);

        &self.get_file_datas()[file_in_folder.file_data_index as usize]
    }

    fn get_folder_offset(&self, file_info: &FileInfo) -> u64 {
        let file_in_folder = self.get_file_in_folder(file_info);

        let folder_offset = self.get_folder_offsets()[file_in_folder.folder_offset_index as usize].offset;

        folder_offset
    }

    fn read_file_data(&self, file_data: &FileData, folder_offset: u64) -> Result<Vec<u8>, LookupError> {
        let offset = folder_offset + self.get_file_section_offset() + ((file_data.offset_in_folder as u64) <<  2);

        if file_data.flags.compressed() && !file_data.flags.use_zstd() {
            return Err(LookupError::UnsupportedCompression)
        }
        
        let mut data = Vec::with_capacity(file_data.decomp_size as usize);

        let mut reader = self.get_file_reader();
        //let mut reader = self.reader.lock().unwrap();
        reader.seek(SeekFrom::Start(offset))?;
        let mut reader = Read::take(&mut reader, file_data.comp_size as u64);

        if file_data.flags.compressed() {
            crate::zstd_backend::copy_decode(reader, &mut data)?;
        } else {
            io::copy(&mut reader, &mut data)?;
        }

        Ok(data)
    }
}

impl FileInfoBucket {
    fn range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start + self.count as usize;

        start..end
    }
}

pub enum FileNode {
    Dir(Hash40),
    File(Hash40)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_data() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        let data = arc.get_file_contents("sound/config/bgm_property.bin").unwrap();

        std::fs::write("bgm_property.bin", data).unwrap();

        //dbg!(arc.file_system.dirs.len());
    }

    #[test]
    fn test_get_stream_file() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        
        let labels = crate::hash_labels::HashLabels::from_file("/home/jam/Downloads/hashes.txt");
        dbg!(arc.file_system.stream_entries[0].hash40().label(&labels));

        let data = arc.get_file_contents("stream:/sound/bgm/bgm_a10_malrpg2_zarazarasabaku.nus3audio").unwrap();

        std::fs::write("bgm_a10_malrpg2_zarazarasabaku.nus3audio", data).unwrap();
    }

    #[test]
    fn test_get_dir() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        let dir_info = arc.get_dir_info_from_hash("fighter/mario").unwrap();

        let start = dir_info.child_dir_start_index as usize;
        let end = (dir_info.child_dir_start_index as usize) + (dir_info.child_dir_count as usize);

        let children = &arc.file_system.folder_child_hashes[start..end].iter()
            .map(|child| &arc.file_system.dir_infos[child.index() as usize])
            .collect::<Vec<_>>();
        let labels = crate::hash_labels::HashLabels::from_file("/home/jam/Downloads/hashes.txt");

        for child in children {
            eprint!("{} ", child.name.label(&labels).map(String::from).unwrap_or_else(|| format!("0x{:X}", child.name.as_u64())));
            eprintln!("{}", child.parent.label(&labels).map(String::from).unwrap_or_else(|| format!("0x{:X}", child.parent.as_u64())));
        }

        dbg!(dir_info);
    }
}
