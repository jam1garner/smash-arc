use crate::*;

/// Open an ArcFile from a given null-terminated path
///
/// **Note:** `Box<ArcFile>` is equivelant in layout to `*mut ArcFile`, but should be treated
/// as an opaque pointer
#[no_mangle]
pub unsafe extern "C" fn arc_open(path: *const i8) -> Box<ArcFile> {
    let path = std::ffi::CStr::from_ptr(path);
    let path = path.to_string_lossy().into_owned();

    Box::new(ArcFile::open(path).unwrap())
}

#[no_mangle]
pub extern "C" fn arc_free(_: Box<ArcFile>) {}

/// Get a listing of all the children of a directory
#[no_mangle]
pub extern "C" fn arc_list_dir(arc: &ArcFile, hash: Hash40) -> DirListing {
    arc.get_dir_listing(hash).into()
}

/// Get an owned slice of the file contents for a given file
#[no_mangle]
pub extern "C" fn arc_get_file_contents(arc: &ArcFile, hash: Hash40) -> FfiBytes {
    arc.get_file_contents(hash).ok().into()
}

#[no_mangle]
pub unsafe extern "C" fn arc_free_file_contents(ffi: FfiBytes) {
    Box::from_raw(std::slice::from_raw_parts_mut(ffi.ptr, ffi.size));
}

/// Extract a file to a given null-terminated path for the given Hash40
#[no_mangle]
pub extern "C" fn arc_get_file_info(arc: &ArcFile, hash: Hash40) -> Option<&FileData> {
    Some(arc.get_file_data(arc.get_file_info_from_hash(hash).ok()?))
}

/// Extract a file to a given null-terminated path for the given Hash40
#[no_mangle]
pub unsafe extern "C" fn arc_extract_file(arc: &ArcFile, hash: Hash40, path: *const i8) -> ExtractResult {
    match arc.get_file_contents(hash) {
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

/// Load hash labels from a given path
#[no_mangle]
pub unsafe extern "C" fn arc_load_labels(path: *const i8) {
    let path = std::ffi::CStr::from_ptr(path);
    let path = path.to_string_lossy().into_owned();

    Hash40::set_global_labels_file(path)
}

/// Get a label for a given Hash40
///
/// **Note:** Will return null if not found.
#[no_mangle]
pub unsafe extern "C" fn arc_hash40_to_str(hash: Hash40) -> *mut i8 {
    let labels = crate::hash_labels::GLOBAL_LABELS.read();

    hash.label(&labels)
        .map(|string| std::ffi::CString::new(string).unwrap().into_raw())
        .unwrap_or(std::ptr::null_mut())
} 

#[no_mangle]
pub unsafe extern "C" fn arc_free_str(string: *mut i8) {
    std::ffi::CString::from_raw(string);
}

#[repr(u8)]
pub enum ExtractResult {
    Ok = 0,
    IoError = 1,
    Missing = 2,
}

/// An owned slice of bytes
#[repr(C)]
pub struct FfiBytes {
    /// May be null on error
    ptr: *mut u8,
    size: usize,
}

impl From<Option<Vec<u8>>> for FfiBytes {
    fn from(bytes: Option<Vec<u8>>) -> Self {
        match bytes {
            Some(bytes) => {
                let size = bytes.len();
                let ptr = Box::leak(bytes.into_boxed_slice()).as_mut_ptr();

                Self { ptr, size }
            }

            None => Self { ptr: std::ptr::null_mut(), size: 0 }
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
                size: list.len()
            },
            None => Self {
                pointer: std::ptr::null(),
                size: 0
            }
        }
    }
}
