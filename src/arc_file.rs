use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Seek, SeekFrom},
    net::ToSocketAddrs,
    path::Path,
    sync::Mutex,
};

use binrw::{io::Cursor, BinRead, BinReaderExt, BinResult, FilePtr64};

use crate::filesystem::HashToIndex;
use crate::hash_labels::HashLabels;
use crate::{CompressedFileSystem, FileNode, FileSystem, Hash40};

pub trait SeekRead: std::io::Read + std::io::Seek {}
impl<R: std::io::Read + std::io::Seek> SeekRead for R {}

/// A struct representing the data.arc file
#[derive(BinRead)]
#[br(magic = 0xABCD_EF98_7654_3210_u64)]
pub struct ArcFile {
    pub stream_section_offset: u64,
    pub file_section_offset: u64,
    pub shared_section_offset: u64,

    #[br(parse_with = FilePtr64::parse)]
    #[br(map = |x: CompressedFileSystem| x.0)]
    pub file_system: FileSystem,
    pub patch_section: u64,

    #[br(calc = Mutex::new(Box::new(Cursor::new([])) as _))]
    pub reader: Mutex<Box<dyn SeekRead + Send>>,

    #[cfg(feature = "dir-listing")]
    #[br(calc = generate_dir_listing(&file_system))]
    pub dirs: HashMap<Hash40, Vec<FileNode>>,
}

#[cfg(feature = "dir-listing")]
fn parents_of_dir(dir: Hash40, labels: &mut HashLabels) -> Option<Vec<(Hash40, FileNode)>> {
    let label = dir.label(labels)?.to_owned();
    let mut label = &label[..];
    let mut hashes = Vec::new();
    let mut last_hash = dir;

    while let Some(len) = label.trim_end_matches('/').rfind('/') {
        label = &label[..len];

        let hash = labels.add_label(label);
        hashes.push((hash, FileNode::Dir(last_hash)));
        last_hash = hash;
    }

    hashes.push((crate::hash40::hash40("/"), FileNode::Dir(last_hash)));

    Some(hashes)
}

#[cfg(feature = "dir-listing")]
fn dir_listing_flat<'a>(
    fs: &'a FileSystem,
    labels: &'a mut HashLabels,
) -> impl Iterator<Item = (Hash40, FileNode)> + 'a {
    let dirs: HashSet<_> = fs
        .file_paths
        .iter()
        .map(|path| path.parent.hash40())
        .collect();

    let mut stream_dirs = Vec::new();
    for path_hash in fs.stream_hash_to_entries.iter().map(HashToIndex::hash40) {
        if let Some(label) = path_hash
            .label(labels)
            .and_then(|label| label.rfind('/').map(|pos| label[..pos].to_owned()))
        {
            let mut label = &label[..];
            let mut last_hash = crate::hash40::hash40(label);
            labels.add_label(label);

            while let Some(len) = label.trim_end_matches('/').rfind('/') {
                label = &label[..len];

                let hash = labels.add_label(label);
                stream_dirs.push((hash, FileNode::Dir(last_hash)));
                last_hash = hash;
            }

            stream_dirs.push((crate::hash40::hash40("/"), FileNode::Dir(last_hash)));

            labels.add_label(label);
        }
    }

    let stream_paths: Vec<(Hash40, &str)> = fs
        .stream_hash_to_entries
        .iter()
        .filter_map(|entry| {
            entry
                .hash40()
                .label(labels)
                .map(|path| (entry.hash40(), path))
        })
        .collect();
    let stream_files: Vec<(Hash40, FileNode)> = stream_paths
        .iter()
        .flat_map(|(path_hash, path)| {
            path.rfind('/').map(|pos| {
                let dir = crate::hash40::hash40(&path[..pos]);

                (dir, FileNode::File(*path_hash))
            })
        })
        .collect();

    // Generate parents for directories
    let dirs = dirs
        .into_iter()
        .filter_map(move |dir| parents_of_dir(dir, labels).map(|x| x.into_iter()))
        .flatten();

    // Generate parents for files
    fs.file_paths
        .iter()
        .map(|path| (path.parent.hash40(), FileNode::File(path.path.hash40())))
        .chain(dirs)
        .chain(stream_files.into_iter())
        .chain(stream_dirs.into_iter())
}

#[cfg(feature = "dir-listing")]
fn generate_dir_listing(fs: &FileSystem) -> HashMap<Hash40, Vec<FileNode>> {
    let mut dirs = HashMap::new();

    let mut labels = crate::hash_labels::GLOBAL_LABELS.write();
    for (parent, child) in dir_listing_flat(fs, &mut labels) {
        let listing = dirs.entry(parent).or_insert_with(Vec::new);
        match listing.binary_search(&child) {
            Ok(_) => (),
            Err(insert_point) => listing.insert(insert_point, child),
        }
    }

    dirs
}

impl ArcFile {
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }

    #[cfg(feature = "network")]
    pub fn open_over_network<Addr: ToSocketAddrs>(ip: Addr) -> BinResult<Self> {
        let mut reader = BufReader::new(network_reader::NetworkReader::new(ip)?);

        reader.seek(SeekFrom::Start(0))?;

        Self::from_reader(reader)
    }

    pub fn from_reader<R: SeekRead + Send + 'static>(mut reader: R) -> BinResult<Self> {
        let arc: Self = reader.read_le()?;

        *arc.reader.lock().unwrap() = Box::new(reader);

        Ok(arc)
    }

    #[cfg(feature = "dir-listing")]
    pub fn get_dir_listing<Hash: Into<Hash40>>(&self, hash: Hash) -> Option<&[FileNode]> {
        self.dirs.get(&hash.into()).map(AsRef::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_tree_hash(arc: &ArcFile, hash: Hash40, depth: usize) {
        for file in arc.get_dir_listing(hash).unwrap() {
            (0..depth).for_each(|_| print!("    "));
            match file {
                FileNode::File(file) => {
                    println!(
                        "L {}",
                        file.global_label()
                            .unwrap_or_else(|| format!("{:#x}", file.as_u64()))
                    );
                }
                FileNode::Dir(dir) => {
                    println!(
                        "L {}",
                        dir.global_label()
                            .unwrap_or_else(|| format!("{:#x}", dir.as_u64()))
                    );
                    //print_tree_hash(arc, *dir, depth + 1);
                }
            }
        }
    }

    fn print_tree(arc: &ArcFile, dir: &str) {
        println!("{}:", dir);
        print_tree_hash(arc, dir.into(), 1);
    }

    #[test]
    fn test_listing() {
        Hash40::set_global_labels_file("/home/jam/Downloads/hashes.txt");
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        //let arc = ArcFile::open_over_network(("192.168.86.32", 43022)).unwrap();

        print_tree(&arc, "/");
        //dbg!(arc.get_dir_listing("fighter/mario/model/body/c00/"));
    }

    #[test]
    fn test_stream_listing() {
        Hash40::set_global_labels_file("/home/jam/Downloads/hashes.txt");
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();
        //let arc = ArcFile::open_over_network(("192.168.86.32", 43022)).unwrap();

        print_tree(&arc, "stream:");
        //dbg!(arc.get_dir_listing("fighter/mario/model/body/c00/"));
    }
}
