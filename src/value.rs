//////////
//// Value
//////////

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    Enum(&'static str),
    String(String),
    F32(f32),
    F64(f64),
    I64(i64),
    U64(u64),
    Time(u32),
    ArrU8(Vec<u8>),
    ArrU16(Vec<u16>),
    ArrU32(Vec<u32>),
}
impl Value {
    pub(super) fn scale(&mut self, val: f32) {
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
    pub(super) fn offset(&mut self, val: i16) {
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
