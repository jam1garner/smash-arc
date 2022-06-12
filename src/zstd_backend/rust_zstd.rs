use ruzstd::streaming_decoder::StreamingDecoder;
use std::io::{self, Error, ErrorKind, Read, Result, Write};

pub fn copy_decode<R, W>(mut source: R, mut destination: W) -> Result<()>
where
    R: Read,
    W: Write,
{
    let mut decoder =
        StreamingDecoder::new(&mut source).map_err(|err| Error::new(ErrorKind::Other, err))?;

    io::copy(&mut decoder, &mut destination).map(|_| ())
}

pub fn decode_all<R: Read>(mut source: R) -> Result<Vec<u8>> {
    let mut decoder =
        StreamingDecoder::new(&mut source).map_err(|err| Error::new(ErrorKind::Other, err))?;

    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;

    Ok(out)
}
