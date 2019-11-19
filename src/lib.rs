mod developer_fields;
mod io;
mod value;
mod consts;

use fitsdk::{
    match_custom_field_value, match_message_field, match_message_offset, match_message_scale,
    match_message_timestamp_field, match_messagetype, FieldType, MessageType,
};
use memmap::{MmapOptions,Mmap};
use std::collections::VecDeque;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::{fs::File, path::PathBuf};
use copyless::VecHelper;
use developer_fields::{DeveloperFieldDefinition, DeveloperFieldDescription};
use io::*;
use value::*;
use consts::*;

type ScaleClosure = dyn std::ops::Fn(usize) -> std::option::Option<f32>;
type OffsetClosure = dyn std::ops::Fn(usize) -> std::option::Option<i16>;
type FieldClosure = dyn std::ops::Fn(usize) -> fitsdk::FieldType;

//////////
//// Fit
//////////

pub struct Fit {
    _file_header: FileHeader,
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
            _file_header: fh,
            buf: buf,
            queue: VecDeque::new(),
            developer_fields: Vec::new(),
            last_timestamp: 0,
        }
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
                    Some((_,def)) => def
                };
                let message_type = match_messagetype(definition.global_message_number);
                let mut dev_fields: Option<Vec<DevDataField>> = None;
                let mut values = Vec::with_capacity(definition.field_definitions.len());

                // read all the values for this reacord type's defined fields
                for fd in definition.field_definitions.iter() {
                    if message_type == MessageType::None {
                        skip_bytes(&mut self.buf, fd.size);
                    } else if let Some(data) = read_next_field(
                            fd.base_type,
                            fd.size,
                            definition.endianness,
                            &mut self.buf,
                        ){
                        values.alloc().init(DataField::new(fd.definition_number, data));
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
                                if let Some(v) = read_next_field(e.fit_base_type, df.size, definition.endianness, &mut self.buf){
                                    temp_dev_fields.push(DevDataField::new(
                                        df.developer_data_index,
                                        df.field_number,
                                        v
                                    ));
                                }
                            }
                        }
                    }
                    dev_fields = Some(temp_dev_fields);
                }

                // check each value in case the raw value needs further processing
                let scales = match_message_scale(message_type);
                let offsets = match_message_offset(message_type);
                let fields = match_message_field(message_type);
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

                    if let Some(timestamp_field_number) =
                        match_message_timestamp_field(message_type)
                    {
                        values.alloc().init(
                                DataField::new(
                                    timestamp_field_number,
                                    Value::Time(timestamp + PSEUDO_EPOCH),
                                )
                            );
                    }
                }
                
                // if any values were invalid we have a vec that's now too long
                values.shrink_to_fit();
                return Some(Message {
                    values: values,
                    kind: message_type,
                    dev_values: dev_fields,
                });
            }
        }
    }
}


#[allow(clippy::cognitive_complexity)]
pub fn read_next_field<R>(
    base_type: u8,
    size: u8,
    endianness: Endianness,
    map: &mut R,
) -> Option<Value>
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

fn process_value(v: &mut DataField, fields: &FieldClosure, scales: &ScaleClosure, offsets: &OffsetClosure){
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
                if let Some(t) = match_custom_field_value(f, usize::from(k)) {
                    std::mem::replace(&mut v.value, Value::Enum(t));
                }
            } else if let Value::U16(k) = v.value {
                if let Some(t) = match_custom_field_value(f, usize::from(k)) {
                    std::mem::replace(&mut v.value, Value::Enum(t));
                }
            }
        }
    }
}

//////////
//// Message
//////////

#[derive(Clone, Debug)]
pub struct Message {
    pub kind: MessageType,
    pub values: Vec<DataField>,
    pub dev_values: Option<Vec<DevDataField>>,
}

//////////
//// FileHeader
//////////

#[derive(Debug)]
struct FileHeader {
    filesize: u8,
    protocol: u8,
    profile_version: u16,
    num_record_bytes: u32,
    fileext: bool,
    crc: u16,
}
impl FileHeader {
    fn new<R>(map: &mut R) -> Self
    where
        R: Read,
    {
        Self {
            filesize: read_u8(map),
            protocol: read_u8(map),
            profile_version: read_u16(map, Endianness::Little),
            num_record_bytes: read_u32(map, Endianness::Little),
            fileext: {
                let buf = arr4(map);
                &buf == b".FIT"
            },
            crc: read_u16(map, Endianness::Little),
        }
    }
}

//////////
//// HeaderByte
//////////

#[derive(Debug)]
struct HeaderByte {
    compressed_header: bool,
    definition: bool,
    dev_fields: bool,
    local_num: u8,
    time_offset: Option<u8>,
}
impl HeaderByte {
    fn new<R>(map: &mut R) -> Self
    where
        R: Read,
    {
        let b = read_u8(map);
        if (b & COMPRESSED_HEADER_MASK) == COMPRESSED_HEADER_MASK {
            Self {
                compressed_header: true,
                definition: false,
                dev_fields: false,
                local_num: (b & COMPRESSED_HEADER_LOCAL_MESSAGE_NUMBER_MASK) >> 5,
                time_offset: Some(b & COMPRESSED_HEADER_TIME_OFFSET_MASK),
            }
        } else {
            Self {
                compressed_header: false,
                definition: (b & DEFINITION_HEADER_MASK) == DEFINITION_HEADER_MASK,
                dev_fields: (b & DEVELOPER_FIELDS_MASK) == DEVELOPER_FIELDS_MASK,
                local_num: b & LOCAL_MESSAGE_NUMBER_MASK,
                time_offset: None,
            }
        }
    }
    fn compressed_timestamp(self) -> Option<u8> {
        if self.compressed_header {
            return self.time_offset;
        } else {
            return None;
        }
    }
}

//////////
//// DefinitionRecord
//////////

struct DefinitionRecord {
    endianness: Endianness,
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
    developer_fields: Option<Vec<DeveloperFieldDefinition>>,
}
impl DefinitionRecord {
    fn new<R>(
        map: &mut R,
        dev_fields: bool,
    ) -> Self
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

//////////
//// DataField
//////////

#[derive(Clone, Debug)]
pub struct DataField {
    pub field_num: usize,
    pub value: Value,
}
impl DataField {
    fn new(fnum: usize, v: Value) -> Self {
        Self {
            field_num: fnum,
            value: v,
        }
    }
}

//////////
//// DevDataField
//////////

#[derive(Clone, Debug)]
pub struct DevDataField {
    pub data_index: u8,
    pub field_num: u8,
    pub value: Value,
}
impl DevDataField {
    fn new(ddi: u8, fnum: u8, v: Value) -> Self {
        Self {
            data_index: ddi,
            field_num: fnum,
            value: v,
        }
    }
}


//////////
//// FieldDefinition
//////////

#[derive(Debug, Copy, Clone)]
pub struct FieldDefinition {
    pub definition_number: usize,
    pub size: u8,
    pub base_type: u8,
}
impl FieldDefinition {
    pub fn new<R>(map: &mut R) -> Self
    where
        R: Read,
    {
        let mut buf: [u8; 3] = [0; 3];
        let _ = map.read(&mut buf);
        Self {
            definition_number: buf[0].into(),
            size: buf[1],
            base_type: buf[2] & FIELD_DEFINITION_BASE_NUMBER,
        }
    }
}

