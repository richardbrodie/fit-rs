//! **fit** aims to be an extremely fast decoder for the [FIT file](https://www.thisisant.com) format from ANT+.
//!
//! # Use
//!
//! Simply call `FitFile::read` with a path to a fit file.

#![allow(unused)]
#[allow(clippy::all)]
use log::warn;
use std::collections::HashMap;
use std::path::PathBuf;

mod error;
mod file;
mod messages;
mod reader;
mod value;

pub use self::error::{Error, ErrorKind};
pub use self::file::{DataField, FitFile};
pub use self::messages::{DefinedMessage, Field};
pub use self::value::Value;

use crate::messages::new_record;
use crate::reader::Reader;
use file::{DefinitionRecord, FileHeader, RecordHeaderByte};

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
    let mut records: Vec<Box<dyn DefinedMessage>> = Vec::new();

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
                .ok_or(crate::ErrorKind::MissingDefinition(h.local_msg_number()))?;
            read_data_record(&def, &mut reader).map_or_else(
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

fn read_data_record(
    def: &DefinitionRecord,
    reader: &mut Reader,
) -> Option<Box<dyn DefinedMessage>> {
    let raw_fields: Vec<_> = def
        .field_defs
        .iter()
        .map(|fd| DataField::new(reader, &def.architecture, &fd))
        .collect();
    new_record(def.global_message_num).and_then(|mut r| {
        raw_fields.into_iter().for_each(|df| {
            if df.value.is_some() {
                r.process_raw_value(&df);
            }
        });
        Some(r)
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
