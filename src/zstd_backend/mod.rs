#[cfg(feature = "libzstd")]
mod libzstd;

#[cfg(feature = "libzstd")]
pub use libzstd::*;

#[cfg(feature = "rust-zstd")]
mod rust_zstd;

#[cfg(feature = "rust-zstd")]
pub use rust_zstd::*;

#[cfg(all(feature = "libzstd", feature = "rust-zstd"))]
compile_error!("Only one ZSTD backend can be enabled at a time");

#[cfg(not(any(feature = "libzstd", feature = "rust-zstd", feature = "nozstd")))]
compile_error!("At least one ZSTD backend feature must be enabled");

#[cfg(not(any(feature = "libzstd", feature = "rust-zstd")))]
mod template {
    use std::io::{Read, Write, Result};

    pub fn copy_decode<R, W>(mut _source: R, mut _destination: W) -> Result<()>
        where R: Read,
              W: Write,
    {
        todo!()
    }

    pub fn decode_all<R: Read>(mut _source: R) -> Result<Vec<u8>> {
        todo!()
    }
}

// Reduce the number of errors, as "at least one zstd backend must be enabled" is enough
#[cfg(not(any(feature = "libzstd", feature = "rust-zstd")))]
pub use template::*;
