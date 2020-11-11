use std::io::{self, prelude::*};
use std::net::{ToSocketAddrs, TcpStream};

struct NetworkReader {
    socket: TcpStream,

}

impl Seek for NetworkReader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        todo!()
    }
}

impl Read for NetworkReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}
