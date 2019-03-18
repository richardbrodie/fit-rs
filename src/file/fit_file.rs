use super::{definition_record::DefinitionRecord, file_header::FileHeader, RecordHeaderByte};
use crate::reader::Reader;
use crate::{DefinedMessageType, Error};

use log::warn;
use std::{collections::HashMap, path::PathBuf};

type MessageBox = Box<dyn DefinedMessageType>;

/// An iterator over the parsed Records
pub struct MessageIterator<'a> {
    i: usize,
    v: &'a Vec<MessageBox>,
}
impl<'a> MessageIterator<'a> {
    pub fn filter_name(self, name: &'a str) -> FilterMessageIterator<'a, Self> {
        FilterMessageIterator { k: name, i: self }
    }
}
impl<'a> Iterator for MessageIterator<'a> {
    type Item = &'a MessageBox;

    fn next(&mut self) -> Option<Self::Item> {
        match self.v.get(self.i) {
            Some(item) => {
                self.i += 1;
                Some(item)
            }
            _ => None,
        }
    }
}
pub struct FilterMessageIterator<'a, MessageIterator> {
    k: &'a str,
    i: MessageIterator,
}
impl<'a, I: Iterator<Item = &'a MessageBox>> Iterator for FilterMessageIterator<'a, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        for x in &mut self.i {
            if x.name() == self.k {
                return Some(x);
            }
        }
        None
    }
}
/// A wrapper around the sequence of Records parsed
pub struct FitFile {
    _file_header: FileHeader,
    records: Vec<MessageBox>,
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
    pub fn messages<'a>(&'a self) -> MessageIterator<'a> {
        MessageIterator {
            i: 0,
            v: &self.records,
        }
    }

    /// Return the name and value of a specific field number
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    pub fn read(path: PathBuf) -> Result<FitFile, Error> {
        let mut reader = Reader::new(path)?;
        let mut definitions: HashMap<u8, DefinitionRecord> = HashMap::new();
        let mut records: Vec<MessageBox> = Vec::new();

        let header = FileHeader::new(&mut reader)?;

        let file_length = u64::from(header.file_length());
        loop {
            let h = RecordHeaderByte::new(&mut reader)?;
            if h.has_developer_fields() {
                Err(crate::ErrorKind::HasDeveloperFields)?
            } else if h.is_definition() {
                DefinitionRecord::new(&mut reader, h.has_developer_fields())
                    .map(|def| definitions.insert(h.local_msg_number(), def));
            } else {
                let def = definitions
                    .get(&h.local_msg_number())
                    .ok_or(crate::ErrorKind::MissingDefinition(h.local_msg_number()))?;
                def.read_data_record(&mut reader).map_or_else(
                    || warn!(":: no record found for {}", def.global_message_num),
                    |record| records.push(record),
                );
            }
            if reader.pos()? >= file_length {
                break;
            }
        }
        Ok(FitFile {
            _file_header: header,
            records,
        })
    }
}
