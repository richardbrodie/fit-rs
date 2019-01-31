use super::definition_record::DefinitionRecord;
use super::file_header::FileHeader;
use super::RecordHeaderByte;
use crate::{MessageType, Reader};

use log::warn;
use std::collections::HashMap;
use std::path::PathBuf;

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
            let _ = RecordHeaderByte::new(&mut reader).map(|h| {
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
                                records.push(record);
                            }
                            None => warn!(":: no record found for {}", def.global_message_num),
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
