use crate::*;
use region::Region;

/// Open an ArcFile from a given null-terminated path
///
/// **Note:** `Box<ArcFile>` is equivelant in layout to `*mut ArcFile`, but should be treated
/// as an opaque pointer
///
/// **Note:** If you want directory listing to work, be sure to set a hashfile using
/// [`arc_load_labels`](arc_load_labels)
#[no_mangle]
pub unsafe extern "C" fn arc_open(path: *const i8) -> Option<Box<ArcFile>> {
    let path = std::ffi::CStr::from_ptr(path);
    let path = path.to_string_lossy().into_owned();

    Some(Box::new(ArcFile::open(path).ok()?))
}

/// Open an ArcFile from a given IP address.
/// This is intended for use with [arc-network](https://github.com/jam1garner/arc-network).
///
/// **Note:** `Box<ArcFile>` is equivelant in layout to `*mut ArcFile`, but should be treated
/// as an opaque pointer
///
/// **Note:** If you want directory listing to work, be sure to set a hashfile using
/// [`arc_load_labels`](arc_load_labels)
#[no_mangle]
pub unsafe extern "C" fn arc_open_networked(ip: *const i8) -> Option<Box<ArcFile>> {
    let ip = std::ffi::CStr::from_ptr(ip);
    let ip = ip.to_string_lossy().into_owned();

    Some(Box::new(
        ArcFile::open_over_network((ip.as_str(), 43022)).ok()?,
    ))
}

/// Frees the memory allocated by [arc_open] or [arc_open_networked].
#[no_mangle]
pub extern "C" fn arc_free(_: Box<ArcFile>) {}

/// Get a listing of all the children of a directory
#[no_mangle]
pub extern "C" fn arc_list_dir(arc: &ArcFile, hash: Hash40) -> DirListing {
    arc.get_dir_listing(hash).into()
}

/// Get the file version of the ARC.
#[no_mangle]
pub extern "C" fn arc_get_version(arc: &ArcFile) -> u32 {
    arc.file_system.fs_header.version
}

/// Get a listing of all the children of a directory
#[no_mangle]
pub extern "C" fn arc_list_root_dir(arc: &ArcFile) -> DirListing {
    arc.get_dir_listing("/").into()
}

/// Get an owned slice of the file contents for a given file
#[no_mangle]
pub extern "C" fn arc_get_file_contents(arc: &ArcFile, hash: Hash40) -> FfiBytes {
    arc.get_file_contents(hash, Region::UsEnglish).ok().into()
}

/// Get an owned slice of the file contents for a given file, picking a specific region
#[no_mangle]
pub extern "C" fn arc_get_file_contents_regional(
    arc: &ArcFile,
    hash: Hash40,
    region: Region,
) -> FfiBytes {
    arc.get_file_contents(hash, region).ok().into()
}

/// Frees the memory allocated by [arc_get_file_contents] or [arc_get_file_contents_regional].
#[no_mangle]
pub unsafe extern "C" fn arc_free_file_contents(ffi: FfiBytes) {
    Box::from_raw(std::slice::from_raw_parts_mut(ffi.ptr, ffi.size));
}

/// Get file information such as offset and decompressed size for the given [Hash40].
#[no_mangle]
pub extern "C" fn arc_get_file_info(arc: &ArcFile, hash: Hash40) -> Option<&FileData> {
    arc.get_file_data_from_hash(hash, Region::UsEnglish).ok()
}

/// Get file information such as offset and decompressed size for the given [Hash40] for a specified region.
#[no_mangle]
pub extern "C" fn arc_get_file_info_regional(
    arc: &ArcFile,
    hash: Hash40,
    region: Region,
) -> Option<&FileData> {
    arc.get_file_data_from_hash(hash, region).ok()
}

/// Get an owned list of shared files for a file, given its hash
#[no_mangle]
pub extern "C" fn arc_get_shared_files(arc: &ArcFile, hash: Hash40) -> FfiVec<Hash40> {
    arc.get_shared_files(hash, Region::UsEnglish).ok().into()
}

/// Get an owned list of shared files for a file, given its hash for a specified region
#[no_mangle]
pub extern "C" fn arc_get_shared_files_regional(
    arc: &ArcFile,
    hash: Hash40,
    region: Region,
) -> FfiVec<Hash40> {
    arc.get_shared_files(hash, region).ok().into()
}

/// Free an owned list of shared files
#[no_mangle]
pub unsafe extern "C" fn arc_free_shared_file_list(ffi: FfiVec<Hash40>) {
    Box::from_raw(std::slice::from_raw_parts_mut(ffi.ptr, ffi.size));
}

/// Extract a file to a given null-terminated path for the given [Hash40]
#[no_mangle]
pub unsafe extern "C" fn arc_extract_file(
    arc: &ArcFile,
    hash: Hash40,
    path: *const i8,
) -> ExtractResult {
    match arc.get_file_contents(hash, Region::UsEnglish) {
        Ok(contents) => {
            let path = std::ffi::CStr::from_ptr(path);
            let path = path.to_string_lossy().into_owned();
            match std::fs::write(path, contents) {
                Ok(_) => ExtractResult::Ok,
                Err(_) => ExtractResult::IoError,
            }
        }
        Err(lookups::LookupError::Missing) => ExtractResult::Missing,
        Err(_) => ExtractResult::IoError,
    }
}

/// Extract a file to a given null-terminated path for the given [Hash40] for a given region
#[no_mangle]
pub unsafe extern "C" fn arc_extract_file_regional(
    arc: &ArcFile,
    hash: Hash40,
    path: *const i8,
    region: Region,
) -> ExtractResult {
    match arc.get_file_contents(hash, region) {
        Ok(contents) => {
            let path = std::ffi::CStr::from_ptr(path);
            let path = path.to_string_lossy().into_owned();
            match std::fs::write(path, contents) {
                Ok(_) => ExtractResult::Ok,
                Err(_) => ExtractResult::IoError,
            }
        }
        Err(lookups::LookupError::Missing) => ExtractResult::Missing,
        Err(_) => ExtractResult::IoError,
    }
}

/// Load hash labels from a given path.
/// Returns `true` on success.
#[no_mangle]
pub unsafe extern "C" fn arc_load_labels(path: *const i8) -> bool {
    let path = std::ffi::CStr::from_ptr(path);
    let path = path.to_string_lossy().into_owned();

    Hash40::set_global_labels_file(path).is_ok()
}

/// Get a label for a given Hash40 or [std::ptr::null_mut] if no label could be found.
#[no_mangle]
pub unsafe extern "C" fn arc_hash40_to_str(hash: Hash40) -> *mut i8 {
    let labels = crate::hash_labels::GLOBAL_LABELS.read();

    hash.label(&labels)
        .map(|string| std::ffi::CString::new(string).unwrap().into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Hashes a given string and returns the result as a [Hash40].
#[no_mangle]
pub unsafe extern "C" fn arc_str_to_hash40(string: *const i8) -> Hash40 {
    let string = std::ffi::CStr::from_ptr(string);
    hash40::hash40_from_bytes(string.to_bytes())
}

/// Frees the memory allocated by [arc_hash40_to_str].
#[no_mangle]
pub unsafe extern "C" fn arc_free_str(string: *mut i8) {
    // this is definitely technically maybe UB depending on the allocator, oops?
    drop(std::ffi::CString::from_raw(string));
}

#[no_mangle]
pub extern "C" fn arc_get_file_metadata(
    arc: &ArcFile,
    hash: Hash40,
) -> crate::lookups::FileMetadata {
    arc.get_file_metadata(hash, Region::UsEnglish)
        .unwrap_or(crate::lookups::FileMetadata {
            path_hash: hash,
            ext_hash: Hash40(0),
            parent_hash: Hash40(0),
            file_name_hash: Hash40(0),
            offset: 0,
            comp_size: 0,
            decomp_size: 0,
            is_stream: false,
            is_shared: false,
            is_redirect: false,
            is_regional: false,
            is_localized: false,
            is_compressed: false,
            uses_zstd: false,
        })
}

#[no_mangle]
pub extern "C" fn arc_get_file_metadata_regional(
    arc: &ArcFile,
    hash: Hash40,
    region: Region,
) -> crate::lookups::FileMetadata {
    arc.get_file_metadata(hash, region)
        .unwrap_or(crate::lookups::FileMetadata {
            path_hash: hash,
            ext_hash: Hash40(0),
            parent_hash: Hash40(0),
            file_name_hash: Hash40(0),
            offset: 0,
            comp_size: 0,
            decomp_size: 0,
            is_stream: false,
            is_shared: false,
            is_redirect: false,
            is_regional: false,
            is_localized: false,
            is_compressed: false,
            uses_zstd: false,
        })
}

#[no_mangle]
pub extern "C" fn arc_get_file_count(arc: &ArcFile) -> u64 {
    arc.file_system.file_paths.len() as u64 + arc.file_system.stream_entries.len() as u64
}

/// Generates a cache to aid in faster fuzzy searching
#[cfg(feature = "search")]
#[no_mangle]
pub extern "C" fn arc_generate_search_cache(arc: &ArcFile) -> Box<crate::search::SearchCache> {
    Box::new(arc.generate_search_cache())
}

/// Searches for a given path. Requires labels to be loaded and a search cache be generated
#[cfg(feature = "search")]
#[no_mangle]
pub unsafe extern "C" fn arc_search_files(
    cache: &crate::search::SearchCache,
    search_term: *const i8,
    max_files: usize,
) -> FfiVec<Hash40> {
    let term = std::ffi::CStr::from_ptr(search_term);
    let term = term.to_string_lossy();
    let labels = crate::hash_labels::GLOBAL_LABELS.read();

    Some(cache.search(&term, &labels, max_files)).into()
}

#[repr(u8)]
pub enum ExtractResult {
    Ok = 0,
    IoError = 1,
    Missing = 2,
}

type FfiBytes = FfiVec<u8>;

/// An owned slice of bytes
#[repr(C)]
pub struct FfiVec<T: Sized> {
    /// May be null on error
    ptr: *mut T,
    size: usize,
}

impl<T: Sized> From<Option<Vec<T>>> for FfiVec<T> {
    fn from(list: Option<Vec<T>>) -> Self {
        match list {
            Some(list) => {
                let size = list.len();
                let ptr = Box::leak(list.into_boxed_slice()).as_mut_ptr();

                Self { ptr, size }
            }

            None => Self {
                ptr: std::ptr::null_mut(),
                size: 0,
            },
        }
    }
}

/// A list representing the borrowed contents of a directory
///
/// **Note:** FileNode is equivelant in layout to:
/// ```rs
/// #[repr(C)]
/// struct FileNode {
///     kind: u64, // 0 = dir, 1 = file
///     hash: u64
/// }
/// ```
#[repr(C)]
pub struct DirListing {
    /// Will be null if directory listing failed
    pointer: *const FileNode,
    size: usize,
}

impl From<Option<&[FileNode]>> for DirListing {
    fn from(list: Option<&[FileNode]>) -> Self {
        match list {
            Some(list) => Self {
                pointer: list.as_ptr(),
                size: list.len(),
            },
            None => Self {
                pointer: std::ptr::null(),
                size: 0,
            },
        }
    }
}
