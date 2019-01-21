use fit::Value;
use std::io::{BufReader, Error, Read, Seek, SeekFrom, Take};

use super::definition_record::{BaseType, DefinitionRecord, FieldDefinition};
use crate::consts::*;
use crate::reader::{Endian, Reader};

#[derive(Debug)]
pub struct DataRecord {
    pub global_message_num: u16,
    pub fields: Vec<DataField>,
}
impl DataRecord {
    pub fn new(reader: &mut Reader, definition: &DefinitionRecord) -> Self {
        let mut fields = Vec::with_capacity(definition.number_of_fields as usize);
        for fd in &definition.field_defs {
            let data_field = DataField::new(reader, &definition.architecture, &fd);
            // println!("{:?}", data_field);
            fields.push(data_field);
        }
        Self {
            global_message_num: definition.global_message_num,
            fields: fields,
        }
    }
}

#[derive(Debug)]
pub struct DataField {
    pub id: u16,
    pub values: Vec<Value>,
}

impl DataField {
    pub fn new(reader: &mut Reader, endianness: &Endian, field_def: &FieldDefinition) -> Self {
        let vals = match field_def.base_type {
            BaseType::ENUM => {
                let typedef = &ENUM_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .byte()
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u8))
                        .map(|val| Value::Enum(val))
                })
            }
            BaseType::BYTE => {
                let typedef = &BYTE_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .byte()
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u8))
                        .map(|val| Value::U8(val))
                })
            }
            BaseType::UINT8 => {
                let typedef = &UINT8_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .byte()
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u8))
                        .map(|val| Value::U8(val))
                })
            }
            BaseType::UINT16 => {
                let typedef = &UINT16_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .u16(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u16))
                        .map(|val| Value::U16(val))
                })
            }
            BaseType::UINT32 => {
                let typedef = &UINT32_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .u32(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u32))
                        .map(|val| Value::U32(val))
                })
            }
            BaseType::UINT64 => {
                let typedef = &UINT64_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .u64(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue))
                        .map(|val| Value::U64(val))
                })
            }
            BaseType::SINT8 => {
                let typedef = &SINT8_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .i8()
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as i8))
                        .map(|val| Value::I8(val))
                })
            }
            BaseType::SINT16 => {
                let typedef = &SINT16_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .i16(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as i16))
                        .map(|val| Value::I16(val))
                })
            }
            BaseType::SINT32 => {
                let typedef = &SINT32_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .i32(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as i32))
                        .map(|val| Value::I32(val))
                })
            }
            BaseType::SINT64 => {
                let typedef = &SINT64_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .i64(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as i64))
                        .map(|val| Value::I64(val))
                })
            }
            BaseType::FLOAT32 => {
                let typedef = &FLOAT32_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .f32(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as f32))
                        .map(|val| Value::F32(val))
                })
            }
            BaseType::FLOAT64 => {
                let typedef = &FLOAT64_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .f64(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as f64))
                        .map(|val| Value::F64(val))
                })
            }
            BaseType::STRING => {
                let typedef = &STRING_TYPE;
                let number_of_values = field_def.size / typedef.byte_size;
                let str_vec = (0..number_of_values)
                    .into_iter()
                    .filter_map(|_| {
                        reader
                            .byte()
                            .ok()
                            .and_then(|v| is_valid(v, typedef.invalidvalue as u8))
                    })
                    .collect();
                String::from_utf8(str_vec)
                    .iter()
                    .map(|s| Value::Str(s.to_string()))
                    .collect()
            }
            BaseType::UINT8Z => {
                let typedef = &UINT8Z_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .byte()
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u8))
                        .map(|val| Value::U8(val))
                })
            }
            BaseType::UINT16Z => {
                let typedef = &UINT16Z_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .u16(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u16))
                        .map(|val| Value::U16(val))
                })
            }
            BaseType::UINT32Z => {
                let typedef = &UINT32Z_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .u32(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue as u32))
                        .map(|val| Value::U32(val))
                })
            }
            BaseType::UINT64Z => {
                let typedef = &UINT64Z_TYPE;
                read_values(field_def.size, typedef.byte_size, || {
                    reader
                        .u64(endianness)
                        .ok()
                        .and_then(|v| is_valid(v, typedef.invalidvalue))
                        .map(|val| Value::U64(val))
                })
            }
        };

        Self {
            id: field_def.field_def_number,
            values: vals,
        }
    }
}

// private

fn is_valid<T: PartialEq>(val: T, invalid: T) -> Option<T> {
    match val == (invalid as T) {
        true => None,
        false => Some(val),
    }
}

fn read_values<T>(field_size: u8, type_size: u8, mut fun: T) -> Vec<Value>
where
    T: FnMut() -> Option<Value>,
{
    let number_of_values = (field_size / type_size) as usize;
    let mut v: Vec<Value> = Vec::with_capacity(number_of_values);
    for _ in 0..number_of_values {
        match fun() {
            Some(val) => v.push(val),
            _ => (),
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn it_reads_a_data_record() {
        let mut reader = fit_setup();
        reader.skip(14); // FileHeader
        reader.skip(1); // HeaderByte
        let definition = DefinitionRecord::new(&mut reader, false);
        reader.skip(1); // HeaderByte
        let data = DataRecord::new(&mut reader, &definition);
        assert_eq!(data.fields[0].values[0], Value::U32(3902378567)); // base type 12
        assert_eq!(data.fields[1].values[0], Value::U32(849790468));
        assert_eq!(data.fields[3].values[0], Value::U16(1));
        assert_eq!(
            fit::get_message_struct(&definition.global_message_num)
                .unwrap()
                .msg_name(),
            "File Id"
        );
    }
}
