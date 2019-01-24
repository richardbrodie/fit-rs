use fit::MessageType;
use fit::TryFrom;
use log::{debug, info};
use std::collections::HashMap;
use std::io::Error;
use std::path::PathBuf;

use super::definition_record::DefinitionRecord;
use super::file_header::FileHeader;
use crate::reader::Reader;

const TIMESTAMP_HEADER_MASK: u8 = 0x80;
const TIMESTAMP_MESSAGE_TYPE_MASK: u8 = 0x60;
const TIMESTAMP_OFFSET_MASK: u8 = 0x1F;
const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

#[derive(Debug)]
struct HeaderByte {
    byte: u8,
}
impl HeaderByte {
    fn new(reader: &mut Reader) -> Result<Self, Error> {
        Ok(Self {
            byte: reader.byte()?,
        })
    }
    fn is_timestamp(&self) -> bool {
        (self.byte & TIMESTAMP_HEADER_MASK) == TIMESTAMP_HEADER_MASK
    }
    fn is_definition(&self) -> bool {
        (self.byte & DEFINITION_HEADER_MASK) == DEFINITION_HEADER_MASK
    }
    fn has_developer_fields(&self) -> bool {
        (self.byte & DEVELOPER_FIELDS_MASK) == DEVELOPER_FIELDS_MASK
    }
    fn local_msg_number(&self) -> u8 {
        self.byte & LOCAL_MESSAGE_NUMBER_MASK
    }
}

pub struct FitFile {
    file_header: FileHeader,
    definitions: HashMap<u8, DefinitionRecord>,
}
impl FitFile {
    pub fn read(path: PathBuf) {
        let mut reader = Reader::new(path);
        let mut definitions: HashMap<u8, DefinitionRecord> = HashMap::new();
        // let mut records: Vec<dyn fit::MessageType> = Vec::new();

        let header = FileHeader::new(&mut reader).unwrap();

        while reader.pos().unwrap() < u64::from(header.file_length()) {
            if let Ok(h) = HeaderByte::new(&mut reader) {
                match h.is_definition() {
                    true => {
                        let definition =
                            DefinitionRecord::new(&mut reader, h.has_developer_fields());
                        definitions.insert(h.local_msg_number(), definition);
                    }
                    false => match definitions.get(&h.local_msg_number()) {
                        Some(def) => match def.new_record(&mut reader) {
                            Some(record) => {
                                if let Some(v) = record.get_value(253) {
                                    println!("{:?}", u32::try_from(v));
                                }
                            }
                            None => debug!(":: no record found for {}", def.global_message_num),
                        },
                        None => {
                            panic!("could not find definition for {}", &h.local_msg_number());
                        }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn it_reads_header_byte() {
        let mut reader = fit_setup();
        reader.skip(14); // FileHeader
        let header_byte = HeaderByte::new(&mut reader).unwrap();
        assert_eq!(header_byte.is_definition(), true);
    }

    #[test]
    fn it_reads_whole_file() {
        let mut reader = fit_setup();
        let filepath = PathBuf::from("fits/working_garmin.fit");
        let fit = FitFile::read(filepath);
    }
}
