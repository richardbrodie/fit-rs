use super::definition_record::DefinitionRecord;
use super::file_header::FileHeader;
use super::RecordHeaderByte;
use crate::reader::Reader;
use crate::DefinedMessageType;

use log::warn;
use std::collections::HashMap;
use std::path::PathBuf;

type MessageBox = Box<dyn DefinedMessageType>;

/// A wrapper around the sequence of Records parsed

pub struct FitFile {
    _file_header: FileHeader,
    records: Vec<MessageBox>,
}
impl FitFile {
    /// Return the name and value of a specific field number
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    pub fn message_counts(&self) -> HashMap<&str, u32> {
        self.records.iter().fold(HashMap::new(), |mut acc, x| {
            let c = acc.entry(x.name()).or_insert(0);
            *c += 1;
            acc
        })
    }

    /// Return the name and value of a specific field number
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    pub fn single_message(&self, name: &str) -> Option<&MessageBox> {
        self.records.iter().find(|r| r.name() == name)
    }

    /// Return the name and value of a specific field number
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    pub fn multiple_messages(&self, name: &str) -> Vec<&MessageBox> {
        self.records
            .iter()
            .filter_map(|r| if r.name() == name { Some(r) } else { None })
            .collect()
    }

    /// Return the name and value of a specific field number
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    pub fn read(path: PathBuf) -> FitFile {
        let mut reader = Reader::new(path);
        let mut definitions: HashMap<u8, DefinitionRecord> = HashMap::new();
        let mut records: Vec<MessageBox> = Vec::new();

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
                        .map(|def| {
                            def.read_data_record(&mut reader).map_or_else(
                                || warn!(":: no record found for {}", def.global_message_num),
                                |record| records.push(record),
                            )
                        })
                        .or_else(|| {
                            panic!("could not find definition for {}", &h.local_msg_number())
                        });
                }
            });
        }
        FitFile {
            _file_header: header,
            records: records,
        }
    }
}
