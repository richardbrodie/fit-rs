/// The container type for the different values.
#[derive(Clone)]
pub enum Value {
    Enum(u8),
    String(String),
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
    Array(Vec<Value>),
}

impl Value {
    pub fn scale(self, s: Option<f64>) -> Self {
        match s {
            None => self,
            Some(rhs) => match self {
                Value::U8(v) => Value::F64(f64::from(v) / rhs),
                Value::U16(v) => Value::F64(f64::from(v) / rhs),
                Value::U32(v) => Value::F64(f64::from(v) / rhs),
                Value::U64(v) => Value::F64(v as f64 / rhs),
                Value::I8(v) => Value::F64(f64::from(v) / rhs),
                Value::I16(v) => Value::F64(f64::from(v) / rhs),
                Value::I32(v) => Value::F64(f64::from(v) / rhs),
                Value::I64(v) => Value::F64(v as f64 / rhs),
                Value::F32(v) => Value::F64(f64::from(v) / rhs),
                Value::F64(v) => Value::F64(v / rhs),
                _ => self,
            },
        }
    }
    pub fn offset(self, o: Option<f64>) -> Self {
        match o {
            None => self,
            Some(rhs) => match self {
                Value::U8(v) => Value::F64(f64::from(v) - rhs),
                Value::U16(v) => Value::F64(f64::from(v) - rhs),
                Value::U32(v) => Value::F64(f64::from(v) - rhs),
                Value::U64(v) => Value::F64(v as f64 - rhs),
                Value::I8(v) => Value::F64(f64::from(v) - rhs),
                Value::I16(v) => Value::F64(f64::from(v) - rhs),
                Value::I32(v) => Value::F64(f64::from(v) - rhs),
                Value::I64(v) => Value::F64(v as f64 - rhs),
                Value::F32(v) => Value::F64(f64::from(v) - rhs),
                Value::F64(v) => Value::F64(v - rhs),
                _ => self,
            },
        }
    }

    pub fn u8(&self) -> u8 {
        if let Value::U8(i) = self {
            *i
        } else {
            panic!("not a u8")
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match self {
            Value::U8(v) => {
                if let Value::U8(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing U8 {:?} with {:?}", self, other)
                }
            }
            Value::U16(v) => {
                if let Value::U16(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing U16 {:?} with {:?}", self, other)
                }
            }
            Value::U32(v) => {
                if let Value::U32(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing U32 {:?} with {:?}", self, other)
                }
            }
            Value::U64(v) => {
                if let Value::U64(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing U64 {:?} with {:?}", self, other)
                }
            }
            Value::I8(v) => {
                if let Value::I8(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing I8 {:?} with {:?}", self, other)
                }
            }
            Value::I16(v) => {
                if let Value::I16(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing I16 {:?} with {:?}", self, other)
                }
            }
            Value::I32(v) => {
                if let Value::I32(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing I32 {:?} with {:?}", self, other)
                }
            }
            Value::I64(v) => {
                if let Value::I64(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing I64 {:?} with {:?}", self, other)
                }
            }
            Value::F32(v) => match other {
                Value::U32(ov) => {
                    let f = *ov as f32;
                    *v == f
                }
                Value::F32(f) => v == f,
                _ => panic!("failed comparing F32 {:?} with {:?}", self, other),
            },
            Value::F64(v) => match other {
                Value::U64(ov) => {
                    let f = *ov as f64;
                    *v == f
                }
                Value::F64(f) => v == f,
                _ => panic!("failed comparing F64 {:?} with {:?}", self, other),
            },
            Value::Time(v) => {
                if let Value::Time(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing Time {:?} with {:?}", self, other)
                }
            }
            Value::String(v) => {
                if let Value::String(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing Str {:?} with {:?}", self, other)
                }
            }
            Value::Enum(v) => {
                if let Value::Enum(ov) = other {
                    v == ov
                } else {
                    panic!("failed comparing Enum {:?} with {:?}", self, other)
                }
            }
            Value::Array(_v) => true,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::U8(v) => write!(f, "{}", v),
            Value::U16(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
            Value::Time(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "\"{}\"", v),
            Value::Enum(v) => write!(f, "\"{}\"", v),
            Value::Array(v) => write!(f, "\"{:?}\"", v),
        }
    }
}
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::U8(v) => write!(f, "{} /u8", v),
            Value::U16(v) => write!(f, "{} /u16", v),
            Value::U32(v) => write!(f, "{} /u32", v),
            Value::U64(v) => write!(f, "{} /u64", v),
            Value::I8(v) => write!(f, "{} /i8", v),
            Value::I16(v) => write!(f, "{} /i16", v),
            Value::I32(v) => write!(f, "{} /i32", v),
            Value::I64(v) => write!(f, "{} /i64", v),
            Value::F32(v) => write!(f, "{} /f32", v),
            Value::F64(v) => write!(f, "{} /f64", v),
            Value::Time(v) => write!(f, "{} /time", v),
            Value::String(v) => write!(f, "\"{}\" /str", v),
            Value::Enum(v) => write!(f, "\"{}\" /enum", v),
            Value::Array(v) => write!(f, "\"{:?}\" /arr", v),
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
        Value::String(f.to_owned())
    }
}
impl From<String> for Value {
    fn from(f: String) -> Self {
        Value::String(f)
    }
}
impl From<&[Value]> for Value {
    fn from(v: &[Value]) -> Self {
        Value::Array(v.to_vec())
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}
