#![allow(dead_code)]
use copyless::VecHelper;
use memmap::MmapOptions;
use std::collections::{HashMap, VecDeque};
use std::{fs::File, path::PathBuf};

mod sdk {
    #![allow(clippy::unreadable_literal)]
    include!(concat!(env!("OUT_DIR"), "/message_type_enum.rs"));
    include!(concat!(env!("OUT_DIR"), "/field_type_enum.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_field.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_offset.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_scale.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_message_type.rs"));
    include!(concat!(env!("OUT_DIR"), "/match_custom_enum.rs"));
}
mod developer_fields;
mod io;

use developer_fields::{DeveloperFieldDefinition, DeveloperFieldDescription};
use io::*;
pub use sdk::MessageType;
use sdk::{
    enum_type, match_message_field, match_message_offset, match_message_scale, match_message_type,
    FieldType,
};

const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

const _FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10_000_000;
const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00_011_111;

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

pub fn run(path: &PathBuf) -> Vec<Message> {
    let mut global_string_map: HashMap<u8, String> = HashMap::with_capacity(64);
    let file = File::open(path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let mut buf: &[u8] = &mmap;

    let _fh = FileHeader::new(&mut buf);
    let mut q: VecDeque<(u8, DefinitionRecord)> = VecDeque::new();
    let mut records: Vec<Message> = Vec::with_capacity(2500);
    let mut developer_field_descriptions: Vec<DeveloperFieldDescription> = Vec::new();

    let mut fielddefinition_buffer: [FieldDefinition; 128];
    let mut datafield_buffer: [DataField; 128];
    unsafe {
        fielddefinition_buffer = std::mem::uninitialized();
        datafield_buffer = std::mem::uninitialized();
        for elem in &mut datafield_buffer[..] {
            std::ptr::write(elem, DataField::new());
        }
    }

    loop {
        let h = HeaderByte::new(&mut buf);
        if h.definition {
            let d = DefinitionRecord::new(&mut buf, &mut fielddefinition_buffer, h.dev_fields);
            q.push_front((h.local_num, d));
        } else if let Some((_, d)) = q.iter().find(|x| x.0 == h.local_num) {
            let m = match_message_type(d.global_message_number);
            let mut skip = false;
            let mut dev_field_description = false;
            match m {
                MessageType::None => skip = true,
                MessageType::FieldDescription => dev_field_description = true,
                _ => {}
            }
            // we must read all fields, regardless if we already know we won't
            // process the results further otherwise we'll lose our place in the file
            let mut valid_fields = 0;
            for fd in d.field_definitions.iter() {
                let field = read_next_field(
                    fd.base_type,
                    fd.size,
                    d.endianness,
                    &mut buf,
                    skip,
                    &mut global_string_map,
                );
                if skip || !field.is_some() {
                    continue;
                }
                // it's 'safe' to use `unsafe` here because we make sure to keep
                // track of the number of values we've written and only read those
                unsafe {
                    std::ptr::write(
                        &mut datafield_buffer[valid_fields],
                        DataField {
                            field_num: fd.definition_number,
                            value: field,
                        },
                    );
                }
                valid_fields += 1;
            }
            if let Some(dev_fields) = &d.developer_fields {
                dev_fields.iter().for_each(|df| {
                    let def = developer_field_descriptions
                        .iter()
                        .find(|e| {
                            e.developer_data_index == df.developer_data_index
                                && e.field_definition_number == df.field_number
                        })
                        .unwrap();
                    let bt = reverse_map_base_type(def.fit_base_type);
                    let f = read_next_field(
                        bt,
                        df.size,
                        d.endianness,
                        &mut buf,
                        skip,
                        &mut global_string_map,
                    );
                    dbg!(f);
                });
            }

            if dev_field_description {
                let d = DeveloperFieldDescription::new(
                    datafield_buffer[0..valid_fields].to_vec(),
                    &mut global_string_map,
                );
                developer_field_descriptions.push(d);
            }
            // if we have a valid MessageType and some valid fields
            else if !skip && valid_fields > 0 {
                // no need to look these up until now
                let scales: &[Option<f32>] = match_message_scale(m);
                let offsets: &[Option<i16>] = match_message_offset(m);
                let fields: &[FieldType] = match_message_field(m);

                // datafield_buffer is an array longer than we needed, so only take the number of elements we
                // need
                for v in datafield_buffer.iter_mut().take(valid_fields) {
                    if v.field_num >= fields.len() {
                        continue;
                    }
                    match fields[v.field_num] {
                        FieldType::None => (),
                        FieldType::Coordinates => {
                            if let Value::I32(ref inner) = v.value {
                                let coord = *inner as f32 * COORD_SEMICIRCLES_CALC;
                                std::mem::replace(&mut v.value, Value::F32(coord));
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
                        | FieldType::Uint8z
                        | FieldType::Uint16
                        | FieldType::Uint16z
                        | FieldType::Uint32
                        | FieldType::Uint32z
                        | FieldType::Sint8 => {
                            if let Some(s) = scales.get(v.field_num) {
                                if let Some(s) = s {
                                    v.value.scale(*s);
                                }
                            }
                            if let Some(o) = offsets.get(v.field_num) {
                                if let Some(o) = o {
                                    v.value.offset(*o)
                                }
                            }
                        }
                        f => {
                            if let Value::U8(k) = v.value {
                                if let Some(t) = enum_type(f, u16::from(k)) {
                                    std::mem::replace(&mut v.value, Value::Enum(t));
                                }
                            } else if let Value::U16(k) = v.value {
                                if let Some(t) = enum_type(f, k) {
                                    std::mem::replace(&mut v.value, Value::Enum(t));
                                }
                            }
                        }
                    }
                }
                let final_values = datafield_buffer[0..valid_fields].to_vec();

                let msg = Message {
                    values: final_values,
                    kind: m,
                };
                records.alloc().init(msg);
            }
        }
        if buf.len() <= 14 {
            break;
        }
    }
    records
}

#[derive(Clone)]
pub struct Message {
    pub kind: MessageType,
    pub values: Vec<DataField>,
}

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
    fn new(map: &mut &[u8]) -> Self {
        Self {
            filesize: u8(map),
            protocol: u8(map),
            profile_version: u16(map, Endianness::Little),
            num_record_bytes: u32(map, Endianness::Little),
            fileext: {
                let buf = arr4(map);
                &buf == b".FIT"
            },
            crc: u16(map, Endianness::Little),
        }
    }
}

#[derive(Debug)]
struct HeaderByte {
    definition: bool,
    dev_fields: bool,
    local_num: u8,
}
impl HeaderByte {
    fn new(map: &mut &[u8]) -> Self {
        let b = u8(map);
        Self {
            definition: (b & DEFINITION_HEADER_MASK) == DEFINITION_HEADER_MASK,
            dev_fields: (b & DEVELOPER_FIELDS_MASK) == DEVELOPER_FIELDS_MASK,
            local_num: b & LOCAL_MESSAGE_NUMBER_MASK,
        }
    }
}

struct DefinitionRecord {
    endianness: Endianness,
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
    developer_fields: Option<Vec<DeveloperFieldDefinition>>,
}
impl DefinitionRecord {
    fn new(map: &mut &[u8], buffer: &mut [FieldDefinition; 128], dev_fields: bool) -> Self {
        skip_bytes(map, 1);
        let endian = match u8(map) {
            1 => Endianness::Big,
            0 => Endianness::Little,
            _ => panic!("unexpected endian byte"),
        };
        let global_message_number = u16(map, endian);
        let number_of_fields = u8(map);

        for i in 0..number_of_fields {
            buffer[i as usize] = FieldDefinition::new(map);
        }
        let dev_fields: Option<Vec<DeveloperFieldDefinition>> = if dev_fields {
            let number_of_fields = u8(map);
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
            field_definitions: buffer[0..number_of_fields as usize].to_vec(),
            developer_fields: dev_fields,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataField {
    field_num: usize,
    value: Value,
}
impl DataField {
    fn new() -> Self {
        Self {
            field_num: 0,
            value: Value::None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    Enum(&'static str),
    String(u8),
    F32(f32),
    F64(f64),
    I64(i64),
    U64(u64),
    Time(u32),
    ArrU8(Box<[u8]>),
    ArrU16(Box<[u16]>),
    ArrU32(Box<[u32]>),
    None,
}
impl Value {
    fn is_some(&self) -> bool {
        match self {
            Value::None => false,
            _ => true,
        }
    }
    fn scale(&mut self, val: f32) {
        match self {
            Value::U8(mut inner) => {
                let new_inner = f32::from(inner) / val;
                std::mem::replace(&mut inner, new_inner as u8);
            }
            Value::I8(mut inner) => {
                let new_inner = f32::from(inner) / val;
                std::mem::replace(&mut inner, new_inner as i8);
            }
            Value::U16(mut inner) => {
                let new_inner = f32::from(inner) / val;
                std::mem::replace(&mut inner, new_inner as u16);
            }
            Value::I16(mut inner) => {
                let new_inner = f32::from(inner) / val;
                std::mem::replace(&mut inner, new_inner as i16);
            }
            Value::U32(mut inner) => {
                let new_inner = inner as f32 / val;
                std::mem::replace(&mut inner, new_inner as u32);
            }
            Value::I32(mut inner) => {
                let new_inner = inner as f32 / val;
                std::mem::replace(&mut inner, new_inner as i32);
            }
            _ => (),
        }
    }
    fn offset(&mut self, val: i16) {
        match self {
            Value::U8(mut inner) => {
                let new_inner = i16::from(inner) - val;
                std::mem::replace(&mut inner, new_inner as u8);
            }
            Value::I8(mut inner) => {
                let new_inner = i16::from(inner) - val;
                std::mem::replace(&mut inner, new_inner as i8);
            }
            Value::U16(mut inner) => {
                let new_inner = inner as i16 - val;
                std::mem::replace(&mut inner, new_inner as u16);
            }
            Value::I16(mut inner) => {
                let new_inner = inner - val;
                std::mem::replace(&mut inner, new_inner);
            }
            Value::U32(mut inner) => {
                let new_inner = inner as i16 - val;
                std::mem::replace(&mut inner, new_inner as u32);
            }
            Value::I32(mut inner) => {
                let new_inner = inner as i16 - val;
                std::mem::replace(&mut inner, i32::from(new_inner));
            }
            _ => (),
        }
    }
}
impl From<Value> for u8 {
    fn from(item: Value) -> Self {
        match item {
            Value::U8(v) => v,
            _ => panic!("can't call this on a non-u8 variant"),
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
        Self {
            definition_number: buf[0].into(),
            size: buf[1],
            base_type: buf[2] & FIELD_DEFINITION_BASE_NUMBER,
        }
    }
}

fn reverse_map_base_type(i: u8) -> u8 {
    match i {
        _ => i,
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn read_next_field(
    base_type: u8,
    size: u8,
    endianness: Endianness,
    map: &mut &[u8],
    skip: bool,
    global_string_map: &mut HashMap<u8, String>,
) -> Value {
    if skip {
        skip_bytes(map, size);
        return Value::None;
    }
    match base_type {
        0 | 13 => {
            // enum / byte
            if size > 1 {
                println!("0/13:enum/byte: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = u8(map);
                if val == 0xFF {
                    Value::None
                } else {
                    Value::U8(val)
                }
            }
        }
        1 => {
            // sint8
            if size > 1 {
                println!("1 i8: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = i8(map);
                if val == 0x7F {
                    Value::None
                } else {
                    Value::I8(val)
                }
            }
        }
        2 => {
            // uint8
            if size > 1 {
                let (buf, rest) = map.split_at(size.into());
                *map = rest;
                let c: Vec<u8> = buf.iter().cloned().filter(|x| *x != 0xFF).collect();
                if c.is_empty() {
                    Value::None
                } else {
                    let b = c.into_boxed_slice();
                    Value::ArrU8(b)
                }
            } else {
                let val = u8(map);
                if val == 0xFF {
                    Value::None
                } else {
                    Value::U8(val)
                }
            }
        }
        3 => {
            // sint16
            let number_of_values = size / 2;
            if number_of_values > 1 {
                println!("3 i16: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = i16(map, endianness);
                if val == 0x7FFF {
                    Value::None
                } else {
                    Value::I16(val)
                }
            }
        }
        4 => {
            // uint16
            let number_of_values = size / 2;
            if number_of_values > 1 {
                let c: Vec<_> = (0..number_of_values)
                    .map(|_| u16(map, endianness))
                    .filter(|x| *x != 0xFFFF)
                    .collect();
                if c.is_empty() {
                    Value::None
                } else {
                    let b = c.into_boxed_slice();
                    Value::ArrU16(b)
                }
            } else {
                let val = u16(map, endianness);
                if val == 0xFFFF {
                    Value::None
                } else {
                    Value::U16(val)
                }
            }
        }
        5 => {
            // sint32
            let number_of_values = size / 4;
            if number_of_values > 1 {
                println!("5 i32: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = i32(map, endianness);
                if val == 0x7F_FFF_FFF {
                    Value::None
                } else {
                    Value::I32(val)
                }
            }
        }
        6 => {
            // uint32
            let number_of_values = size / 4;
            if number_of_values > 1 {
                let c: Vec<_> = (0..number_of_values)
                    .map(|_| u32(map, endianness))
                    .filter(|x| *x != 0xFFFF_FFFF)
                    .collect();
                if c.is_empty() {
                    Value::None
                } else {
                    let b = c.into_boxed_slice();
                    Value::ArrU32(b)
                }
            } else {
                let val = u32(map, endianness);
                if val == 0xFFFF_FFFF {
                    Value::None
                } else {
                    Value::U32(val)
                }
            }
        }
        7 => {
            // string
            let (buf, rest) = map.split_at(size as usize);
            *map = rest;
            let buf: Vec<u8> = buf.iter().filter(|b| *b != &0x00).cloned().collect();
            if let Ok(s) = String::from_utf8(buf) {
                let k = match global_string_map.keys().max() {
                    Some(k) => k + 1,
                    None => 0,
                };
                global_string_map.insert(k, s);
                Value::String(k)
            } else {
                Value::None
            }
        }
        8 => {
            // float32
            let number_of_values = size / 4;
            if number_of_values > 1 {
                println!("8 f32: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let uval = u32(map, endianness);
                if uval == 0xFF_FFF_FFF {
                    Value::None
                } else {
                    let val = f32::from_bits(uval);
                    Value::F32(val)
                }
            }
        }
        9 => {
            // float64
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("9 f64: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let uval = u64(map, endianness);
                if uval == 0xF_FFF_FFF_FFF_FFF_FFF {
                    Value::None
                } else {
                    let val = f64::from_bits(uval);
                    Value::F64(val)
                }
            }
        }
        10 => {
            // uint8z
            if size > 1 {
                println!("10:uint8z {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = u8(map);
                if val == 0x00 {
                    Value::None
                } else {
                    Value::U8(val)
                }
            }
        }
        11 => {
            // uint16z
            let number_of_values = size / 2;
            if number_of_values > 1 {
                println!("11 u16: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = u16(map, endianness);
                if val == 0x0000 {
                    Value::None
                } else {
                    Value::U16(val)
                }
            }
        }
        12 => {
            // uint32z
            let number_of_values = size / 4;
            if number_of_values > 1 {
                println!("12 u32: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = u32(map, endianness);
                if val == 0x0000_0000 {
                    Value::None
                } else {
                    Value::U32(val)
                }
            }
        }
        14 => {
            // sint64
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("14 i64: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = i64(map, endianness);
                if val == 0x7_FFF_FFF_FFF_FFF_FFF {
                    Value::None
                } else {
                    Value::I64(val)
                }
            }
        }
        15 => {
            // uint64
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("15 u64: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = u64(map, endianness);
                if val == 0xF_FFF_FFF_FFF_FFF_FFF {
                    Value::None
                } else {
                    Value::U64(val)
                }
            }
        }
        16 => {
            // uint64z
            let number_of_values = size / 8;
            if number_of_values > 1 {
                println!("16 u64: {}", size);
                skip_bytes(map, size);
                Value::None
            } else {
                let val = u64(map, endianness);
                if val == 0x0_000_000_000_000_000 {
                    Value::None
                } else {
                    Value::U64(val)
                }
            }
        }
        _ => Value::None,
    }
}
