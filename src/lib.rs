use arrayvec::ArrayString;
use copyless::VecHelper;
use memmap::MmapOptions;
use std::collections::VecDeque;
use std::{fs::File, path::PathBuf, mem::MaybeUninit};
use fitsdk::{
    FieldType,
    MessageType,
    match_messagetype,
    match_custom_field_value,
    match_message_field,
    match_message_offset,
    match_message_scale,
    match_message_timestamp_field
};

mod developer_fields;
mod io;

use developer_fields::{DeveloperFieldDefinition, DeveloperFieldDescription};
use io::*;

const COMPRESSED_HEADER_MASK: u8 = 0b1000_0000; // MASK: determine if the header has compressed timestamp
const COMPRESSED_HEADER_LOCAL_MESSAGE_NUMBER_MASK: u8 = 0b0110_0000; // MASK: Extract message number from a compressed header
const COMPRESSED_HEADER_TIME_OFFSET_MASK: u8 = 0b0001_1111; // MASK: Extract timestamp offset from a compressed header
const COMPRESSED_HEADER_TIME_OFFSET_ROLLOVER: u32 = 0b0010_0000; // Compressed header: rollover to eventually add when computing the new timestamp
const COMPRESSED_HEADER_LAST_TIMESTAMP_MASK: u32 = 0xFFFF_FFE0; // Compressed header: mask to apply to the previous timestamp before adding the time offset

const DEFINITION_HEADER_MASK: u8 = 0x40;
const DEVELOPER_FIELDS_MASK: u8 = 0x20;
const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

const _FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10_000_000;
const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00_011_111;

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

const MAGIC_BUFFER_LENGTH: usize = 128;

pub fn run(path: &PathBuf) -> Vec<Message> {
    let file = File::open(path).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let mut buf: &[u8] = &mmap;

    let _fh = FileHeader::new(&mut buf);
    let mut q: VecDeque<(u8, DefinitionRecord)> = VecDeque::new();
    let mut records: Vec<Message> = Vec::with_capacity(2500);
    let mut developer_field_descriptions: Vec<DeveloperFieldDescription> = Vec::new();

    let mut datafield_buffer = {
        let mut data: [MaybeUninit<DataField>; MAGIC_BUFFER_LENGTH] = unsafe { MaybeUninit::uninit().assume_init() };
        for elem in &mut data[..] {
            *elem = MaybeUninit::new(DataField::default());
        }
        unsafe { std::mem::transmute::<_, [DataField; MAGIC_BUFFER_LENGTH]>(data) }
    };
    let mut fielddefinition_buffer = {
        let mut data: [MaybeUninit<FieldDefinition>; MAGIC_BUFFER_LENGTH] = unsafe { MaybeUninit::uninit().assume_init() };
        for elem in &mut data[..] {
            *elem = MaybeUninit::new(FieldDefinition::default());
        }
        unsafe { std::mem::transmute::<_, [FieldDefinition; MAGIC_BUFFER_LENGTH]>(data) }
    };

    let mut last_timestamp: u32 = 0;
    loop {
        let h = HeaderByte::new(&mut buf);
        if h.definition {
            let d = DefinitionRecord::new(&mut buf, &mut fielddefinition_buffer, h.dev_fields);
            q.push_front((h.local_num, d));
        } else if let Some((_, d)) = q.iter().find(|x| x.0 == h.local_num) {
            let m = match_messagetype(d.global_message_number);

            // we must read all fields, regardless if we already know we won't
            // process the results further otherwise we'll lose our place in the file
            let mut valid_fields = 0;
            for fd in d.field_definitions.iter() {
                let data = read_next_field(
                    fd.base_type,
                    fd.size,
                    d.endianness,
                    &mut buf,
                    m == MessageType::None,
                );

                match data {
                    Value::None => {},
                    _ => {
                        // it's 'safe' to use `unsafe` here because we make sure to keep
                        // track of the number of values we've written and only read those
                        unsafe {
                            std::ptr::write(
                                &mut datafield_buffer[valid_fields],
                                DataField::new(fd.definition_number, data),
                            );
                        }
                        valid_fields += 1;
                    }
                }
            }

            // if this is a developer field definition
            if m == MessageType::FieldDescription {
                let d = DeveloperFieldDescription::new(
                    datafield_buffer[0..valid_fields].to_vec(),
                );
                developer_field_descriptions.push(d);
            } else {
                // if this record is a message that contains developer-defined fields read those too
                let dev_fields = match &d.developer_fields {
                    Some(dev_fields) => dev_fields
                        .iter()
                        .filter_map(|df| {
                            let def = developer_field_descriptions
                                .iter()
                                .find(|e| {
                                    e.developer_data_index == df.developer_data_index
                                        && e.field_definition_number == df.field_number
                                })
                                .unwrap();
                            match read_next_field(
                                def.fit_base_type,
                                df.size,
                                d.endianness,
                                &mut buf,
                                false,
                            ) {
                                Value::None => None,
                                v => Some(DevDataField::new(
                                    df.developer_data_index,
                                    df.field_number,
                                    v,
                                )),
                            }
                        })
                        .collect(),
                    None => Vec::new(),
                };

                // finally, if we have a valid MessageType and some valid fields, then we have a
                // message we want to save
                if m != MessageType::None && valid_fields > 0 {
                    // no need to look these up until now
                    let scales = match_message_scale(m);
                    let offsets = match_message_offset(m);
                    let fields = match_message_field(m);

                    // datafield_buffer is an array longer than we needed, so only take the number of elements we
                    // need
                    for v in datafield_buffer.iter_mut().take(valid_fields) {
                        // see if the fields need any further processing
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
                                    last_timestamp = *inner;
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
                                    v.value.offset(o)
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

                    if let Some(time_offset) = h.compressed_timestamp() {
                        let mut timestamp = last_timestamp & COMPRESSED_HEADER_LAST_TIMESTAMP_MASK + u32::from(time_offset);
                        if time_offset < (last_timestamp as u8 & COMPRESSED_HEADER_TIME_OFFSET_MASK)
                        {
                            timestamp += COMPRESSED_HEADER_TIME_OFFSET_ROLLOVER
                        };

                        if let Some(timestamp_field_number) = match_message_timestamp_field(m) {
                            unsafe {
                                std::ptr::write(
                                    &mut datafield_buffer[valid_fields],
                                    DataField::new(
                                        timestamp_field_number,
                                        Value::Time(timestamp + PSEUDO_EPOCH),
                                    ),
                                );
                            }
                            valid_fields += 1;
                        }
                    }

                    let msg = Message {
                        values: datafield_buffer[0..valid_fields].to_vec(),
                        kind: m,
                        dev_values: dev_fields,
                    };
                    records.alloc().init(msg);
                }
            }
        }
        if buf.len() <= 14 {
            break;
        }
    }
    records
}

#[derive(Clone, Debug)]
pub struct Message {
    pub kind: MessageType,
    pub values: Vec<DataField>,
    pub dev_values: Vec<DevDataField>,
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

#[derive(Debug)]
struct HeaderByte {
    compressed_header: bool,
    definition: bool,
    dev_fields: bool,
    local_num: u8,
    time_offset: Option<u8>,
}
impl HeaderByte {
    fn new(map: &mut &[u8]) -> Self {
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

struct DefinitionRecord {
    endianness: Endianness,
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
    developer_fields: Option<Vec<DeveloperFieldDefinition>>,
}
impl DefinitionRecord {
    fn new(map: &mut &[u8], buffer: &mut [FieldDefinition; MAGIC_BUFFER_LENGTH], dev_fields: bool) -> Self {
        skip_bytes(map, 1);
        let endian = match read_u8(map) {
            1 => Endianness::Big,
            0 => Endianness::Little,
            _ => panic!("unexpected endian byte"),
        };
        let global_message_number = read_u16(map, endian);
        let number_of_fields = read_u8(map);

        for i in 0..number_of_fields {
            buffer[i as usize] = FieldDefinition::new(map);
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
            field_definitions: buffer[0..number_of_fields as usize].to_vec(),
            developer_fields: dev_fields,
        }
    }
}

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
    fn default() -> Self {
        Self::new(0, Value::None)
    }
}
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

#[derive(Clone, Debug)]
pub enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    Enum(&'static str),
    String(ArrayString<[u8;32]>),
    F32(f32),
    F64(f64),
    I64(i64),
    U64(u64),
    Time(u32),
    ArrU8(Vec<u8>),
    ArrU16(Vec<u16>),
    ArrU32(Vec<u32>),
    None,
}
impl Value {
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
    fn default() -> Self {
        Self { definition_number: 0, size: 0, base_type: 0 }
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn read_next_field(
    base_type: u8,
    size: u8,
    endianness: Endianness,
    map: &mut &[u8],
    skip: bool,
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
                match read_u8(map){
                    0xFF => Value::None,
                    v => Value::U8(v)
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
                match read_i8(map){
                    0x7F => Value::None,
                    v => Value::I8(v)
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
                    Value::ArrU8(c)
                }
            } else {
                match read_u8(map){
                    0xFF => Value::None,
                    v => Value::U8(v)
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
                let val = read_i16(map, endianness);
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
                    .filter_map(|_| {
                            match read_u16(map, endianness){
                                0xFFFF => None,
                                v => Some(v)
                            }
                        }).collect();
                if c.is_empty() {
                    Value::None
                } else {
                    Value::ArrU16(c)
                }
            } else {
                let val = read_u16(map, endianness);
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
                let val = read_i32(map, endianness);
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
                        .filter_map(|_| {
                            match read_u32(map, endianness){
                                0xFFFF_FFFF => None,
                                v => Some(v)
                            }
                        }).collect();
                if c.is_empty() {
                    Value::None
                } else {
                    Value::ArrU32(c)
                }
            } else {
                let val = read_u32(map, endianness);
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
            if let Ok(s) = std::str::from_utf8(&buf) {
                let mut string = ArrayString::<[_; 32]>::new();
                string.push_str(s);
                Value::String(string)
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
                let uval = read_u32(map, endianness);
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
                let uval = read_u64(map, endianness);
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
                let val = read_u8(map);
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
                let val = read_u16(map, endianness);
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
                let val = read_u32(map, endianness);
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
                let val = read_i64(map, endianness);
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
                let val = read_u64(map, endianness);
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
                let val = read_u64(map, endianness);
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
