use super::data_record::DataRecord;
use super::definition_record::DefinitionRecord;
use super::file_header::FileHeader;
use super::reader::Reader;
use std::collections::HashMap;
use std::io::Error;
use std::path::PathBuf;

const TIMESTAMP_HEADER_MASK: u8 = 0x80;
const TIMESTAMP_MESSAGE_TYPE_MASK: u8 = 0x60;
const TIMESTAMP_OFFSET_MASK: u8 = 0x1F;
const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

#[derive(Debug)]
struct HeaderByte {
    byte: u8,
}
impl HeaderByte {
    fn new(reader: &mut Reader) -> Result<Self, Error> {
        Ok(Self {
            byte: reader.byte()?,
        })
    }
    fn is_timestamp(&self) -> bool {
        (self.byte & TIMESTAMP_HEADER_MASK) == TIMESTAMP_HEADER_MASK
    }
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

pub struct Fit {
    file_header: FileHeader,
    definitions: HashMap<u8, DefinitionRecord>,
}
impl Fit {
    pub fn read_file(path: PathBuf) {
        let mut reader = Reader::new(path);
        let mut definitions: HashMap<u8, DefinitionRecord> = HashMap::new();
        let mut records: Vec<DataRecord> = Vec::new();

        let header = FileHeader::new(&mut reader).unwrap();

        while reader.pos().unwrap() < u64::from(header.file_length()) {
            let h = HeaderByte::new(&mut reader).unwrap();
            if h.is_definition() {
                let definition = DefinitionRecord::new(&mut reader, h.has_developer_fields());
                definitions.insert(h.local_msg_number(), definition);
            } else {
                match definitions.get(&h.local_msg_number()) {
                    Some(def) => {
                        let data = DataRecord::new(&mut reader, &def);
                        records.push(data);
                    }
                    None => {
                        println!(
                            "keys found: {:?}, key wanted: {}",
                            definitions.keys(),
                            &h.local_msg_number()
                        );
                        panic!("could not find definition");
                    }
                }
            }
        }
    }

    // fn next_record(
    //     reader: &mut Reader,
    //     definitions: &mut HashMap<u8, DefinitionRecord>,
    // ) -> Result<RecordType, std::io::Error> {
    //     let h = HeaderByte::new(reader).unwrap();
    //     if h.is_definition() {
    //         let definition = DefinitionRecord::new(reader, h.has_developer_fields());
    //         Ok(RecordType::Definition(definition))
    //     } else {
    //         let definition: DefinitionRecord;
    //         let data = DataRecord::new(reader, &definition);
    //         Ok(RecordType::Data(data))
    //     }
    // }
}

enum RecordType {
    Definition(DefinitionRecord),
    Data(DataRecord),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn it_reads_header_byte() {
        let mut reader = fit_setup();
        reader.skip(14); // FileHeader
        let header_byte = HeaderByte::new(&mut reader).unwrap();
        assert_eq!(header_byte.is_definition(), true);
    }

    #[test]
    fn it_reads_whole_file() {
        let mut reader = fit_setup();
        let filepath = PathBuf::from("fits/working_garmin.fit");
        let fit = Fit::read_file(filepath);
    }
}
