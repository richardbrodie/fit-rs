use super::arrayable::VArray;
use super::io::*;
use super::Value;

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
        let base_num = buf[2] & super::FIELD_DEFINITION_BASE_NUMBER;
        Self {
            definition_number: buf[0].into(),
            size: buf[1],
            base_type: base_num,
        }
    }
    pub fn read_raw_field(&self, endianness: Endianness, map: &mut &[u8], skip: bool) -> Value {
        match self.base_type {
            0 | 13 => {
                // enum / byte
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if self.size > 1 {
                    println!("0/13:enum/byte: {}", self.size);
                    skip_bytes(map, self.size);
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
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if self.size > 1 {
                    println!("1 i8: {}", self.size);
                    skip_bytes(map, self.size);
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
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if self.size > 1 {
                    let (buf, rest) = map.split_at(self.size.into());
                    *map = rest;
                    let c: Vec<u8> = buf.iter().cloned().filter(|x| *x != 0xFF).collect();
                    if c.is_empty() {
                        return Value::None;
                    } else if c.len() <= 16 {
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
                let number_of_values = self.size / 2;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("3 i16: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 2;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    let c: Vec<_> = (0..number_of_values)
                        .map(|_| u16(map, endianness))
                        .filter(|x| *x != 0xFFFF)
                        .collect();
                    if c.is_empty() {
                        return Value::None;
                    } else if c.len() <= 16 {
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
                let number_of_values = self.size / 4;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("5 i32: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 4;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("TODO: 6 u32: {}", self.size);
                    skip_bytes(map, self.size);
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
                let (buf, rest) = map.split_at(self.size as usize);
                *map = rest;
                let buf: Vec<u8> = buf.iter().filter(|b| *b != &0x00).cloned().collect();
                if let Ok(s) = String::from_utf8(buf) {
                    match super::GSTRING.lock() {
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
                let number_of_values = self.size / 4;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("8 f32: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("9 f64: {}", self.size);
                    skip_bytes(map, self.size);
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
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if self.size > 1 {
                    println!("10:uint8z {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 2;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("11 u16: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 4;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("12 u32: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("14 i64: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("15 u64: {}", self.size);
                    skip_bytes(map, self.size);
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
                let number_of_values = self.size / 8;
                if skip {
                    skip_bytes(map, self.size);
                    return Value::None;
                } else if number_of_values > 1 {
                    println!("16 u64: {}", self.size);
                    skip_bytes(map, self.size);
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
}
