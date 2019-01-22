use std::io::{BufReader, Error, Read, Seek, SeekFrom, Take};

use crate::consts::*;
use crate::definitions::{BaseType, DataField};
use crate::reader::{Endian, Reader};

const FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10000000;
const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00011111;

#[derive(Debug)]
pub struct DefinitionRecord {
    pub architecture: Endian,
    pub global_message_num: u16,
    pub number_of_fields: u8,
    pub field_defs: Vec<FieldDefinition>,
    dev_field_defs: Vec<u8>,
}
impl DefinitionRecord {
    pub fn new(reader: &mut Reader, dev_fields: bool) -> Self {
        if dev_fields {
            panic!("has developer fields")
        }
        reader.skip(1); // skip reserved byte
        let endian = match reader.byte() {
            Ok(1) => Endian::Big,
            Ok(0) => Endian::Little,
            _ => panic!("some error"),
        };
        let global_message_num = reader.u16(&endian).unwrap();
        let number_of_fields = reader.byte().unwrap();
        let mut field_defs = Vec::with_capacity(number_of_fields as usize);
        for i in 0..number_of_fields {
            let mut buf = reader.bytes(3).unwrap();
            let field = FieldDefinition::new(&buf);
            field_defs.push(field);
        }
        DefinitionRecord {
            architecture: endian,
            global_message_num: global_message_num,
            number_of_fields: number_of_fields,
            field_defs: field_defs,
            dev_field_defs: Vec::new(),
        }
    }
    pub fn new_record(&self, reader: &mut Reader) -> Option<Box<impl fit::MessageType + ?Sized>> {
        let mut record = fit::new_record(&self.global_message_num);
        for fd in &self.field_defs {
            let data_field = DataField::new(reader, &self.architecture, &fd);
            if let Some(vals) = data_field.values {
                match &mut record {
                    Some(r) => r.add_value(data_field.id, vals[0].clone()),
                    None => (),
                }
            }
        }
        record
    }
}

#[derive(Debug)]
pub struct FieldDefinition {
    pub field_def_number: u16,
    pub size: u8,
    endianness: bool,
    pub base_type: BaseType,
}
impl FieldDefinition {
    fn new(buf: &[u8]) -> Self {
        let base_num = buf[2] & FIELD_DEFINITION_BASE_NUMBER;
        let endianness = (buf[2] & FIELD_DEFINITION_ARCHITECTURE) == FIELD_DEFINITION_ARCHITECTURE;
        return Self {
            field_def_number: buf[0].into(),
            size: buf[1],
            endianness: endianness,
            base_type: BaseType::get(base_num),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn it_reads_a_definition() {
        let mut reader = fit_setup();
        reader.skip(14); // FileHeader
        reader.skip(1); // HeaderByte
        let definition = DefinitionRecord::new(&mut reader, false);
        // now 41
        // println!("def: {:?}", definition);
    }
}
