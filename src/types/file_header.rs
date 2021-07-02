use std::io::Read;

use crate::io::{arr4, read_u16, read_u32, read_u8, Endianness};

//////////
//// FileHeader
//////////

#[derive(Debug, PartialEq)]
pub struct FileHeader {
    pub filesize: u8,
    pub protocol: u8,
    pub profile_version: u16,
    pub num_record_bytes: u32,
    pub fileext: bool,
    pub crc: u16,
}
impl FileHeader {
    pub fn new<R>(map: &mut R) -> Self
    where
        R: Read,
    {
        Self {
            filesize: read_u8(map),
            protocol: read_u8(map),
            profile_version: read_u16(map, Endianness::Little),
            num_record_bytes: read_u32(map, Endianness::Little),
            fileext: {
                let buf = arr4(map);
                &buf == b".FIT"
            },
            crc: read_u16(map, Endianness::Little),
        }
    }
}
