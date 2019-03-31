use std::collections::HashMap;

use super::iterators::MessageIterator;
use super::reader::{Endian, Reader};
use super::DataField;
use super::DefinitionRecord;
use super::Error;
use crate::{DefinedMessage, Message};

/// A wrapper around the sequence of Records parsed
pub struct FitFile {
    pub file_header: FileHeader,
    pub records: Vec<Message>,
}
impl FitFile {
    /// Return a summary of parsed messages
    ///
    pub fn message_counts(&self) -> HashMap<&str, u32> {
        self.records.iter().fold(HashMap::new(), |mut acc, x| {
            let c = acc.entry(x.name()).or_insert(0);
            *c += 1;
            acc
        })
    }

    /// Return an iterator over the parsed messages
    ///
    pub fn messages(&self) -> MessageIterator {
        MessageIterator {
            i: 0,
            v: &self.records,
        }
    }
}

#[derive(Debug)]
pub struct RecordHeaderByte {
    byte: u8,
}
impl RecordHeaderByte {
    pub fn new(reader: &mut Reader) -> Result<Self, crate::Error> {
        reader.byte().map(|b| Self { byte: b })
    }
    pub fn is_definition(&self) -> bool {
        (self.byte & crate::DEFINITION_HEADER_MASK) == crate::DEFINITION_HEADER_MASK
    }
    pub fn has_developer_fields(&self) -> bool {
        (self.byte & crate::DEVELOPER_FIELDS_MASK) == crate::DEVELOPER_FIELDS_MASK
    }
    pub fn local_msg_number(&self) -> u8 {
        self.byte & crate::LOCAL_MESSAGE_NUMBER_MASK
    }
}

#[derive(Debug)]
pub struct FileHeader {
    filesize: u8,
    protocol: u8,
    profile_version: u16,
    pub num_record_bytes: u32,
    fileext: bool,
    crc: u16,
}

// needs 14 bytes
impl FileHeader {
    pub fn new(reader: &mut Reader) -> Result<FileHeader, Error> {
        let endianness = Endian::Little;
        Ok(FileHeader {
            filesize: reader.byte()?,
            protocol: reader.byte()?,
            profile_version: reader.u16(&endianness)?,
            num_record_bytes: reader.u32(&endianness)?,
            fileext: read_fit_string(reader.bytes(4)?.as_slice()),
            crc: reader.u16(&endianness)?,
        })
    }
    pub fn file_length(&self) -> u32 {
        self.num_record_bytes + 14
    }
}

fn read_fit_string(buffer: &[u8]) -> bool {
    let ext = ".FIT";
    buffer == ext.as_bytes()
}

pub fn read_data_record(def: &DefinitionRecord, reader: &mut Reader) -> Option<Message> {
    let raw_fields: Vec<_> = def
        .field_defs
        .iter()
        .map(|fd| DataField::new(reader, &def.architecture, &fd))
        .collect();
    Message::new(def.global_message_num).and_then(|mut m| {
        raw_fields.into_iter().for_each(|df| {
            if df.value.is_some() {
                m.add_value(&df);
            }
        });
        Some(m)
    })
}
