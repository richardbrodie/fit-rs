use fit_sdk::consts::*;
use fit_sdk::BaseType;

use super::definition_record::FieldDefinition;
use super::reader::{Endian, Reader};
use super::Value;

#[derive(Debug)]
pub struct DataField {
    pub id: u16,
    pub value: Option<Value>,
}

impl DataField {
    pub fn new(reader: &mut Reader, endianness: &Endian, field_def: &FieldDefinition) -> Self {
        let val: Option<Value> = match field_def.base_type {
            BaseType::ENUM => read_values(field_def, &ENUM_TYPE, || reader.byte()),
            BaseType::BYTE => read_values(field_def, &BYTE_TYPE, || reader.byte()),
            BaseType::UINT8 => read_values(field_def, &UINT8_TYPE, || reader.byte()),
            BaseType::UINT16 => read_values(field_def, &UINT16_TYPE, || reader.u16(endianness)),
            BaseType::UINT32 => read_values(field_def, &UINT32_TYPE, || reader.u32(endianness)),
            BaseType::UINT64 => read_values(field_def, &UINT64_TYPE, || reader.u64(endianness)),
            BaseType::SINT8 => read_values(field_def, &SINT8_TYPE, || reader.i8()),
            BaseType::SINT16 => read_values(field_def, &SINT16_TYPE, || reader.i16(endianness)),
            BaseType::SINT32 => read_values(field_def, &SINT32_TYPE, || reader.i32(endianness)),
            BaseType::SINT64 => read_values(field_def, &SINT64_TYPE, || reader.i64(endianness)),
            BaseType::FLOAT32 => read_values(field_def, &FLOAT32_TYPE, || reader.f32(endianness)),
            BaseType::FLOAT64 => read_values(field_def, &FLOAT64_TYPE, || reader.f64(endianness)),
            BaseType::STRING => {
                let number_of_values = field_def.size / STRING_TYPE.byte_size;
                let invalid = STRING_TYPE.invalidvalue.u8();
                let str_vec: Vec<u8> = (0..number_of_values)
                    .filter_map(|_| {
                        reader
                            .byte()
                            .ok()
                            .and_then(|v| if v == invalid { Some(v) } else { None })
                    })
                    .collect();
                String::from_utf8(str_vec).ok().map(|v| v.into())
            }
            BaseType::UINT8Z => read_values(field_def, &UINT8Z_TYPE, || reader.byte()),
            BaseType::UINT16Z => read_values(field_def, &UINT16Z_TYPE, || reader.u16(endianness)),
            BaseType::UINT32Z => read_values(field_def, &UINT32Z_TYPE, || reader.u32(endianness)),
            BaseType::UINT64Z => read_values(field_def, &UINT64Z_TYPE, || reader.u64(endianness)),
        };

        Self {
            id: field_def.field_def_number,
            value: val,
        }
    }
}

// private

fn read_values<F, T: Into<Value> + std::cmp::PartialEq>(
    fdef: &FieldDefinition,
    typ: &BaseTypeStruct,
    mut fun: F,
) -> Option<Value>
where
    F: FnMut() -> Result<T, crate::Error>,
{
    let number_of_values = (fdef.size / typ.byte_size) as usize;
    let mut ind_fun = || {
        fun()
            .ok()
            .map(|v| v.into())
            .filter(|v| v != &typ.invalidvalue)
    };

    if number_of_values == 1 {
        ind_fun()
    } else if number_of_values > 1 {
        let mut v: Vec<_> = (0..number_of_values).filter_map(|_| ind_fun()).collect();
        if v.is_empty() {
            None
        } else {
            v.shrink_to_fit();
            Some(v.into())
        }
    } else {
        panic!("tried to read 0 values")
    }
}
