//! **fit** aims to be an extremely fast decoder for the [FIT file](https://www.thisisant.com) format from ANT+.
//!
//! # Use
//!
//! Simply call `FitFile::read` with a path to a fit file.

#![allow(unused)]
use std::path::PathBuf;

mod file;
mod messages;
mod reader;
mod value;

pub use self::file::FitFile;
pub use self::messages::DefinedMessageType;
pub use self::value::{TryFrom, Value, ValueError};

/// Reads the file and returns a FitFile.
pub fn read(path: PathBuf) -> FitFile {
    file::FitFile::read(path)
}

#[cfg(test)]
pub mod tests {
    use super::FitFile;
    use crate::reader::Reader;
    use std::path::PathBuf;

    pub fn fit_setup() -> Reader {
        let path = PathBuf::from("fits/garmin_1000.fit");
        Reader::new(path)
    }

    #[test]
    fn it_reads_garmin_1000_file() {
        let filepath = PathBuf::from("fits/garmin_1000.fit");
        let _ = FitFile::read(filepath);
    }
    #[test]
    fn it_reads_garmin_520_file() {
        let filepath = PathBuf::from("fits/garmin_520_long.fit");
        let _ = FitFile::read(filepath);
    }
    #[test]
    fn it_reads_garmin_520_file_with_power() {
        let filepath = PathBuf::from("fits/garmin_520_power.fit");
        let _ = FitFile::read(filepath);
    }
    #[test]
    fn it_reads_trainerroad_file() {
        let filepath = PathBuf::from("fits/trainerroad.fit");
        let _ = FitFile::read(filepath);
    }
    #[test]
    fn it_reads_wahoo_file() {
        let filepath = PathBuf::from("fits/wahoo.fit");
        let _ = FitFile::read(filepath);
    }
    #[test]
    #[should_panic]
    fn it_panics_reading_wahoo_file_with_developer_fields() {
        let filepath = PathBuf::from("fits/wahoo_dev_fields.fit");
        let _ = FitFile::read(filepath);
    }
}
