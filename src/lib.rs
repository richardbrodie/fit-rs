#![allow(unused)]
// use copyless::VecHelper;
use lazy_static::lazy_static;
use memmap::MmapOptions;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
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
mod io;

use io::*;
use sdk::{
    enum_type, match_message_field, match_message_offset, match_message_scale, match_message_type,
    FieldType, MessageType,
};

lazy_static! {
    static ref GSTRING: Mutex<HashMap<u8, String>> = Mutex::new(HashMap::with_capacity(64));
}

const VARRAY_LENGTH: usize = 64;

const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

const FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10_000_000;
const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00_011_111;

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

pub fn run(path: &PathBuf) {
    GSTRING.lock().unwrap().drain();
    let file = File::open(path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let mut buf: &[u8] = &mmap;

    let _fh = FileHeader::new(&mut buf);
    let mut q: VecDeque<(u8, DefinitionRecord)> = VecDeque::new();
    let mut records: Vec<Message> = Vec::new();
    loop {
        let h = HeaderByte::new(&mut buf);
        if h.definition {
            let d = DefinitionRecord::new(&mut buf);
            q.push_front((h.local_num, d));
        } else if let Some((_, d)) = q.iter().find(|x| x.0 == h.local_num) {
            let m = match_message_type(d.global_message_number);
            let skip = match m {
                MessageType::None => true,
                _ => false,
            };
            let mut values: [DataField; 64] = [Default::default(); 64];

            // read all fields, regardless if we already know we won't process the results further
            // otherwise we'll lose our place in the file
            let mut valid_fields = 0;
            for i in 0..d.field_definitions.len() {
                let fd = &d.field_definitions[i];
                let d = read_raw_field(fd, d.endianness, &mut buf, skip);
                if !skip && d.is_some() {
                    values[valid_fields] = DataField {
                        field_num: fd.definition_number,
                        value: d,
                    };
                    valid_fields += 1;
                }
            }

            // if we have a valid MessageType and some valid fields
            if !skip && valid_fields > 0 {
                // no need to look these up until now
                let fields: &[FieldType] = match_message_field(m);
                let scales: &[Option<f32>] = match_message_scale(m);
                let offsets: &[Option<i16>] = match_message_offset(m);

                // values is an array longer than we needed, so only take the number of elements we
                // need
                for vi in 0..valid_fields {
                    let mut v = values[vi];
                    if v.field_num < fields.len() {
                        match fields[v.field_num] {
                            FieldType::None => v.field_num = std::usize::MAX,
                            FieldType::Coordinates => {
                                if let Value::I32(ref inner) = v.value {
                                    let coord = *inner as f32 * COORD_SEMICIRCLES_CALC;
                                    v.value = Value::F32(coord);
                                }
                            }
                            FieldType::DateTime => {
                                if let Value::U32(ref inner) = v.value {
                                    let date = *inner + PSEUDO_EPOCH;
                                    v.value = Value::Time(date);
                                }
                            }
                            FieldType::LocalDateTime => {
                                if let Value::U32(ref inner) = v.value {
                                    let time = *inner + PSEUDO_EPOCH - 3600;
                                    v.value = Value::Time(time);
                                }
                            }
                            FieldType::String | FieldType::LocaltimeIntoDay => {
                                v.field_num = std::usize::MAX
                            }
                            FieldType::Uint8
                            | FieldType::Uint8z
                            | FieldType::Uint16
                            | FieldType::Uint16z
                            | FieldType::Uint32
                            | FieldType::Uint32z
                            | FieldType::Sint8 => {
                                if let Some(s) = scales.get(v.field_num) {
                                    if let Some(s) = s {
                                        v.value.scale(*s)
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
                                        v.value = Value::Enum(t);
                                    }
                                } else if let Value::U16(k) = v.value {
                                    if let Some(t) = enum_type(f, u16::from(k)) {
                                        v.value = Value::Enum(t);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
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

#[derive(Copy, Clone, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

struct DefinitionRecord {
    endianness: Endianness,
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
}
impl DefinitionRecord {
    pub fn new(map: &mut &[u8]) -> Self {
        skip_bytes(map, 1);
        let endian = match u8(map) {
            1 => Endianness::Big,
            0 => Endianness::Little,
            _ => panic!("unexpected endian byte"),
        };
        let global_message_number = u16(map, Endianness::Little);
        let number_of_fields = u8(map);
        DefinitionRecord {
            endianness: endian,
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

#[derive(Clone, Copy)]
struct DataField {
    field_num: usize,
    value: Value,
}
impl Default for DataField {
    fn default() -> DataField {
        DataField {
            field_num: 0,
            value: Default::default(),
        }
    }
}
fn read_raw_field(
    fd: &FieldDefinition,
    endianness: Endianness,
    map: &mut &[u8],
    skip: bool,
) -> Value {
    match fd.base_type {
        0 | 13 => {
            // enum / byte
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if fd.size > 1 {
                println!("0/13:enum/byte: {}", fd.size);
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
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if fd.size > 1 {
                println!("1 i8: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = i8(map);
            if val == 0x7F {
                return Value::None;
            }
            Value::I8(val)
        }
        2 => {
            // uint8
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if fd.size > 1 {
                let (buf, rest) = map.split_at(fd.size.into());
                *map = rest;
                let c: Vec<u8> = buf.iter().cloned().filter(|x| *x != 0xFF).collect();
                if c.is_empty() {
                    return Value::None;
                } else if c.len() <= 8 {
                    return match VArray::from_slice(&c) {
                        Some(a) => Value::ArrU8(a),
                        None => Value::None,
                    };
                } else {
                    panic!("2:u8 arr is too long: {}", c.len());
                }
            }
            let val = u8(map);
            if val == 0xFF {
                return Value::None;
            }
            Value::U8(val)
        }
        3 => {
            // sint16
            let number_of_values = fd.size / 2;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("3 i16: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = i16(map, endianness);
            if val == 0x7FFF {
                return Value::None;
            }
            Value::I16(val)
        }
        4 => {
            // uint16
            let number_of_values = fd.size / 2;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                let c: Vec<_> = (0..number_of_values)
                    .map(|_| u16(map, endianness))
                    .filter(|x| *x != 0xFFFF)
                    .collect();
                if c.is_empty() {
                    return Value::None;
                } else if c.len() <= 8 {
                    return match VArray::from_slice(&c) {
                        Some(a) => Value::ArrU16(a),
                        None => Value::None,
                    };
                } else {
                    panic!("2:u8 arr is too long: {}", c.len());
                }
            }
            let val = u16(map, endianness);
            if val == 0xFFFF {
                return Value::None;
            }
            Value::U16(val)
        }
        5 => {
            // sint32
            let number_of_values = fd.size / 4;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("5 i32: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = i32(map, endianness);
            if val == 0x7F_FFF_FFF {
                return Value::None;
            }
            Value::I32(val)
        }
        6 => {
            // uint32
            let number_of_values = fd.size / 4;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("6 u32: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = u32(map, endianness);
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
                match GSTRING.lock() {
                    Ok(mut h) => {
                        let k = match h.keys().max() {
                            Some(k) => k + 1,
                            None => 0,
                        };
                        h.insert(k, s);
                        Value::String(k)
                    }
                    Err(_) => Value::None,
                }
            } else {
                Value::None
            }
        }
        8 => {
            // float32
            let number_of_values = fd.size / 4;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("8 f32: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let uval = u32(map, endianness);
            if uval == 0xFF_FFF_FFF {
                return Value::None;
            }
            let val = f32::from_bits(uval);
            Value::F32(val)
        }
        9 => {
            // float64
            let number_of_values = fd.size / 8;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("9 f64: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let uval = u64(map, endianness);
            if uval == 0xF_FFF_FFF_FFF_FFF_FFF {
                return Value::None;
            }
            let val = f64::from_bits(uval);
            Value::F64(val)
        }
        10 => {
            // uint8z
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if fd.size > 1 {
                println!("10:uint8z {}", fd.size);
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
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("11 u16: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = u16(map, endianness);
            if val == 0x0000 {
                return Value::None;
            }
            Value::U16(val)
        }
        12 => {
            // uint32z
            let number_of_values = fd.size / 4;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("12 u32: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = u32(map, endianness);
            if val == 0x0000_0000 {
                return Value::None;
            }
            Value::U32(val)
        }
        14 => {
            // sint64
            let number_of_values = fd.size / 8;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("14 i64: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = i64(map, endianness);
            if val == 0x7_FFF_FFF_FFF_FFF_FFF {
                return Value::None;
            }
            Value::I64(val)
        }
        15 => {
            // uint64
            let number_of_values = fd.size / 8;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("15 u64: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = u64(map, endianness);
            if val == 0xF_FFF_FFF_FFF_FFF_FFF {
                return Value::None;
            }
            Value::U64(val)
        }
        16 => {
            // uint64z
            let number_of_values = fd.size / 8;
            if skip {
                skip_bytes(map, fd.size);
                return Value::None;
            } else if number_of_values > 1 {
                println!("16 u64: {}", fd.size);
                skip_bytes(map, fd.size);
                return Value::None;
            }
            let val = u64(map, endianness);
            if val == 0x0_000_000_000_000_000 {
                return Value::None;
            }
            Value::U64(val)
        }
        _ => Value::None,
    }
}

#[derive(Clone)]
struct Message {
    kind: MessageType,
    values: Vec<DataField>,
}

#[derive(Clone, Copy)]
enum Value {
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
    ArrU8(VArray<u8>),
    ArrU16(VArray<u16>),
    None,
}
impl Default for Value {
    fn default() -> Self {
        Value::None
    }
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
    fn scale(mut self, val: f32) {
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
    fn offset(mut self, val: i16) {
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

#[derive(Clone, Copy)]
struct VArray<T: Arrayable> {
    length: usize,
    stack: [T; VARRAY_LENGTH],
}
impl<T: Arrayable> VArray<T> {
    fn new() -> Self {
        Self {
            length: 0,
            stack: [Default::default(); VARRAY_LENGTH],
        }
    }

    fn from_slice(v: &[T]) -> Option<Self> {
        match v.len() {
            0 => None,
            _ => {
                let mut a: [T; VARRAY_LENGTH] = [Default::default(); VARRAY_LENGTH];
                v.iter()
                    .enumerate()
                    .take(VARRAY_LENGTH)
                    .for_each(|(i, x)| a[i] = *x);
                Some(Self {
                    length: v.len(),
                    stack: a,
                })
            }
        }
    }

    fn push(&mut self, t: T) -> Result<(), ()> {
        if self.length == VARRAY_LENGTH {
            return Err(());
        }
        let i = self.length + 1;
        self.stack[i] = t;
        self.length = i;
        Ok(())
    }
}

trait Arrayable: Copy + Default {}
impl Arrayable for u8 {}
impl Arrayable for u16 {}
// impl Arrayable for DataField {}
// impl Arrayable for Value {}
