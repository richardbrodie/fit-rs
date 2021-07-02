mod consts;
mod developer_fields;
mod io;
mod types;
mod value;

use consts::*;
use copyless::VecHelper;
use developer_fields::DeveloperFieldDescription;
use fitsdk::{
    get_field_offset_fn, get_field_scale_fn, get_field_string_value_fn, get_field_type_fn,
    get_message_timestamp_field, get_message_type, FieldType, MessageType,
};
use io::*;
use memmap::{Mmap, MmapOptions};
use std::collections::VecDeque;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::{fs::File, path::PathBuf};
use types::data_field::DataField;
use types::definition_record::DefinitionRecord;
use types::dev_data_field::DevDataField;
use types::file_header::FileHeader;
use types::header_byte::HeaderByte;
use types::message::Message;
pub use value::Value;

//////////
//// Fit
//////////

pub struct Fit {
    file_header: FileHeader,
    data_len: u64,
    buf: Cursor<Mmap>,
    queue: VecDeque<(u8, DefinitionRecord)>,
    developer_fields: Vec<DeveloperFieldDescription>,
    last_timestamp: u32,
}
impl Fit {
    pub fn new(path: &PathBuf) -> Self {
        let file = File::open(path).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let mut buf = Cursor::new(mmap);

        let fh = FileHeader::new(&mut buf);
        Self {
            data_len: u64::from(&fh.num_record_bytes + 14),
            file_header: fh,
            buf,
            queue: VecDeque::new(),
            developer_fields: Vec::new(),
            last_timestamp: 0,
        }
    }
    pub fn file_header(&self) -> &FileHeader {
        &self.file_header
    }
}
impl Iterator for Fit {
    type Item = Message;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let r = self.buf.seek(SeekFrom::Current(0)).unwrap();
            if r > self.data_len {
                return None;
            }
            let h = HeaderByte::new(&mut self.buf);
            if h.definition {
                let d = DefinitionRecord::new(&mut self.buf, h.dev_fields);
                self.queue.push_front((h.local_num, d));
            } else {
                // if no definition is found, skip this loop
                let definition = match self.queue.iter().find(|x| x.0 == h.local_num) {
                    None => continue,
                    Some((_, def)) => def,
                };
                let message_type = get_message_type(definition.global_message_number);
                let mut dev_fields: Option<Vec<DevDataField>> = None;
                let mut values = Vec::with_capacity(definition.field_definitions.len());

                // read all the values for this reacord type's defined fields
                for fd in definition.field_definitions.iter() {
                    if message_type == MessageType::None {
                        skip_bytes(&mut self.buf, fd.size);
                    } else if let Some(data) =
                        read_next_field(fd.size, fd.base_type, definition.endianness, &mut self.buf)
                    {
                        values
                            .alloc()
                            .init(DataField::new(fd.definition_number, data));
                    }
                }

                // if this is a developer field definition
                if message_type == MessageType::FieldDescription {
                    let d = DeveloperFieldDescription::new(values);
                    self.developer_fields.push(d);
                    continue;
                }

                // this is not a valid message, so there's no more processing to do this loop
                if message_type == MessageType::None || values.is_empty() {
                    continue;
                }

                // if this record is a message that contains developer-defined fields read those too
                if let Some(dev_field_defs) = &definition.developer_fields {
                    let mut temp_dev_fields = Vec::new();
                    for df in dev_field_defs.iter() {
                        for e in &self.developer_fields {
                            if e.developer_data_index == 1 {
                                if let Some(v) = read_next_field(
                                    e.fit_base_type,
                                    df.size,
                                    definition.endianness,
                                    &mut self.buf,
                                ) {
                                    temp_dev_fields.push(DevDataField::new(
                                        df.developer_data_index,
                                        df.field_number,
                                        v,
                                    ));
                                }
                            }
                        }
                    }
                    dev_fields = Some(temp_dev_fields);
                }

                // check each value in case the raw value needs further processing
                let scales = get_field_scale_fn(message_type);
                let offsets = get_field_offset_fn(message_type);
                let fields = get_field_type_fn(message_type);
                for v in &mut values {
                    process_value(v, fields, scales, offsets);
                }

                // some FIT files use a compressed timestamp to save a little more space
                if let Some(time_offset) = h.compressed_timestamp() {
                    let mut timestamp = self.last_timestamp
                        & COMPRESSED_HEADER_LAST_TIMESTAMP_MASK + u32::from(time_offset);
                    if time_offset
                        < (self.last_timestamp as u8 & COMPRESSED_HEADER_TIME_OFFSET_MASK)
                    {
                        timestamp += COMPRESSED_HEADER_TIME_OFFSET_ROLLOVER
                    };

                    if let Some(timestamp_field_number) = get_message_timestamp_field(message_type)
                    {
                        values.alloc().init(DataField::new(
                            timestamp_field_number,
                            Value::Time(timestamp + PSEUDO_EPOCH),
                        ));
                    }
                }

                // if any values were invalid we have a vec that's now too long
                values.shrink_to_fit();
                return Some(Message {
                    values,
                    kind: message_type,
                    dev_values: dev_fields,
                });
            }
        }
    }
}

#[allow(clippy::cognitive_complexity)]
fn read_next_field<R>(size: u8, base_type: u8, endianness: Endianness, map: &mut R) -> Option<Value>
where
    R: Read + Seek,
{
    match base_type {
        0 | 13 => {
            // enum / byte
            if size > 1 {
                println!("0/13:enum/byte: {}", size);
                skip_bytes(map, size);
                None
            } else {
                match read_u8(map) {
                    0xFF => None,
                    v => Some(Value::U8(v)),
                }
            }
        }
        1 => {
            // sint8
            if size > 1 {
                println!("1 i8: {}", size);
                skip_bytes(map, size);
                None
            } else {
                match read_i8(map) {
                    0x7F => None,
                    v => Some(Value::I8(v)),
                }
            }
        }
        2 => {
            // uint8
            if size > 1 {
                let mut buf: Vec<_> = Vec::with_capacity(size.into());
                let _ = map.take(size.into()).read_to_end(&mut buf);
                buf.retain(|x| *x != 0xFF);
                if buf.is_empty() {
                    None
                } else {
                    Some(Value::ArrU8(buf))
                }
            } else {
                match read_u8(map) {
                    0xFF => None,
                    v => Some(Value::U8(v)),
                }
            }
        }
        3 => {
            // sint16
            let number_of_values = size / 2;
            if number_of_values > 1 {
                println!("3 i16: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_i16(map, endianness);
                if val == 0x7FFF {
                    None
                } else {
                    Some(Value::I16(val))
                }
            }
        }
        4 => {
            // uint16
            let number_of_values = size / 2;
            if number_of_values > 1 {
                let c: Vec<_> = (0..number_of_values)
                    .filter_map(|_| match read_u16(map, endianness) {
                        0xFFFF => None,
                        v => Some(v),
                    })
                    .collect();
                if c.is_empty() {
                    None
                } else {
                    Some(Value::ArrU16(c))
                }
            } else {
                let val = read_u16(map, endianness);
                if val == 0xFFFF {
                    None
                } else {
                    Some(Value::U16(val))
                }
            }
        }
        5 => {
            // sint32
            let number_of_values = size / 4;
            if number_of_values > 1 {
                println!("5 i32: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_i32(map, endianness);
                if val == 0x7F_FFF_FFF {
                    None
                } else {
                    Some(Value::I32(val))
                }
            }
        }
        6 => {
            // uint32
            let number_of_values = size / 4;
            if number_of_values > 1 {
                let c: Vec<_> = (0..number_of_values)
                    .filter_map(|_| match read_u32(map, endianness) {
                        0xFFFF_FFFF => None,
                        v => Some(v),
                    })
                    .collect();
                if c.is_empty() {
                    None
                } else {
                    Some(Value::ArrU32(c))
                }
            } else {
                let val = read_u32(map, endianness);
                if val == 0xFFFF_FFFF {
                    None
                } else {
                    Some(Value::U32(val))
                }
            }
        }
        7 => {
            // string
            let mut buf: Vec<_> = Vec::with_capacity(size.into());
            let _ = map.take(size.into()).read_to_end(&mut buf);
            buf.retain(|b| *b != 0x00);
            if let Ok(string) = String::from_utf8(buf) {
                Some(Value::String(string))
            } else {
                None
            }
        }
        8 => {
            // float32
            let number_of_values = size / 4;
            if number_of_values > 1 {
                println!("8 f32: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let uval = read_u32(map, endianness);
                if uval == 0xFF_FFF_FFF {
                    None
                } else {
                    let val = f32::from_bits(uval);
                    Some(Value::F32(val))
                }
            }
        }
        9 => {
            // float64
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("9 f64: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let uval = read_u64(map, endianness);
                if uval == 0xF_FFF_FFF_FFF_FFF_FFF {
                    None
                } else {
                    let val = f64::from_bits(uval);
                    Some(Value::F64(val))
                }
            }
        }
        10 => {
            // uint8z
            if size > 1 {
                println!("10:uint8z {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_u8(map);
                if val == 0x00 {
                    None
                } else {
                    Some(Value::U8(val))
                }
            }
        }
        11 => {
            // uint16z
            let number_of_values = size / 2;
            if number_of_values > 1 {
                println!("11 u16: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_u16(map, endianness);
                if val == 0x0000 {
                    None
                } else {
                    Some(Value::U16(val))
                }
            }
        }
        12 => {
            // uint32z
            let number_of_values = size / 4;
            if number_of_values > 1 {
                println!("12 u32: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_u32(map, endianness);
                if val == 0x0000_0000 {
                    None
                } else {
                    Some(Value::U32(val))
                }
            }
        }
        14 => {
            // sint64
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("14 i64: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_i64(map, endianness);
                if val == 0x7_FFF_FFF_FFF_FFF_FFF {
                    None
                } else {
                    Some(Value::I64(val))
                }
            }
        }
        15 => {
            // uint64
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("15 u64: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_u64(map, endianness);
                if val == 0xF_FFF_FFF_FFF_FFF_FFF {
                    None
                } else {
                    Some(Value::U64(val))
                }
            }
        }
        16 => {
            // uint64z
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("16 u64: {}", size);
                skip_bytes(map, size);
                None
            } else {
                let val = read_u64(map, endianness);
                if val == 0x0_000_000_000_000_000 {
                    None
                } else {
                    Some(Value::U64(val))
                }
            }
        }
        _ => None,
    }
}

#[allow(unused_must_use)]
fn process_value(
    v: &mut DataField,
    fields: fitsdk::MatchFieldTypeFn,
    scales: fitsdk::MatchScaleFn,
    offsets: fitsdk::MatchOffsetFn,
) {
    match fields(v.field_num) {
        FieldType::None => (),
        FieldType::Coordinates => {
            if let Value::I32(ref inner) = v.value {
                let coord = *inner as f32 * COORD_SEMICIRCLES_CALC;
                std::mem::replace(&mut v.value, Value::F32(coord));
            }
        }
        FieldType::Timestamp => {
            if let Value::U32(ref inner) = v.value {
                // self.last_timestamp = *inner;
                let date = *inner + PSEUDO_EPOCH;
                std::mem::replace(&mut v.value, Value::Time(date));
            }
        }
        FieldType::DateTime => {
            if let Value::U32(ref inner) = v.value {
                let date = *inner + PSEUDO_EPOCH;
                std::mem::replace(&mut v.value, Value::Time(date));
            }
        }
        FieldType::LocalDateTime => {
            if let Value::U32(ref inner) = v.value {
                let time = *inner + PSEUDO_EPOCH - 3600;
                std::mem::replace(&mut v.value, Value::Time(time));
            }
        }
        FieldType::String | FieldType::LocaltimeIntoDay => {}
        FieldType::Uint8
        | FieldType::Uint8Z
        | FieldType::Uint16
        | FieldType::Uint16Z
        | FieldType::Uint32
        | FieldType::Uint32Z
        | FieldType::Sint8 => {
            if let Some(s) = scales(v.field_num) {
                v.value.scale(s);
            }
            if let Some(o) = offsets(v.field_num) {
                v.value.offset(o);
            }
        }
        f => {
            if let Value::U8(k) = v.value {
                if let Some(t) = get_field_string_value_fn(f, usize::from(k)) {
                    std::mem::replace(&mut v.value, Value::Enum(t));
                }
            } else if let Value::U16(k) = v.value {
                if let Some(t) = get_field_string_value_fn(f, usize::from(k)) {
                    std::mem::replace(&mut v.value, Value::Enum(t));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::field_definition::FieldDefinition;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_reads_file_header() {
        let a = [14, 16, 116, 6, 51, 92, 1, 0, 46, 70, 73, 84, 213, 14];
        let mut c = Cursor::new(a);
        let fh = FileHeader::new(&mut c);
        assert_eq!(
            fh,
            FileHeader {
                filesize: 14,
                protocol: 16,
                profile_version: 1652,
                num_record_bytes: 89139,
                fileext: true,
                crc: 3797,
            }
        );
    }

    #[test]
    fn it_reads_header_byte_compressed_header() {
        let a = [208];
        let mut c = Cursor::new(a);
        let h = HeaderByte::new(&mut c);
        assert_eq!(
            h,
            HeaderByte {
                compressed_header: true,
                definition: false,
                dev_fields: false,
                local_num: 2,
                time_offset: Some(16)
            }
        );
    }

    #[test]
    fn it_reads_header_byte_normal() {
        let a = [14];
        let mut c = Cursor::new(a);
        let h = HeaderByte::new(&mut c);
        assert_eq!(
            h,
            HeaderByte {
                compressed_header: false,
                definition: false,
                dev_fields: false,
                local_num: 14,
                time_offset: None
            }
        );
    }

    #[test]
    fn it_reads_definition_record() {
        let a = vec![
            0, 0, 0, 0, 7, 3, 4, 140, 4, 4, 134, 7, 4, 134, 1, 2, 132, 2, 2, 132, 5, 2, 132, 0, 1,
            0,
        ];
        let mut c = Cursor::new(a);
        let def = DefinitionRecord::new(&mut c, false);
        let comp = DefinitionRecord {
            endianness: Endianness::Little,
            global_message_number: 0,
            field_definitions: vec![
                FieldDefinition {
                    definition_number: 3,
                    size: 4,
                    base_type: 12,
                },
                FieldDefinition {
                    definition_number: 4,
                    size: 4,
                    base_type: 6,
                },
                FieldDefinition {
                    definition_number: 7,
                    size: 4,
                    base_type: 6,
                },
                FieldDefinition {
                    definition_number: 1,
                    size: 2,
                    base_type: 4,
                },
                FieldDefinition {
                    definition_number: 2,
                    size: 2,
                    base_type: 4,
                },
                FieldDefinition {
                    definition_number: 5,
                    size: 2,
                    base_type: 4,
                },
                FieldDefinition {
                    definition_number: 0,
                    size: 1,
                    base_type: 0,
                },
            ],
            developer_fields: None,
        };
        assert_eq!(def, comp);
    }

    #[test]
    fn it_reads_field_definition() {
        let fda = FieldDefinition::new(&mut Cursor::new([254, 2, 132]));
        assert_eq!(
            fda,
            FieldDefinition {
                definition_number: 254,
                size: 2,
                base_type: 4
            }
        );
        let fdb = FieldDefinition::new(&mut Cursor::new([102, 4, 2]));
        assert_eq!(
            fdb,
            FieldDefinition {
                definition_number: 102,
                size: 4,
                base_type: 2
            }
        );
    }
}
