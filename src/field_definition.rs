#![allow(clippy::cognitive_complexity)]
use super::io::*;
use super::Value;
use std::collections::HashMap;

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
            base_type: buf[2] & super::FIELD_DEFINITION_BASE_NUMBER,
        }
    }
    pub fn read_raw_field(
        &self,
        endianness: Endianness,
        map: &mut &[u8],
        skip: bool,
        global_string_map: &mut HashMap<u8, String>,
    ) -> Value {
        if skip {
            skip_bytes(map, self.size);
            return Value::None;
        }
        match self.base_type {
            0 | 13 => {
                // enum / byte
                if self.size > 1 {
                    println!("0/13:enum/byte: {}", self.size);
                    skip_bytes(map, self.size);
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
                if self.size > 1 {
                    println!("1 i8: {}", self.size);
                    skip_bytes(map, self.size);
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
                if self.size > 1 {
                    let (buf, rest) = map.split_at(self.size.into());
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
                let number_of_values = self.size / 2;
                if number_of_values > 1 {
                    println!("3 i16: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 2;
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
                let number_of_values = self.size / 4;
                if number_of_values > 1 {
                    println!("5 i32: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 4;
                if number_of_values > 1 {
                    println!("TODO: 6 u32: {}", self.size);
                    skip_bytes(map, self.size);
                    Value::None
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
                let (buf, rest) = map.split_at(self.size as usize);
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
                let number_of_values = self.size / 4;
                if number_of_values > 1 {
                    println!("8 f32: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if number_of_values > 1 {
                    println!("9 f64: {}", self.size);
                    skip_bytes(map, self.size);
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
                if self.size > 1 {
                    println!("10:uint8z {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 2;
                if number_of_values > 1 {
                    println!("11 u16: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 4;
                if number_of_values > 1 {
                    println!("12 u32: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if number_of_values > 1 {
                    println!("14 i64: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if number_of_values > 1 {
                    println!("15 u64: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if number_of_values > 1 {
                    println!("16 u64: {}", self.size);
                    skip_bytes(map, self.size);
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
}
