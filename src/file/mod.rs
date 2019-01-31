mod base_type;
mod consts;
mod data_field;
mod definition_record;
mod file_header;
mod fit_file;

pub use self::fit_file::FitFile;

use crate::Reader;

// const TIMESTAMP_HEADER_MASK: u8 = 0x80;
// const TIMESTAMP_MESSAGE_TYPE_MASK: u8 = 0x60;
// const TIMESTAMP_OFFSET_MASK: u8 = 0x1F;
const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

#[derive(Debug)]
struct RecordHeaderByte {
    byte: u8,
}
impl RecordHeaderByte {
    fn new(reader: &mut Reader) -> Result<Self, ()> {
        reader.byte().map(|b| Self { byte: b }).map_err(|_| ())
    }
    // fn is_timestamp(&self) -> bool {
    //     (self.byte & TIMESTAMP_HEADER_MASK) == TIMESTAMP_HEADER_MASK
    // }
    fn is_definition(&self) -> bool {
        (self.byte & DEFINITION_HEADER_MASK) == DEFINITION_HEADER_MASK
    }
    fn has_developer_fields(&self) -> bool {
        (self.byte & DEVELOPER_FIELDS_MASK) == DEVELOPER_FIELDS_MASK
    }
    fn local_msg_number(&self) -> u8 {
        self.byte & LOCAL_MESSAGE_NUMBER_MASK
    }
}

#[cfg(test)]
mod tests {
    use super::definition_record::DefinitionRecord;
    use super::file_header::FileHeader;
    use super::{FitFile, RecordHeaderByte};
    use crate::tests::*;
    use std::path::PathBuf;

    #[test]
    fn it_reads_header_byte() {
        let mut reader = fit_setup();
        reader.skip(14); // FileHeader
        let header_byte = RecordHeaderByte::new(&mut reader).unwrap();
        assert_eq!(header_byte.is_definition(), true);
    }

    #[test]
    fn it_reads_whole_file() {
        let filepath = PathBuf::from("fits/working_garmin.fit");
        let _ = FitFile::read(filepath);
    }

    #[test]
    fn it_reads_fileheader() {
        let mut reader = fit_setup();
        let fileheader = FileHeader::new(&mut reader).unwrap();
        assert_eq!(fileheader.num_record_bytes, 191877);
    }

    #[test]
    fn it_reads_a_definition() {
        let mut reader = fit_setup();
        reader.skip(14); // FileHeader
        reader.skip(1); // HeaderByte
        let definition = DefinitionRecord::new(&mut reader, false);
        // now 41
        assert_eq!(definition.number_of_fields, 7);
    }
}
