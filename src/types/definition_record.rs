use std::io::{Read, Seek};

use crate::{
    developer_fields::DeveloperFieldDefinition,
    io::{read_u16, read_u8, skip_bytes, Endianness},
};

use super::field_definition::FieldDefinition;

//////////
//// DefinitionRecord
//////////
#[derive(Clone, Debug, PartialEq)]
pub struct DefinitionRecord {
    pub endianness: Endianness,
    pub global_message_number: u16,
    pub field_definitions: Vec<FieldDefinition>,
    pub developer_fields: Option<Vec<DeveloperFieldDefinition>>,
}
impl DefinitionRecord {
    pub fn new<R>(map: &mut R, dev_fields: bool) -> Self
    where
        R: Read + Seek,
    {
        skip_bytes(map, 1);
        let mut buffer: Vec<FieldDefinition> = Vec::new();
        let endian = match read_u8(map) {
            1 => Endianness::Big,
            0 => Endianness::Little,
            _ => panic!("unexpected endian byte"),
        };
        let global_message_number = read_u16(map, endian);
        let number_of_fields = read_u8(map);

        for _ in 0..number_of_fields {
            buffer.push(FieldDefinition::new(map));
        }
        let dev_fields: Option<Vec<DeveloperFieldDefinition>> = if dev_fields {
            let number_of_fields = read_u8(map);
            Some(
                (0..number_of_fields)
                    .map(|_| DeveloperFieldDefinition::new(map))
                    .collect(),
            )
        } else {
            None
        };

        DefinitionRecord {
            endianness: endian,
            global_message_number,
            field_definitions: buffer,
            developer_fields: dev_fields,
        }
    }
}
