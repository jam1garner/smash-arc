use crate::*;
use std::ops::Range;
use std::io::{self, SeekFrom, Read, Seek};

use region::Region;
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


/// The trait that allows different implementations of the arc to share the same code for making
/// lookups into the filesystem use the same logic.
///
/// To implement, provide accessors for the needed data and all the lookups themselves will be
/// implemented for you.
pub trait ArcLookup {
    type DirInfoType;

    fn get_file_info_buckets(&self) -> &[FileInfoBucket];
    fn get_file_hash_to_path_index(&self) -> &[HashToIndex];
    fn get_dir_hash_to_info_index(&self) -> &[HashToIndex];
    fn get_dir_infos(&self) -> &[Self::DirInfoType];
    fn get_file_paths(&self) -> &[FilePath];
    fn get_file_info_indices(&self) -> &[FileInfoIndex];
    fn get_file_infos(&self) -> &[FileInfo];
    fn get_file_info_to_datas(&self) -> &[FileInfoToFileData];
    fn get_file_datas(&self) -> &[FileData];
    fn get_folder_offsets(&self) -> &[DirectoryOffset];

    fn get_stream_entries(&self) -> &[StreamEntry];
    fn get_stream_file_indices(&self) -> &[u32];
    fn get_stream_datas(&self) -> &[StreamData];
    fn get_quick_dirs(&self) -> &[QuickDir];
    fn get_stream_hash_to_entries(&self) -> &[HashToIndex];

    fn get_file_reader<'a>(&'a self) -> Box<dyn SeekRead + 'a>;
    fn get_file_section_offset(&self) -> u64;
    fn get_stream_section_offset(&self) -> u64;
    fn get_shared_section_offset(&self) -> u64;

    fn get_dir_info_from_hash<Hash: Into<Hash40> + ?Sized>(&self, hash: Hash) -> Result<&Self::DirInfoType, LookupError>;
    
    // mutable access
    fn get_file_infos_mut(&mut self) -> &mut [FileInfo];
    fn get_file_datas_mut(&mut self) -> &mut [FileData];
    fn get_file_info_to_datas_mut(&mut self) -> &mut [FileInfoToFileData];
    
    fn get_file_contents<Hash: Into<Hash40>>(&self, hash: Hash, region: Region) -> Result<Vec<u8>, LookupError> {
        let hash = hash.into();

        self.get_nonstream_file_contents(hash, region)
            .or_else(|err| match err {
                LookupError::Missing => self.get_stream_file_contents(hash),
                err => Err(err),
            })
    }

    fn get_nonstream_file_contents<Hash: Into<Hash40>>(&self, hash: Hash, region: Region) -> Result<Vec<u8>, LookupError> {
        fn inner<Arc: ArcLookup + ?Sized>(arc: &Arc, hash: Hash40, region: Region) -> Result<Vec<u8>, LookupError> {
            let file_info = arc.get_file_info_from_hash(hash)?;
            let folder_offset = arc.get_folder_offset(file_info, region);
            let file_data = arc.get_file_data(file_info, region);

            arc.read_file_data(&file_data, folder_offset)
        }

        inner(self, hash.into(), region)
    }

    fn get_stream_data(&self, hash: Hash40) -> Result<&StreamData, LookupError> {
        let stream_entries = self.get_stream_entries();

        let index = stream_entries.iter()
            .find(|entry| entry.hash40() == hash)
            .map(|entry| entry.index() as usize)
            .ok_or(LookupError::Missing)?;
        
        let index = self.get_stream_file_indices()[index] as usize;
        
        Ok(&self.get_stream_datas()[index])
    }

    fn get_stream_file_contents<Hash: Into<Hash40>>(&self, hash: Hash) -> Result<Vec<u8>, LookupError> {
        fn inner<Arc: ArcLookup + ?Sized>(arc: &Arc, hash: Hash40) -> Result<Vec<u8>, LookupError> {
            let file_data = arc.get_stream_data(hash)?;
            arc.read_stream_file_data(file_data)
        }

        inner(self, hash.into())
    }

    fn read_stream_file_data(&self, file_data: &StreamData) -> Result<Vec<u8>, LookupError> {
        let offset = file_data.offset;

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
    
    fn get_shared_files(&self, hash: Hash40, region: Region) -> Result<Vec<Hash40>, LookupError> {
        let metadata = self.get_file_metadata(hash, region)?;

        if metadata.is_shared {
            let hash_to_paths = self.get_file_hash_to_path_index();

            let file_data_index = self.get_file_in_folder(
                self.get_file_info_from_hash(hash)?,
                region
            ).file_data_index;

            Ok(
                hash_to_paths
                    .iter()
                    .filter_map(|hash_to_path| {
                        let hash = hash_to_path.hash40();
                        let file_info = self.get_file_info_from_hash(hash).ok()?;
                        let file_in_folder = self.get_file_in_folder(file_info, region);
                        let is_same_fd_index = file_in_folder.file_data_index == file_data_index;
                        if is_same_fd_index {
                            Some(hash)
                        } else {
                            None
                        }
                    })
                    .collect()
            )
        } else {
            Ok(Vec::from([]))
        }
    }

    fn get_bucket_for_hash(&self, hash: Hash40) -> &[HashToIndex] {
        let file_info_buckets = self.get_file_info_buckets();
        let bucket_index = (hash.as_u64() % (file_info_buckets.len() as u64)) as usize;
        let bucket = &file_info_buckets[bucket_index];
        
        &self.get_file_hash_to_path_index()[bucket.range()]
    }

    fn get_file_path_index_from_hash(&self, hash: Hash40) -> Result<FilePathIdx, LookupError> {
        let bucket = self.get_bucket_for_hash(hash);
        
        let index_in_bucket = bucket.binary_search_by_key(&hash, |group| group.hash40())
            .map_err(|_| LookupError::Missing)?;

        Ok(FilePathIdx(bucket[index_in_bucket].index()))
    }

    fn get_file_info_from_hash(&self, hash: Hash40) -> Result<&FileInfo, LookupError> {
        let path_index = self.get_file_path_index_from_hash(hash)?;
        let file_info = self.get_file_info_from_path_index(path_index);
        
        Ok(file_info)
    }

    fn get_stream_listing(&self, dir: &str) -> Result<&[StreamEntry], LookupError> {
        let hash = match dir {
            "bgm" | "smashappeal" | "movie" => crate::hash40::hash40(dir),
            dir if dir.starts_with("stream:/sound") => crate::hash40::hash40(&dir[14..]),
            "stream:/movie" => crate::hash40::hash40("movie"),
            _ => return Err(LookupError::Missing)
        };

        self.get_quick_dirs()
            .iter()
            .find(|dir| dir.hash40() == hash)
            .map(|dir| &self.get_stream_entries()[dir.range()])
            .ok_or(LookupError::Missing)
    }

    fn get_file_info_from_path_index(&self, path_index: FilePathIdx) -> &FileInfo {
        let index = self.get_file_paths()[path_index].path.index() as usize;
        let index = self.get_file_info_indices()[index].file_info_index;

        &self.get_file_infos()[index]
    }

    fn get_file_info_from_path_index_mut(&mut self, path_index: FilePathIdx) -> &mut FileInfo {
        let index = self.get_file_paths()[path_index].path.index() as usize;
        let index = self.get_file_info_indices()[index].file_info_index;

        &mut self.get_file_infos_mut()[index]
    }

    fn get_file_in_folder(&self, file_info: &FileInfo, region: Region) -> FileInfoToFileData {
        if file_info.flags.is_regional() {
            self.get_file_info_to_datas()[usize::from(file_info.info_to_data_index) + (region as usize)]
        } else {
            self.get_file_info_to_datas()[file_info.info_to_data_index]
        }
    }

    fn get_file_in_folder_mut(&mut self, file_info: &FileInfo, region: Region) -> &mut FileInfoToFileData {
        if file_info.flags.is_regional() {
            &mut self.get_file_info_to_datas_mut()[usize::from(file_info.info_to_data_index) + (region as usize)]
        } else {
            &mut self.get_file_info_to_datas_mut()[file_info.info_to_data_index]
        }
    }

    fn get_file_data_from_hash(&self, hash: Hash40, region: Region) -> Result<&FileData, LookupError> {
        Ok(self.get_file_data(self.get_file_info_from_hash(hash)?, region))
    }

    fn get_file_data(&self, file_info: &FileInfo, region: Region) -> &FileData {
        let file_in_folder = self.get_file_in_folder(file_info, region);

        &self.get_file_datas()[file_in_folder.file_data_index]
    }

    fn get_file_data_mut(&mut self, file_info: &FileInfo, region: Region) -> &mut FileData {
        let file_in_folder = self.get_file_in_folder(file_info, region);

        &mut self.get_file_datas_mut()[file_in_folder.file_data_index]
    }

    fn get_folder_offset(&self, file_info: &FileInfo, region: Region) -> u64 {
        let file_in_folder = self.get_file_in_folder(file_info, region);

        self.get_folder_offsets()[file_in_folder.folder_offset_index as usize].offset
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

    fn get_file_offset_from_hash(&self, hash: Hash40, region: Region) -> Result<u64, LookupError> {
        let path_index = self.get_file_path_index_from_hash(hash)?;
        let file_info = self.get_file_info_from_path_index(path_index);
        let folder_offset = self.get_folder_offset(file_info, region);
        let file_data = self.get_file_data(&file_info, region);
        let offset = folder_offset + self.get_file_section_offset() + ((file_data.offset_in_folder as u64) <<  2);

        Ok(offset)
    }

    fn get_file_metadata<Hash: Into<Hash40>>(&self, hash: Hash, region: Region) -> Result<FileMetadata, LookupError> {
        fn inner<Arc: ArcLookup + ?Sized>(arc: &Arc, hash: Hash40, region: Region) -> Result<FileMetadata, LookupError> {
            match arc.get_file_path_index_from_hash(hash) {
                Ok(path_index) => {
                    let file_path = &arc.get_file_paths()[path_index];
                    let file_info = arc.get_file_info_from_path_index(path_index);
                    let folder_offset = arc.get_folder_offset(file_info, region);
                    let file_data = arc.get_file_data(&file_info, region);

                    let offset = folder_offset + arc.get_file_section_offset() + ((file_data.offset_in_folder as u64) <<  2);
                    
                    Ok(FileMetadata {
                        path_hash: file_path.path.hash40(),
                        ext_hash: file_path.ext.hash40(),
                        parent_hash: file_path.parent.hash40(),
                        file_name_hash: file_path.file_name.hash40(),
                        offset,
                        comp_size: file_data.comp_size as _,
                        decomp_size: file_data.decomp_size as _,
                        is_stream: false,
                        is_shared: arc.get_shared_section_offset() < offset,
                        is_redirect: file_info.flags.is_redirect(),
                        is_regional: file_info.flags.is_regional(),
                        is_localized: file_info.flags.is_localized(),
                        is_compressed: file_data.flags.compressed(),
                        uses_zstd: file_data.flags.use_zstd(),
                    })
                }
                Err(LookupError::Missing) => {
                    let stream_data = arc.get_stream_data(hash)?;

                    Ok(FileMetadata {
                        path_hash: hash,
                        ext_hash: Hash40(0),
                        parent_hash: Hash40(0),
                        file_name_hash: Hash40(0),
                        offset: stream_data.offset,
                        comp_size: stream_data.size,
                        decomp_size: stream_data.size,
                        is_stream: true,
                        is_shared: false,
                        is_redirect: false,
                        is_regional: false,
                        is_localized: false,
                        is_compressed: false,
                        uses_zstd: false,
                    })
                }
                Err(err) => Err(err)
            }
        }

        inner(self, hash.into(), region)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FileMetadata {
    pub path_hash: Hash40,
    pub ext_hash: Hash40,
    pub parent_hash: Hash40,
    pub file_name_hash: Hash40,
    pub offset: u64,
    pub comp_size: u64,
    pub decomp_size: u64,
    pub is_stream: bool,
    pub is_shared: bool,
    pub is_redirect: bool,
    pub is_regional: bool,
    pub is_localized: bool,
    pub is_compressed: bool,
    pub uses_zstd: bool,
}

impl QuickDir {
    fn range(&self) -> Range<usize> {
        let start = self.index() as usize;
        let end = start + self.count() as usize;

        start..end
    }
}

impl FileInfoBucket {
    fn range(self) -> Range<usize> {
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
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        let data = arc.get_file_contents("sound/config/bgm_property.bin", Region::UsEnglish).unwrap();

        //std::fs::write("bgm_property.bin", data).unwrap();

        //dbg!(arc.file_system.dirs.len());
    }

    #[test]
    fn test_get_stream_file() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        
        let labels = crate::hash_labels::HashLabels::from_file("/home/jam/Downloads/hashes.txt").unwrap();
        dbg!(arc.file_system.stream_entries[0].hash40().label(&labels));

        let data = arc.get_file_contents("stream:/sound/bgm/bgm_a10_malrpg2_zarazarasabaku.nus3audio", Region::UsEnglish).unwrap();

        //std::fs::write("bgm_a10_malrpg2_zarazarasabaku.nus3audio", data).unwrap();
    }

    #[test]
    fn test_get_shared() {
        let hash: Hash40 = "fighter/mario/model/body/c00/leyes_eye_mario_l_col.nutexb".into();


        let labels = crate::hash_labels::HashLabels::from_file("/home/jam/Downloads/hashes.txt").unwrap();
        dbg!(hash.label(&labels));

        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        let shared_files = arc.get_shared_files(hash, Region::UsEnglish).unwrap();

        let shared_files: Vec<Option<&str>> = shared_files.into_iter()
            .map(|hash| hash.label(&labels))
            .collect();

        dbg!(shared_files);
    }

    #[test]
    fn test_get_dir() {
        let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();
        let dir_info = arc.get_dir_info_from_hash("fighter/mario").unwrap();

        let start = dir_info.child_dir_start_index as usize;
        let end = (dir_info.child_dir_start_index as usize) + (dir_info.child_dir_count as usize);

        let children = &arc.file_system.folder_child_hashes[start..end].iter()
            .map(|child| &arc.file_system.dir_infos[child.index() as usize])
            .collect::<Vec<_>>();
        let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();

        for child in children {
            eprint!("{} ", child.name.label(&labels).map(String::from).unwrap_or_else(|| format!("0x{:X}", child.name.as_u64())));
            eprintln!("{}", child.parent.label(&labels).map(String::from).unwrap_or_else(|| format!("0x{:X}", child.parent.as_u64())));
        }

        dbg!(dir_info);
    }

    #[test]
    fn test_list_stream() {
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();

        let mut extensions = std::collections::HashSet::new();

        let labels = crate::hash_labels::HashLabels::from_file("/home/jam/Downloads/hashes.txt").unwrap();
        for file in arc.get_stream_listing("stream:/sound/bgm").unwrap() {
            if let Some(label) = file.hash40().label(&labels) {
                extensions.insert(label.rsplit(".").next().unwrap());
            }
        }

        assert_eq!(extensions.len(), 2);
        assert!(extensions.contains("nus3audio"));
        assert!(extensions.contains("nus3bank"));
    }

    //#[test]
    // fn test_print_complete_data() {
    //     let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();

    //     let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();
    //     //dbg!(arc.get_file_metadata("fighter/mewtwo/model/body/c00/model.numshb", Region::UsEnglish).unwrap());
    //     let file_path_index = dbg!(arc.get_file_path_index_from_hash("fighter/marth/model/body/c00/model.numshb".into()).unwrap());
    //     dbg!(arc.get_file_info_from_hash("stage/battlefield_s/normal/model/floating_plate_set/battlefield_linegaina_col.nutexb".into()).unwrap());
    //     let path_idx = arc.get_file_info_from_hash("stage/battlefield_s/normal/model/floating_plate_set/battlefield_linegaina_col.nutexb".into()).unwrap();
    //     dbg!(arc.get_file_paths()[path_idx.hash_index as usize].path.hash40().label(&labels).unwrap());
    // }

    // #[test]
    // fn test_battlefield() {
    //     let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();

    //     let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();
    //     // let file_path_index = dbg!(arc.get_file_path_index_from_hash("stage/battlefield_l/normal/model/floating_plate_set/battlefield_linegaina_col.nutexb".into()).unwrap());
    //     let file_path_index = dbg!(arc.get_file_path_index_from_hash("stage/common/shared/model/kumite_itembase2_set/battlefield_linegaina_col.nutexb".into()).unwrap());
    //     let file_path = dbg!(arc.get_file_paths()[file_path_index as usize]);
    //     let file_info_index = dbg!(arc.get_file_info_indices()[file_path.path.index() as usize]);
    //     let file_info = dbg!(arc.get_file_infos()[file_info_index.file_info_index as usize]);
    // }

    // #[test]
    // fn test_stage_common() {
    //     let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();

    //     let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();
    //     // let dir_info = dbg!(arc.get_dir_info_from_hash(Hash40::from("stage/common/shared/model/kumite_itembase2_set")).unwrap());
    //     let dir_info = dbg!(arc.get_dir_info_from_hash(Hash40::from("stage/common/shared/model/kumite_itembase2_set")).unwrap());

    //     let start = dir_info.file_name_start_index as usize;
    //     let end = (dir_info.file_name_start_index as usize) + (dir_info.file_info_count as usize);

    //     let file_infos = &arc.get_file_infos()[start..end].iter().collect::<Vec<_>>();

    //     let file_path_index = dbg!(arc.get_file_path_index_from_hash("stage/common/shared/model/kumite_itembase2_set/battlefield_linegaina_col.nutexb".into()).unwrap());

    //     for (index, info) in file_infos.iter().enumerate() {
    //         if info.hash_index == file_path_index {
    //             dbg!(start + index);
    //             dbg!(info);
    //             let file_path = arc.get_file_paths()[info.hash_index as usize];
    //             let info_to_data = dbg!(arc.get_file_info_to_datas()[info.info_to_data_index as usize]);

    //             let info_to_datas = arc.get_file_info_to_datas();

    //             let idk : Vec<&FileInfoToFileData> = info_to_datas
    //             .iter()
    //             .filter_map(|hash_to_path| {
    //                 if hash_to_path.file_info_index_and_flag == info_to_data.file_info_index_and_flag {
    //                     Some(hash_to_path)
    //                 } else {
    //                     None
    //                 }
    //             }).collect();

    //             dbg!(idk);
    //         }
    //     }
    // }

    #[test]
    fn crash() {
        let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();

        let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();
        let file_path_index = arc.get_file_path_index_from_hash(Hash40::from(154550836855)).unwrap();
        //let file_path_index = arc.get_file_path_index_from_hash("fighter/marth/model/body/c00/model.numshb".into()).unwrap();
        dbg!(file_path_index);
    }

    // #[test]
    // fn test_marth_c00() {
    //     let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();

    //     let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();
    //     let file_path_index = dbg!(arc.get_file_path_index_from_hash("fighter/marth/model/body/c00/model.numshb".into()).unwrap());
    //     let file_path = dbg!(arc.get_file_paths()[file_path_index as usize]);
    //     let file_info_index = dbg!(arc.get_file_info_indices()[file_path.path.index() as usize]);
    //     let dir_offset = &arc.get_folder_offsets()[file_info_index.dir_offset_index as usize];
    //     let start = dir_offset.sub_data_start_index as usize;
    //     let end = dir_offset.sub_data_start_index as usize + dir_offset.sub_data_count as usize;
    //     let subfiles = arc.get_file_datas()[start..end].iter().collect::<Vec<_>>();

    //     for subfile in subfiles {
    //         dbg!(subfile);
    //     }
        
    //     let file_info = dbg!(arc.get_file_infos()[file_info_index.file_info_index as usize]);
    //     dbg!(file_path.parent.hash40().label(&labels).unwrap());
    // }

    // #[test]
    // fn test_marth_c01() {
    //     let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();

    //     let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();
    //     let file_path_index = dbg!(arc.get_file_path_index_from_hash("fighter/marth/model/body/c01/model.numshb".into()).unwrap());
    //     let file_path = dbg!(arc.get_file_paths()[file_path_index as usize]);
    //     let file_info_index = dbg!(arc.get_file_info_indices()[file_path.path.index() as usize]);
    //     let file_info = dbg!(arc.get_file_infos()[file_info_index.file_info_index as usize]);
    // }

    // #[test]
    // fn test_marth_dir_c00() {
    //     let arc = ArcFile::open("H:/Documents/Smash/update/romfs/data_1010.arc").unwrap();

    //     let labels = crate::hash_labels::HashLabels::from_file("H:/Downloads/hashes.txt").unwrap();
    //     let dir_info = dbg!(arc.get_dir_info_from_hash(Hash40::from("fighter/marth/c01")).unwrap());

    //     let start = dir_info.file_name_start_index as usize;
    //     let end = (dir_info.file_name_start_index as usize) + (dir_info.file_info_count as usize);

    //     let file_infos = &arc.get_file_infos()[start..end].iter().collect::<Vec<_>>();

    //     let file_path_index = dbg!(arc.get_file_path_index_from_hash("fighter/marth/model/body/c01/model.numshb".into()).unwrap());

    //     for (index, info) in file_infos.iter().enumerate() {
    //         if info.hash_index == file_path_index {
    //             dbg!(start + index);
    //             dbg!(info.hash_index_2);
    //             let file_path = arc.get_file_paths()[info.hash_index as usize];
    //             dbg!(file_path.path.hash40().label(&labels).unwrap());
    //         }
    //     }
    // }
}
