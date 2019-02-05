// #![allow(unused)]
use std::path::PathBuf;

mod file;
mod messages;
mod reader;
mod value;

pub use self::file::FitFile;
pub use self::messages::{new_record, DefinedMessageType};
pub use self::reader::{Endian, Reader};
pub use self::value::{TryFrom, Value, ValueError};

pub fn read(path: PathBuf) -> FitFile {
    file::FitFile::read(path)
}

#[cfg(test)]
pub mod tests {
    use super::{FitFile, Reader};
    use std::path::PathBuf;

    pub fn fit_setup() -> Reader {
        let path = PathBuf::from("fits/working_garmin.fit");
        Reader::new(path)
    }

    #[test]
    fn it_reads_whole_file() {
        let filepath = PathBuf::from("fits/working_garmin.fit");
        let _ = FitFile::read(filepath);
    }
}
