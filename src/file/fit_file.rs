use super::definition_record::DefinitionRecord;
use super::file_header::FileHeader;
use super::RecordHeaderByte;

use crate::{MessageType, Reader, TryFrom};

use log::{debug, info};
use std::collections::HashMap;
use std::io::Error;
use std::path::PathBuf;

const TIMESTAMP_HEADER_MASK: u8 = 0x80;
const TIMESTAMP_MESSAGE_TYPE_MASK: u8 = 0x60;
const TIMESTAMP_OFFSET_MASK: u8 = 0x1F;
const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

pub struct FitFile {
    pub file_header: FileHeader,
    pub records: Vec<Box<dyn MessageType>>,
}
impl FitFile {
    pub fn read(path: PathBuf) -> FitFile {
        let mut reader = Reader::new(path);
        let mut definitions: HashMap<u8, DefinitionRecord> = HashMap::new();
        let mut records: Vec<Box<dyn MessageType>> = Vec::new();

        let header = FileHeader::new(&mut reader).unwrap();

        while reader.pos().unwrap() < u64::from(header.file_length()) {
            RecordHeaderByte::new(&mut reader).map(|h| {
                if h.is_definition() {
                    definitions.insert(
                        h.local_msg_number(),
                        DefinitionRecord::new(&mut reader, h.has_developer_fields()),
                    );
                } else {
                    definitions
                        .get(&h.local_msg_number())
                        .map(|def| match def.new_record(&mut reader) {
                            Some(record) => {
                                // if (&record.name() == &"Record") {
                                // dbg!(&record.name());
                                // println!("time: {:?}", &record.get_field(253).unwrap().value);
                                // println!("lat: {:?}", &record.get_field(0).unwrap().value);
                                // println!("lon: {:?}", &record.get_field(1).unwrap().value);
                                // }
                                records.push(record);
                            }
                            None => debug!(":: no record found for {}", def.global_message_num),
                        })
                        .or_else(|| {
                            panic!("could not find definition for {}", &h.local_msg_number())
                        });
                }
            });
        }
        FitFile {
            file_header: header,
            records: records,
        }
    }
}
