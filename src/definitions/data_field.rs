use super::definition_record::FieldDefinition;
use crate::consts::*;
use crate::reader::{Endian, Reader};
use fit::Value;

#[derive(Debug)]
pub struct DataField {
    pub id: u16,
    pub values: Option<Vec<Value>>,
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
            values: match vals.is_empty() {
                true => None,
                false => Some(vals),
            },
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
    (0..number_of_values).filter_map(|_| fun()).collect()
}

#[derive(Debug)]
pub enum BaseType {
    ENUM,
    SINT8,
    UINT8,
    SINT16,
    UINT16,
    SINT32,
    UINT32,
    STRING,
    FLOAT32,
    FLOAT64,
    UINT8Z,
    UINT16Z,
    UINT32Z,
    BYTE,
    SINT64,
    UINT64,
    UINT64Z,
}
impl BaseType {
    pub fn get(num: u8) -> Self {
        match num {
            0 => BaseType::ENUM,
            1 => BaseType::SINT8,
            2 => BaseType::UINT8,
            3 => BaseType::SINT16,
            4 => BaseType::UINT16,
            5 => BaseType::SINT32,
            6 => BaseType::UINT32,
            7 => BaseType::STRING,
            8 => BaseType::FLOAT32,
            9 => BaseType::FLOAT64,
            10 => BaseType::UINT8Z,
            11 => BaseType::UINT16Z,
            12 => BaseType::UINT32Z,
            13 => BaseType::BYTE,
            14 => BaseType::SINT64,
            15 => BaseType::UINT64,
            16 => BaseType::UINT64Z,
            _ => panic!("not an option"),
        }
    }
}
