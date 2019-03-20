mod base_type;
mod consts;
mod data_field;
mod definition_record;
mod file_header;
mod fit_file;

pub use self::data_field::DataField;
pub use self::definition_record::DefinitionRecord;
pub use self::file_header::FileHeader;
pub use self::fit_file::FitFile;
use crate::reader::Reader;

const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

#[derive(Debug)]
pub struct RecordHeaderByte {
    byte: u8,
}
impl RecordHeaderByte {
    pub fn new(reader: &mut Reader) -> Result<Self, crate::Error> {
        reader.byte().map(|b| Self { byte: b })
    }
    pub fn is_definition(&self) -> bool {
        (self.byte & DEFINITION_HEADER_MASK) == DEFINITION_HEADER_MASK
    }
    pub fn has_developer_fields(&self) -> bool {
        (self.byte & DEVELOPER_FIELDS_MASK) == DEVELOPER_FIELDS_MASK
    }
    pub fn local_msg_number(&self) -> u8 {
        self.byte & LOCAL_MESSAGE_NUMBER_MASK
    }
}

#[cfg(test)]
mod tests {
    use super::RecordHeaderByte;
    use crate::tests::fit_setup;

    #[test]
    fn it_reads_header_byte() {
        let mut reader = fit_setup().unwrap();
        reader.skip(14); // FileHeader
        let header_byte = RecordHeaderByte::new(&mut reader).unwrap();
        assert_eq!(header_byte.is_definition(), true);
    }
}
