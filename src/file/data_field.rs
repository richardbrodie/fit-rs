use super::base_type::BaseType;
use super::consts::*;
use super::definition_record::FieldDefinition;

use crate::reader::{Endian, Reader};
use crate::Value;

#[derive(Debug)]
pub struct DataField {
    pub id: u16,
    pub values: Option<Vec<Value>>,
}

impl DataField {
    pub fn new(reader: &mut Reader, endianness: &Endian, field_def: &FieldDefinition) -> Self {
        let vals = match field_def.base_type {
            BaseType::ENUM => read_values(field_def.size, ENUM_TYPE.byte_size, || {
                reader
                    .byte()
                    .ok()
                    .and_then(|v| is_valid(v, ENUM_TYPE.invalidvalue as u8))
                    .map(|val| Value::Enum(val))
            }),
            BaseType::BYTE => read_values(field_def.size, BYTE_TYPE.byte_size, || {
                reader
                    .byte()
                    .ok()
                    .and_then(|v| is_valid(v, BYTE_TYPE.invalidvalue as u8))
                    .map(|val| val.into())
            }),
            BaseType::UINT8 => read_values(field_def.size, UINT8_TYPE.byte_size, || {
                reader
                    .byte()
                    .ok()
                    .and_then(|v| is_valid(v, UINT8_TYPE.invalidvalue as u8))
                    .map(|val| val.into())
            }),
            BaseType::UINT16 => read_values(field_def.size, UINT16_TYPE.byte_size, || {
                reader
                    .u16(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, UINT16_TYPE.invalidvalue as u16))
                    .map(|val| val.into())
            }),
            BaseType::UINT32 => read_values(field_def.size, UINT32_TYPE.byte_size, || {
                reader
                    .u32(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, UINT32_TYPE.invalidvalue as u32))
                    .map(|val| val.into())
            }),
            BaseType::UINT64 => read_values(field_def.size, UINT64_TYPE.byte_size, || {
                reader
                    .u64(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, UINT64_TYPE.invalidvalue))
                    .map(|val| val.into())
            }),
            BaseType::SINT8 => read_values(field_def.size, SINT8_TYPE.byte_size, || {
                reader
                    .i8()
                    .ok()
                    .and_then(|v| is_valid(v, SINT8_TYPE.invalidvalue as i8))
                    .map(|val| val.into())
            }),
            BaseType::SINT16 => read_values(field_def.size, SINT16_TYPE.byte_size, || {
                reader
                    .i16(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, SINT16_TYPE.invalidvalue as i16))
                    .map(|val| val.into())
            }),
            BaseType::SINT32 => read_values(field_def.size, SINT32_TYPE.byte_size, || {
                reader
                    .i32(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, SINT32_TYPE.invalidvalue as i32))
                    .map(|val| val.into())
            }),
            BaseType::SINT64 => read_values(field_def.size, SINT64_TYPE.byte_size, || {
                reader
                    .i64(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, SINT64_TYPE.invalidvalue as i64))
                    .map(|val| val.into())
            }),
            BaseType::FLOAT32 => read_values(field_def.size, FLOAT32_TYPE.byte_size, || {
                reader
                    .f32(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, FLOAT32_TYPE.invalidvalue as f32))
                    .map(|val| val.into())
            }),
            BaseType::FLOAT64 => read_values(field_def.size, FLOAT64_TYPE.byte_size, || {
                reader
                    .f64(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, FLOAT64_TYPE.invalidvalue as f64))
                    .map(|val| val.into())
            }),
            BaseType::STRING => {
                let number_of_values = field_def.size / STRING_TYPE.byte_size;
                let str_vec: Vec<u8> = (0..number_of_values)
                    .into_iter()
                    .filter_map(|_| {
                        reader
                            .byte()
                            .ok()
                            .and_then(|v| is_valid(v, STRING_TYPE.invalidvalue as u8))
                    })
                    .collect();
                std::str::from_utf8(&str_vec)
                    .into_iter()
                    .map(|s| s.into())
                    .collect()
            }
            BaseType::UINT8Z => read_values(field_def.size, UINT8Z_TYPE.byte_size, || {
                reader
                    .byte()
                    .ok()
                    .and_then(|v| is_valid(v, UINT8Z_TYPE.invalidvalue as u8))
                    .map(|val| val.into())
            }),
            BaseType::UINT16Z => read_values(field_def.size, UINT16Z_TYPE.byte_size, || {
                reader
                    .u16(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, UINT16Z_TYPE.invalidvalue as u16))
                    .map(|val| val.into())
            }),
            BaseType::UINT32Z => read_values(field_def.size, UINT32Z_TYPE.byte_size, || {
                reader
                    .u32(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, UINT32Z_TYPE.invalidvalue as u32))
                    .map(|val| val.into())
            }),
            BaseType::UINT64Z => read_values(field_def.size, UINT64Z_TYPE.byte_size, || {
                reader
                    .u64(endianness)
                    .ok()
                    .and_then(|v| is_valid(v, UINT64Z_TYPE.invalidvalue))
                    .map(|val| val.into())
            }),
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
    match val == invalid {
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
