#![allow(unused)]
use std::collections::HashMap;
use std::io::{BufReader, Error, Read, Seek, SeekFrom, Take};

mod consts;
mod definitions;
mod reader;
pub use self::definitions::FitFile;
use self::reader::{Endian, Reader};

#[cfg(test)]
pub mod tests {
    use super::Reader;
    use std::path::PathBuf;

    pub fn fit_setup() -> Reader {
        let path = PathBuf::from("fits/working_garmin.fit");
        Reader::new(path)
    }
}
