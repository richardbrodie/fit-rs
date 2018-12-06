#![allow(unused)]
use std::collections::HashMap;
use std::io::{BufReader, Error, Read, Seek, SeekFrom, Take};

mod consts;
mod data_record;
mod definition_record;
mod file_header;
pub mod fit;
pub mod reader;
use self::consts::*;
use self::data_record::*;
use self::definition_record::*;
use self::file_header::*;
use self::fit::*;
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
