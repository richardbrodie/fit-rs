#![warn(unstable_name_collisions)]
use std::ops::{Add, Mul};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Enum(u8),
    Str(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Time(u32),
}
impl Mul<f64> for Value {
    type Output = Value;
    fn mul(self, rhs: f64) -> Value {
        match self {
            Value::U8(v) => match v.checked_mul(rhs as u8) {
                Some(r) => Value::U8(r),
                None => Value::U16(v as u16) * rhs,
            },
            Value::U16(v) => match v.checked_mul(rhs as u16) {
                Some(r) => Value::U16(r),
                None => Value::U32(v as u32) * rhs,
            },
            Value::U32(v) => match v.checked_mul(rhs as u32) {
                Some(r) => Value::U32(r),
                None => Value::U64(v as u64) * rhs,
            },
            Value::U64(v) => match v.checked_mul(rhs as u64) {
                Some(r) => Value::U64(r),
                None => panic!("can't multiply a u64 by an f64"),
            },
            Value::I8(v) => match v.checked_mul(rhs as i8) {
                Some(r) => Value::I8(r),
                None => Value::I16(v as i16) * rhs,
            },
            Value::I16(v) => match v.checked_mul(rhs as i16) {
                Some(r) => Value::I16(r),
                None => Value::I32(v as i32) * rhs,
            },
            Value::I32(v) => match v.checked_mul(rhs as i32) {
                Some(r) => Value::I32(r),
                None => Value::I64(v as i64) * rhs,
            },
            Value::I64(v) => match v.checked_mul(rhs as i64) {
                Some(r) => Value::I64(r),
                None => panic!("can't multiply an i64 by an f64"),
            },
            Value::F32(v) => Value::F64((v as f64) * rhs),
            Value::F64(v) => Value::F64(v * rhs),
            _ => self,
        }
    }
}
impl Add<f64> for Value {
    type Output = Value;
    fn add(self, rhs: f64) -> Value {
        match self {
            Value::U8(v) => match v.checked_add(rhs as u8) {
                Some(r) => Value::U8(r),
                None => Value::U16(v as u16) + rhs,
            },
            Value::U16(v) => match v.checked_add(rhs as u16) {
                Some(r) => Value::U16(r),
                None => Value::U32(v as u32) + rhs,
            },
            Value::U32(v) => match v.checked_add(rhs as u32) {
                Some(r) => Value::U32(r),
                None => Value::U64(v as u64) + rhs,
            },
            Value::U64(v) => match v.checked_add(rhs as u64) {
                Some(r) => Value::U64(r),
                None => panic!("can't add u64 and f64"),
            },
            Value::I8(v) => match v.checked_add(rhs as i8) {
                Some(r) => Value::I8(r),
                None => Value::I16(v as i16) + rhs,
            },
            Value::I16(v) => match v.checked_add(rhs as i16) {
                Some(r) => Value::I16(r),
                None => Value::I32(v as i32) + rhs,
            },
            Value::I32(v) => match v.checked_add(rhs as i32) {
                Some(r) => Value::I32(r),
                None => Value::I64(v as i64) + rhs,
            },
            Value::I64(v) => match v.checked_add(rhs as i64) {
                Some(r) => Value::I64(r),
                None => panic!("can't add i64 and f64"),
            },
            Value::F32(v) => Value::F64((v as f64) + rhs),
            Value::F64(v) => Value::F64(v + rhs),
            _ => self,
        }
    }
}

impl Value {
    pub fn scale(self, s: Option<f64>) -> Self {
        match s {
            Some(s) => self * s,
            None => self,
        }
    }
    pub fn offset(self, o: Option<f64>) -> Self {
        match o {
            Some(o) => self + o,
            None => self,
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            Value::Str(_s) => true,
            _ => false,
        }
    }
    pub fn is_u8(&self) -> bool {
        match self {
            Value::U8(_v) => true,
            _ => false,
        }
    }
    pub fn is_u16(&self) -> bool {
        match self {
            Value::U16(_v) => true,
            _ => false,
        }
    }
    pub fn is_u32(&self) -> bool {
        match self {
            Value::U32(_v) => true,
            _ => false,
        }
    }
    pub fn is_u64(&self) -> bool {
        match self {
            Value::U64(_v) => true,
            _ => false,
        }
    }
    pub fn is_i8(&self) -> bool {
        match self {
            Value::I8(_v) => true,
            _ => false,
        }
    }
    pub fn is_i16(&self) -> bool {
        match self {
            Value::I16(_v) => true,
            _ => false,
        }
    }
    pub fn is_i32(&self) -> bool {
        match self {
            Value::I32(_v) => true,
            _ => false,
        }
    }
    pub fn is_i64(&self) -> bool {
        match self {
            Value::I64(_v) => true,
            _ => false,
        }
    }
    pub fn is_f32(&self) -> bool {
        match self {
            Value::F32(_v) => true,
            _ => false,
        }
    }
    pub fn is_f64(&self) -> bool {
        match self {
            Value::F64(_v) => true,
            _ => false,
        }
    }
}

impl From<u8> for Value {
    fn from(f: u8) -> Self {
        Value::U8(f)
    }
}
impl From<u16> for Value {
    fn from(f: u16) -> Self {
        Value::U16(f)
    }
}
impl From<u32> for Value {
    fn from(f: u32) -> Self {
        Value::U32(f)
    }
}
impl From<u64> for Value {
    fn from(f: u64) -> Self {
        Value::U64(f)
    }
}
impl From<i8> for Value {
    fn from(f: i8) -> Self {
        Value::I8(f)
    }
}
impl From<i16> for Value {
    fn from(f: i16) -> Self {
        Value::I16(f)
    }
}
impl From<i32> for Value {
    fn from(f: i32) -> Self {
        Value::I32(f)
    }
}
impl From<i64> for Value {
    fn from(f: i64) -> Self {
        Value::I64(f)
    }
}
impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::F32(f)
    }
}
impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::F64(f)
    }
}
impl From<&str> for Value {
    fn from(f: &str) -> Self {
        Value::Str(f.to_owned())
    }
}

#[derive(Debug)]
pub struct ValueError {}
pub trait TryFrom<T> {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl TryFrom<&Value> for u8 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::U8(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
impl TryFrom<&Value> for u16 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::U8(v) => Ok(*v as u16),
            Value::U16(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
impl TryFrom<&Value> for u32 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::U8(v) => Ok(*v as u32),
            Value::U16(v) => Ok(*v as u32),
            Value::U32(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
impl TryFrom<&Value> for u64 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::U8(v) => Ok(*v as u64),
            Value::U16(v) => Ok(*v as u64),
            Value::U32(v) => Ok(*v as u64),
            Value::U64(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
impl TryFrom<&Value> for i8 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::I8(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
impl TryFrom<&Value> for i16 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::I8(v) => Ok(*v as i16),
            Value::I16(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
impl TryFrom<&Value> for i32 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::I8(v) => Ok(*v as i32),
            Value::I16(v) => Ok(*v as i32),
            Value::I32(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
impl TryFrom<&Value> for i64 {
    type Error = ValueError;
    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::I8(v) => Ok(*v as i64),
            Value::I16(v) => Ok(*v as i64),
            Value::I32(v) => Ok(*v as i64),
            Value::I64(v) => Ok(*v),
            _ => Err(ValueError {}),
        }
    }
}
// impl TryFrom<Value> for f32 {
//     type Error = ValueError;
//     fn try_from(val: &Value) -> Result<Self, Self::Error> {
//         match val {
//             Value::I8(v) => Ok(*v as f32),
//             Value::I16(v) => Ok(*v as f32),
//             Value::I32(v) => Ok(*v as f32),
//             _ => Err(ValueError {}),
//         }
//     }
// }
// impl TryFrom<Value> for f64 {
//     type Error = ValueError;
//     fn try_from(val: &Value) -> Result<Self, Self::Error> {
//         match val {
//             Value::I8(v) => Ok(*v as f64),
//             Value::I16(v) => Ok(*v as f64),
//             Value::I32(v) => Ok(*v as f64),
//             Value::I64(v) => Ok(*v as f64),
//             _ => Err(ValueError {}),
//         }
//     }
// }
