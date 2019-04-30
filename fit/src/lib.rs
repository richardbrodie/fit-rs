#![allow(unused)]
use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::sync::Mutex;
use std::{fs::File, path::PathBuf};

mod sdk {
    include!(concat!(env!("OUT_DIR"), "/message_type_enum.rs"));
    include!(concat!(env!("OUT_DIR"), "/field_type_enum.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_field.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_offset.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_scale.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_type.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_custom_enum.rs"));
}

use memmap::{Mmap, MmapOptions};
use sdk::{
    enum_type, match_message_field, match_message_offset, match_message_scale, match_message_type,
    FieldType, MessageType,
};

lazy_static! {
    static ref GSTRING: Mutex<HashMap<u16, String>> = Mutex::new(HashMap::new());
}

const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

const FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10_000_000;
const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00_011_111;

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

pub fn run(path: &PathBuf) {
    let file = File::open(path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let mut buf: &[u8] = &mmap;

    let fh = FileHeader::new(&mut buf);
    let mut q: VecDeque<(u8, DefinitionRecord)> = VecDeque::new();
    let mut records: Vec<Message> = Vec::with_capacity(7500);
    loop {
        let h = HeaderByte::new(&mut buf);
        if h.definition {
            let d = DefinitionRecord::new(&mut buf);
            q.push_front((h.local_num, d));
        } else if let Some((i, d)) = q.iter().find(|x| x.0 == h.local_num) {
            let m = match_message_type(d.global_message_number);
            let mut skip = false;
            let mut fields: &[FieldType] = Default::default();
            let mut scales: &[Option<f32>] = Default::default();
            let mut offsets: &[Option<i16>] = Default::default();
            let mut values: Vec<DataField> = Vec::new();
            if m == MessageType::None {
                skip = true;
            } else {
                values = Vec::with_capacity(d.field_definitions.len());
                fields = match_message_field(m);
                scales = match_message_scale(m);
                offsets = match_message_offset(m);
            };

            values.extend(
                d.field_definitions
                    .iter()
                    .map(|fd| (fd.definition_number, DataField::read(fd, &mut buf, skip)))
                    .filter(|(def_num, data)| !skip && data.is_some() && fields.len() > *def_num)
                    .map(|(def_num, mut data)| {
                        match fields[def_num] {
                            FieldType::None => {}
                            FieldType::Coordinates => {
                                if let Value::I32(ref inner) = &mut data {
                                    let coord = *inner as f32 * COORD_SEMICIRCLES_CALC;
                                    std::mem::replace(&mut data, Value::F32(coord));
                                }
                            }
                            FieldType::DateTime => {
                                if let Value::U32(ref inner) = data {
                                    let date = *inner + PSEUDO_EPOCH;
                                    std::mem::replace(&mut data, Value::Time(date));
                                }
                            }
                            FieldType::LocalDateTime => {
                                if let Value::U32(ref inner) = data {
                                    let time = *inner + PSEUDO_EPOCH - 3600;
                                    std::mem::replace(&mut data, Value::Time(time));
                                }
                            }
                            FieldType::String | FieldType::LocaltimeIntoDay => (),
                            FieldType::Uint8
                            | FieldType::Uint8z
                            | FieldType::Uint16
                            | FieldType::Uint16z
                            | FieldType::Uint32
                            | FieldType::Uint32z
                            | FieldType::Sint8 => {
                                if let Some(s) = scales[def_num] {
                                    data.scale(s)
                                }
                                if let Some(o) = offsets[def_num] {
                                    data.offset(o)
                                }
                            }
                            f => {
                                if let Value::U8(k) = data {
                                    if let Some(t) = enum_type(f, k.into()) {
                                        std::mem::replace(&mut data, Value::string(t.into()));
                                    }
                                } else if let Value::U16(k) = data {
                                    if let Some(t) = enum_type(f, k) {
                                        std::mem::replace(&mut data, Value::string(t.into()));
                                    }
                                }
                            }
                        }
                        DataField {
                            field_num: def_num,
                            value: data,
                        }
                    }),
            );
            // records.push(Message {
            // kind: m,
            // values: values,
            // });
        }
        if buf.len() <= 14 {
            break;
        }
    }
}

#[derive(Debug)]
pub struct FileHeader {
    filesize: u8,
    protocol: u8,
    profile_version: u16,
    num_record_bytes: u32,
    fileext: bool,
    crc: u16,
}
impl FileHeader {
    pub fn new(map: &mut &[u8]) -> Self {
        Self {
            filesize: u8(map),
            protocol: u8(map),
            profile_version: u16(map, 0),
            num_record_bytes: u32(map, 0),
            fileext: {
                let buf = arr4(map);
                &buf == b".FIT"
            },
            crc: u16(map, 0),
        }
    }
}

#[derive(Debug)]
struct HeaderByte {
    definition: bool,
    local_num: u8,
}
impl HeaderByte {
    pub fn new(map: &mut &[u8]) -> Self {
        let b = u8(map);
        if (b & DEVELOPER_FIELDS_MASK) == DEVELOPER_FIELDS_MASK {
            panic!("unsupported developer fields");
        }
        Self {
            definition: (b & DEFINITION_HEADER_MASK) == DEFINITION_HEADER_MASK,
            local_num: b & LOCAL_MESSAGE_NUMBER_MASK,
        }
    }
}

#[derive(Debug)]
struct DefinitionRecord {
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
}
impl DefinitionRecord {
    pub fn new(map: &mut &[u8]) -> Self {
        skip_bytes(map, 1);
        let endian = match u8(map) {
            1 => 1, // big
            0 => 0, // little
            _ => panic!("unexpected endian byte"),
        };
        let global_message_number = u16(map, 0);
        let number_of_fields = u8(map);
        DefinitionRecord {
            global_message_number,
            field_definitions: (0..number_of_fields)
                .map(|_| FieldDefinition::new(map))
                .collect(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FieldDefinition {
    pub definition_number: usize,
    pub size: u8,
    pub base_type: u8,
}
impl FieldDefinition {
    pub fn new(map: &mut &[u8]) -> Self {
        let (buf, rest) = map.split_at(3);
        *map = rest;
        let base_num = buf[2] & FIELD_DEFINITION_BASE_NUMBER;
        Self {
            definition_number: buf[0].into(),
            size: buf[1],
            base_type: base_num,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DataField {
    field_num: usize,
    value: Value,
}
impl DataField {
    fn read(fd: &FieldDefinition, map: &mut &[u8], skip: bool) -> Value {
        match fd.base_type {
            0 | 2 | 13 => {
                // enum / uint8 / byte
                if fd.size > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u8(map);
                if val == 0xFF {
                    return Value::None;
                }
                Value::U8(val)
            }
            1 => {
                // sint8
                if fd.size > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = i8(map);
                if val == 0x7F {
                    return Value::None;
                }
                Value::I8(val)
            }
            3 => {
                // sint16
                let number_of_values = fd.size / 2;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = i16(map, 0);
                if val == 0x7FFF {
                    return Value::None;
                }
                Value::I16(val)
            }
            4 => {
                // uint16
                let number_of_values = fd.size / 2;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u16(map, 0);
                if val == 0xFFFF {
                    return Value::None;
                }
                Value::U16(val)
            }
            5 => {
                // sint32
                let number_of_values = fd.size / 4;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = i32(map, 0);
                if val == 0x7F_FFF_FFF {
                    return Value::None;
                }
                Value::I32(val)
            }
            6 => {
                // uint32
                let number_of_values = fd.size / 4;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u32(map, 0);
                if val == 0xFFFF_FFFF {
                    return Value::None;
                }
                Value::U32(val)
            }
            7 => {
                // string
                let (buf, rest) = map.split_at(fd.size as usize);
                *map = rest;
                let buf: Vec<u8> = buf.iter().filter(|b| *b != &0x00).cloned().collect();
                if let Ok(s) = String::from_utf8(buf) {
                    Value::string(s)
                } else {
                    Value::None
                }
            }
            8 => {
                // float32
                let number_of_values = fd.size / 4;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let uval = u32(map, 0);
                if uval == 0xFF_FFF_FFF {
                    return Value::None;
                }
                let val = f32::from_bits(uval);
                Value::F32(val)
            }
            9 => {
                // float64
                let number_of_values = fd.size / 8;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let uval = u64(map, 0);
                if uval == 0xF_FFF_FFF_FFF_FFF_FFF {
                    return Value::None;
                }
                let val = f64::from_bits(uval);
                Value::F64(val)
            }
            10 => {
                // uint8z
                if fd.size > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u8(map);
                if val == 0x00 {
                    return Value::None;
                }
                Value::U8(val)
            }
            11 => {
                // uint16z
                let number_of_values = fd.size / 2;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u16(map, 0);
                if val == 0x0000 {
                    return Value::None;
                }
                Value::U16(val)
            }
            12 => {
                // uint32z
                let number_of_values = fd.size / 4;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u32(map, 0);
                if val == 0x0000_0000 {
                    return Value::None;
                }
                Value::U32(val)
            }
            14 => {
                // sint64
                let number_of_values = fd.size / 8;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = i64(map, 0);
                if val == 0x7_FFF_FFF_FFF_FFF_FFF {
                    return Value::None;
                }
                Value::I64(val)
            }
            15 => {
                // uint64
                let number_of_values = fd.size / 8;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u64(map, 0);
                if val == 0xF_FFF_FFF_FFF_FFF_FFF {
                    return Value::None;
                }
                Value::U64(val)
            }
            16 => {
                // uint64z
                let number_of_values = fd.size / 8;
                if number_of_values > 1 {
                    skip_bytes(map, fd.size);
                    return Value::None;
                }
                let val = u64(map, 0);
                if val == 0x0_000_000_000_000_000 {
                    return Value::None;
                }
                Value::U64(val)
            }
            _ => Value::None,
        }
    }
}

#[derive(Clone)]
struct Message {
    kind: MessageType,
    values: Vec<DataField>,
}

#[derive(Debug, Clone, Copy)]
enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    String(u16),
    F32(f32),
    F64(f64),
    I64(i64),
    U64(u64),
    Time(u32),
    None,
}
impl Value {
    fn is_none(&self) -> bool {
        match self {
            Value::None => true,
            _ => false,
        }
    }
    fn is_some(&self) -> bool {
        match self {
            Value::None => false,
            _ => true,
        }
    }
    fn string(s: String) -> Self {
        let x = rand::random::<u16>();
        GSTRING.lock().unwrap().insert(x, s);
        Value::String(x)
    }
    fn scale(&mut self, val: f32) {
        match self {
            Value::U8(ref mut inner) => {
                let new_inner = f32::from(*inner) / val;
                std::mem::replace(inner, new_inner as u8);
            }
            Value::I8(ref mut inner) => {
                let new_inner = f32::from(*inner) / val;
                std::mem::replace(inner, new_inner as i8);
            }
            Value::U16(ref mut inner) => {
                let new_inner = f32::from(*inner) / val;
                std::mem::replace(inner, new_inner as u16);
            }
            Value::I16(ref mut inner) => {
                let new_inner = f32::from(*inner) / val;
                std::mem::replace(inner, new_inner as i16);
            }
            Value::U32(ref mut inner) => {
                let new_inner = *inner as f32 / val;
                std::mem::replace(inner, new_inner as u32);
            }
            Value::I32(ref mut inner) => {
                let new_inner = *inner as f32 / val;
                std::mem::replace(inner, new_inner as i32);
            }
            _ => (),
        }
    }
    fn offset(&mut self, val: i16) {
        match self {
            Value::U8(ref mut inner) => {
                let new_inner = i16::from(*inner) - val;
                std::mem::replace(inner, new_inner as u8);
            }
            Value::I8(ref mut inner) => {
                let new_inner = i16::from(*inner) - val;
                std::mem::replace(inner, new_inner as i8);
            }
            Value::U16(ref mut inner) => {
                let new_inner = *inner as i16 - val;
                std::mem::replace(inner, new_inner as u16);
            }
            Value::I16(ref mut inner) => {
                let new_inner = *inner - val;
                std::mem::replace(inner, new_inner);
            }
            Value::U32(ref mut inner) => {
                let new_inner = *inner as i16 - val;
                std::mem::replace(inner, new_inner as u32);
            }
            Value::I32(ref mut inner) => {
                let new_inner = *inner as i16 - val;
                std::mem::replace(inner, i32::from(new_inner));
            }
            _ => (),
        }
    }
}

fn u8(map: &mut &[u8]) -> u8 {
    let (val, rest) = map.split_first().unwrap();
    *map = rest;
    *val
}
fn i8(map: &mut &[u8]) -> i8 {
    u8(map) as i8
}
fn u16(map: &mut &[u8], endianness: u8) -> u16 {
    let arr = arr2(map);
    if endianness == 0 {
        u16::from_le_bytes(arr)
    } else {
        u16::from_be_bytes(arr)
    }
}
fn i16(map: &mut &[u8], endianness: u8) -> i16 {
    let arr = arr2(map);
    if endianness == 0 {
        i16::from_le_bytes(arr)
    } else {
        i16::from_be_bytes(arr)
    }
}
fn u32(map: &mut &[u8], endianness: u8) -> u32 {
    let arr = arr4(map);
    if endianness == 0 {
        u32::from_le_bytes(arr)
    } else {
        u32::from_be_bytes(arr)
    }
}
fn i32(map: &mut &[u8], endianness: u8) -> i32 {
    let arr = arr4(map);
    if endianness == 0 {
        i32::from_le_bytes(arr)
    } else {
        i32::from_be_bytes(arr)
    }
}
fn u64(mut map: &mut &[u8], endianness: u8) -> u64 {
    let arr = arr8(map);
    if endianness == 0 {
        u64::from_le_bytes(arr)
    } else {
        u64::from_be_bytes(arr)
    }
}
fn i64(mut map: &mut &[u8], endianness: u8) -> i64 {
    let arr = arr8(map);
    if endianness == 0 {
        i64::from_le_bytes(arr)
    } else {
        i64::from_be_bytes(arr)
    }
}

fn arr2(map: &mut &[u8]) -> [u8; 2] {
    let (buf, rest) = map.split_at(2);
    *map = rest;
    buf.try_into().unwrap()
}

fn arr4(map: &mut &[u8]) -> [u8; 4] {
    let (buf, rest) = map.split_at(4);
    *map = rest;
    buf.try_into().unwrap()
}

fn arr8(map: &mut &[u8]) -> [u8; 8] {
    let (buf, rest) = map.split_at(8);
    *map = rest;
    buf.try_into().unwrap()
}

fn skip_bytes(map: &mut &[u8], s: u8) {
    let (buf, rest) = map.split_at(s as usize);
    *map = rest;
}
