//! **fit** aims to be an extremely fast decoder for the [FIT file](https://www.thisisant.com) format from ANT+.
//!
//! # Use
//!
//! Simply call `FitFile::read` with a path to a fit file.

#![allow(unused)]
#![allow(clippy::unreadable_literal)]
#![warn(clippy::perf, clippy::complexity)]

use fit_sdk::{new_record, DefinedMessage, Value};
use log::warn;
use std::collections::HashMap;
use std::path::PathBuf;

mod data_field;
mod definition_record;
mod error;
mod fit_file;
mod iterators;
mod message;
mod reader;

pub use self::data_field::DataField;
pub use self::error::{Error, ErrorKind};
pub use fit_file::{FileHeader, FitFile, RecordHeaderByte};
pub use message::Message;

use definition_record::DefinitionRecord;
use reader::Reader;

const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

/// Reads a given FIT file and returns a FitFile struct containing the messages
///
/// # Example
///
/// ```rust
/// use std::path::PathBuf;
///
/// let filepath = PathBuf::from("fits/garmin_1000.fit");
/// let _ = fit::read(filepath);
/// ```
pub fn read(path: PathBuf) -> Result<FitFile, Error> {
    let mut reader = Reader::new(path)?;
    let mut definitions: HashMap<u8, DefinitionRecord> = HashMap::new();
    let mut records: Vec<Message> = Vec::new();

    let header = FileHeader::new(&mut reader)?;

    let file_length = u64::from(header.file_length());
    loop {
        let h = RecordHeaderByte::new(&mut reader)?;
        if h.has_developer_fields() {
            Err(crate::ErrorKind::HasDeveloperFields)?
        } else if h.is_definition() {
            let def = DefinitionRecord::new(&mut reader, h.has_developer_fields())?;
            definitions.insert(h.local_msg_number(), def);
        } else {
            let def = definitions
                .get(&h.local_msg_number())
                .ok_or_else(|| crate::ErrorKind::MissingDefinition(h.local_msg_number()))?;
            fit_file::read_data_record(&def, &mut reader).map_or_else(
                || warn!(":: no record found for {}", def.global_message_num),
                |record| records.push(record),
            );
        }
        if reader.pos()? >= file_length {
            break;
        }
    }
    Ok(FitFile {
        file_header: header,
        records,
    })
}

#[cfg(test)]
pub mod tests {
    use crate::reader::Reader;
    use std::path::PathBuf;

    pub fn fit_setup() -> Result<Reader, crate::Error> {
        let path = PathBuf::from("fits/garmin_1000.fit");
        Reader::new(path)
    }

    #[test]
    fn it_reads_garmin_1000_file() {
        let filepath = PathBuf::from("fits/garmin_1000.fit");
        let _ = crate::read(filepath);
    }
    #[test]
    fn it_reads_garmin_520_file() {
        let filepath = PathBuf::from("fits/garmin_520_long.fit");
        let _ = crate::read(filepath);
    }
    #[test]
    fn it_reads_garmin_520_file_with_power() {
        let filepath = PathBuf::from("fits/garmin_520_power.fit");
        let _ = crate::read(filepath);
    }
    #[test]
    fn it_reads_trainerroad_file() {
        let filepath = PathBuf::from("fits/trainerroad.fit");
        let _ = crate::read(filepath);
    }
    #[test]
    fn it_reads_wahoo_file() {
        let filepath = PathBuf::from("fits/wahoo.fit");
        assert!(crate::read(filepath).is_ok());
    }
    #[test]
    fn it_panics_reading_wahoo_file_with_developer_fields() {
        let filepath = PathBuf::from("fits/wahoo_dev_fields.fit");
        assert!(crate::read(filepath).is_err());
    }
}
