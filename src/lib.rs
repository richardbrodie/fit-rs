use copyless::VecHelper;
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
mod arrayable;
mod field_definition;
mod io;

use arrayable::*;
use field_definition::FieldDefinition;
use io::*;
pub use sdk::MessageType;
use sdk::{
    enum_type, match_message_field, match_message_offset, match_message_scale, match_message_type,
    FieldType,
};

lazy_static! {
    static ref GSTRING: Mutex<HashMap<u8, String>> = Mutex::new(HashMap::with_capacity(64));
}

const VARRAY_LENGTH: usize = 16;

const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

const FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10_000_000;
const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00_011_111;

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

pub fn run(path: &PathBuf) -> Vec<Message> {
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
            let mut values: [DataField; 46] = [Default::default(); 46];

            // read all fields, regardless if we already know we won't process the results further
            // otherwise we'll lose our place in the file
            let mut valid_fields = 0;
            for i in 0..d.field_definitions.len() {
                let fd = &d.field_definitions[i];
                let d = fd.read_raw_field(d.endianness, &mut buf, skip);
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
                let mut final_values: Vec<DataField> = Vec::with_capacity(valid_fields);
                final_values.extend(
                    values
                        .iter_mut()
                        .take(valid_fields)
                        .filter(|v| v.field_num < fields.len())
                        .map(|v| {
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
                                            std::mem::replace(&mut v.value, Value::Enum(t));
                                        }
                                    } else if let Value::U16(k) = v.value {
                                        if let Some(t) = enum_type(f, k) {
                                            std::mem::replace(&mut v.value, Value::Enum(t));
                                        }
                                    }
                                }
                            }
                            *v
                        }),
                );
                let m = Message {
                    values: final_values,
                    kind: m,
                };
                records.alloc().init(m);
            }
        }
        if buf.len() <= 14 {
            break;
        }
    }
    records
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
    local_num: u8,
}
impl HeaderByte {
    fn new(map: &mut &[u8]) -> Self {
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

struct DefinitionRecord {
    endianness: Endianness,
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
}
impl DefinitionRecord {
    fn new(map: &mut &[u8]) -> Self {
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

#[derive(Clone, Copy, Debug)]
pub struct DataField {
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

#[derive(Clone)]
pub struct Message {
    pub kind: MessageType,
    pub values: Vec<DataField>,
}

#[derive(Clone, Copy, Debug)]
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
