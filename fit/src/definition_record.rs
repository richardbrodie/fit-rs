use fit_sdk::BaseType;

use crate::reader::{Endian, Reader};
use crate::Error;

const FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10_000_000;
const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00_011_111;

#[derive(Debug)]
pub struct DefinitionRecord {
    pub architecture: Endian,
    pub global_message_num: u16,
    pub number_of_fields: u8,
    pub field_defs: Vec<FieldDefinition>,
    dev_field_defs: Vec<u8>,
}
impl DefinitionRecord {
    pub fn new(reader: &mut Reader, _dev_fields: bool) -> Result<Self, Error> {
        reader.skip(1); // skip reserved byte
        let endian = match reader.byte() {
            Ok(1) => Endian::Big,
            Ok(0) => Endian::Little,
            _ => Err(crate::ErrorKind::UnexpectedValue)?,
        };
        let global_message_num = reader.u16(&endian)?;
        let number_of_fields = reader.byte()?;
        (0..number_of_fields)
            .map(|_| reader.bytes(3).map(|buf| FieldDefinition::new(&buf)))
            .collect::<Result<Vec<FieldDefinition>, Error>>()
            .map(|field_defs| DefinitionRecord {
                architecture: endian,
                global_message_num,
                number_of_fields,
                field_defs,
                dev_field_defs: Vec::new(),
            })
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
        Self {
            field_def_number: buf[0].into(),
            size: buf[1],
            endianness,
            base_type: BaseType::get(base_num),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DefinitionRecord;
    use crate::tests::fit_setup;

    #[test]
    fn it_reads_a_definition() {
        let mut reader = fit_setup().unwrap();
        reader.skip(14); // FileHeader
        reader.skip(1); // HeaderByte
        let definition = DefinitionRecord::new(&mut reader, false).unwrap();
        // now 41
        assert_eq!(definition.number_of_fields, 7);
    }
}
