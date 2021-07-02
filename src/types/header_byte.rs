use std::io::Read;

use crate::{
    consts::{
        COMPRESSED_HEADER_LOCAL_MESSAGE_NUMBER_MASK, COMPRESSED_HEADER_MASK,
        COMPRESSED_HEADER_TIME_OFFSET_MASK, DEFINITION_HEADER_MASK, DEVELOPER_FIELDS_MASK,
        LOCAL_MESSAGE_NUMBER_MASK,
    },
    io::read_u8,
};

//////////
//// HeaderByte
//////////

#[derive(Debug, PartialEq)]
pub struct HeaderByte {
    pub compressed_header: bool,
    pub definition: bool,
    pub dev_fields: bool,
    pub local_num: u8,
    pub time_offset: Option<u8>,
}
impl HeaderByte {
    pub fn new<R>(map: &mut R) -> Self
    where
        R: Read,
    {
        let b = read_u8(map);
        if (b & COMPRESSED_HEADER_MASK) == COMPRESSED_HEADER_MASK {
            Self {
                compressed_header: true,
                definition: false,
                dev_fields: false,
                local_num: (b & COMPRESSED_HEADER_LOCAL_MESSAGE_NUMBER_MASK) >> 5,
                time_offset: Some(b & COMPRESSED_HEADER_TIME_OFFSET_MASK),
            }
        } else {
            Self {
                compressed_header: false,
                definition: (b & DEFINITION_HEADER_MASK) == DEFINITION_HEADER_MASK,
                dev_fields: (b & DEVELOPER_FIELDS_MASK) == DEVELOPER_FIELDS_MASK,
                local_num: b & LOCAL_MESSAGE_NUMBER_MASK,
                time_offset: None,
            }
        }
    }
    pub fn compressed_timestamp(self) -> Option<u8> {
        if self.compressed_header {
            return self.time_offset;
        } else {
            return None;
        }
    }
}
