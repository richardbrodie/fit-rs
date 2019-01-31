use log::warn;
use std::collections::HashMap;

use crate::value::{TryFrom, Value};
mod types;
pub use self::types::{message_name, type_value};

include!(concat!(env!("OUT_DIR"), "/message_definitions.rs"));
include!(concat!(env!("OUT_DIR"), "/messages.rs"));

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631065600;

pub fn new_record(num: &u16) -> Option<Box<dyn MessageType>> {
    message_name(num).and_then(|name| message(name))
}

#[derive(Debug)]
pub struct MessageField {
    pub num: u16,
    pub name: &'static str,
    pub kind: &'static str,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
}
#[derive(Debug)]
pub struct Field<'a> {
    pub name: &'static str,
    pub value: Option<&'a Value>,
}

pub trait MessageType {
    // public
    fn new() -> Self
    where
        Self: Sized;
    fn name(&self) -> &str;
    fn add_read_value(&mut self, num: u16, val: Value) {
        match self.get_message_field(num) {
            Some(field) => {
                preprocess_value(val, field).map(|v| self.write_value(num, v));
            }
            None => (),
        }
    }
    fn get_field(&self, num: u16) -> Option<Field> {
        self.get_message_field(num).map(|f| Field {
            name: f.name,
            value: self.get_value(num),
        })
    }

    // internal
    fn get_message_field(&self, num: u16) -> Option<&MessageField>;
    fn get_value(&self, num: u16) -> Option<&Value>;
    fn write_value(&mut self, num: u16, val: Value);
}

fn preprocess_value(val: Value, field: &MessageField) -> Option<Value> {
    match field.kind {
        x if x.starts_with("uint") || x.starts_with("sint") => {
            Some(val.scale(field.scale).offset(field.offset))
        }
        "string" => Some(val),
        "manufacturer" => {
            if let Value::U16(inner) = val {
                types::type_value("manufacturer", &inner.into()).map(|s| s.into())
            } else {
                warn!("wrong type for manfacturer: {:?}", val);
                None
            }
        }
        "date_time" => {
            if let Value::U32(inner) = val {
                Some(Value::Time(inner + PSEUDO_EPOCH))
            } else {
                warn!("wrong type for timestamp");
                None
            }
        }
        "device_index" => {
            if let Value::U8(inner) = val {
                types::type_value("device_index", &inner.into()).map(|s| s.into())
            } else {
                warn!("wrong type for device index: {:?}", val);
                None
            }
        }
        "battery_status" => {
            if let Value::U8(inner) = val {
                types::type_value("battery_status", &inner.into()).map(|s| s.into())
            } else {
                warn!("wrong type for battery_status: {:?}", val);
                None
            }
        }
        "message_index" => {
            if let Value::U16(inner) = val {
                types::type_value("message_index", &inner.into()).map(|s| s.into())
            } else {
                warn!("wrong type for message_index: {:?}", val);
                None
            }
        }
        "local_date_time" => {
            if let Value::U32(inner) = val {
                Some(Value::Time(inner + PSEUDO_EPOCH - 3600)) // hardcoded to +0100
            } else {
                warn!("wrong type for timestamp");
                None
            }
        }
        "localtime_into_day" => Some(val),
        x if x.ends_with("_lat") || x.ends_with("_long") => {
            if let Value::I32(inner) = val {
                let coord = inner as f32 * COORD_SEMICIRCLES_CALC;
                Some(Value::F32(coord))
            } else {
                warn!("wrong type for coordinate");
                None
            }
        }
        _ => {
            if let Value::Enum(inner) = val {
                types::type_value(field.kind, &inner.into()).map(|e| e.into())
            } else {
                warn!("wrong type for `{}`: {:?}", field.kind, &val);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_message_name() {
        let t = message_name(&1);
        assert_eq!(t, Some("capabilities"));
    }

    #[test]
    fn it_gets_type() {
        let t = type_value("file", &4);
        assert_eq!(t, Some("activity"));
    }

    #[test]
    fn it_gets_message() {
        let mut t = message("file_id").unwrap();
        let n = t.name();
        assert_eq!(n, "File Id");
        let f = t.get_field(0).unwrap();
        let f_n = f.name;
        assert_eq!(f_n, "type");
        t.write_value(0, Value::U32(12));
        let v: Option<&Value> = t.get_value(0);
        assert_eq!(v.unwrap(), &Value::U32(12));
    }

    #[test]
    fn it_uses_scale() {
        let mut t = message("device_settings").unwrap();
        let n = t.name();
        assert_eq!(n, "Device Settings");
        t.add_read_value(5, Value::U32(20));
        let v: Option<&Value> = t.get_value(5);
        assert_eq!(v.unwrap(), &Value::U32(80));
    }

    #[test]
    fn it_uses_offset_and_scale() {
        let mut t = message("gps_metadata").unwrap();
        let n = t.name();
        assert_eq!(n, "Gps Metadata");
        t.add_read_value(3, Value::U32(5));
        let v: Option<&Value> = t.get_value(3);
        assert_eq!(v.unwrap(), &Value::U32(525));
    }
}
