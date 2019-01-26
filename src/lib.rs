#![allow(unused)]
use std::collections::HashMap;
use std::io::{BufReader, Error, Read, Seek, SeekFrom, Take};

mod file;
mod messages;
mod reader;
mod value;

pub use self::file::FitFile;
pub use self::messages::{new_record, MessageType};
pub use self::reader::{Endian, Reader};
pub use self::value::{TryFrom, Value, ValueError};

#[cfg(test)]
pub mod tests {
    use super::Reader;
    use std::path::PathBuf;

    pub fn fit_setup() -> Reader {
        let path = PathBuf::from("fits/working_garmin.fit");
        Reader::new(path)
    }
}
