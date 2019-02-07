use log::warn;
use std::collections::HashMap;

mod structs;
mod types;
pub use self::structs::{DefinedMessageField, DefinedMessageType, FieldNameAndValue};
pub use self::types::{message_name, type_value};
use crate::value::Value;

include!(concat!(env!("OUT_DIR"), "/message_definitions.rs"));
include!(concat!(env!("OUT_DIR"), "/messages.rs"));

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631065600;

pub fn new_record(num: &u16) -> Option<Box<dyn DefinedMessageType>> {
    message_name(num).and_then(|name| message(name))
}

fn convert_value(val: &Value, field: &DefinedMessageField) -> Option<Value> {
    match field.kind {
        x if x.starts_with("uint") || x.starts_with("sint") => {
            if field.name.ends_with("_lat") || field.name.ends_with("_long") {
                if let Value::I32(inner) = val {
                    let coord = *inner as f32 * COORD_SEMICIRCLES_CALC;
                    Some(Value::F32(coord))
                } else {
                    warn!("wrong type for coordinate");
                    None
                }
            } else {
                Some(val.clone().scale(field.scale).offset(field.offset))
            }
        }
        "string" => Some(val.clone()),
        "manufacturer" => {
            if let Value::U16(inner) = val {
                types::type_value("manufacturer", &(*inner as u32)).map(|s| s.into())
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
                types::type_value("device_index", &(*inner as u32)).map(|s| s.into())
            } else {
                warn!("wrong type for device index: {:?}", val);
                None
            }
        }
        "battery_status" => {
            if let Value::U8(inner) = *val {
                types::type_value("battery_status", &(inner as u32)).map(|s| s.into())
            } else {
                warn!("wrong type for battery_status: {:?}", val);
                None
            }
        }
        "message_index" => {
            if let Value::U16(inner) = val {
                types::type_value("message_index", &(*inner as u32)).map(|s| s.into())
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
        "localtime_into_day" => Some(val.clone()),
        _ => {
            if let Value::Enum(inner) = val {
                types::type_value(field.kind, &(*inner as u32)).map(|e| e.into())
            } else {
                warn!("wrong type for `{}`: {:?}", field.kind, &val);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{message, message_name, type_value};
    use crate::Value;

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
        let f = t.defined_message_field(0).unwrap();
        let f_n = f.name;
        assert_eq!(f_n, "type");
        t.write_value(0, Value::U32(12));
        let v: Option<&Value> = t.read_value(0);
        assert_eq!(v.unwrap(), &Value::U32(12));
    }

    #[test]
    fn it_uses_scale() {
        let mut t = message("device_settings").unwrap();
        let n = t.name();
        assert_eq!(n, "Device Settings");
        t.process_raw_value(5, &[Value::U32(20)]);
        let v: Option<&Value> = t.read_value(5);
        assert_eq!(v.unwrap(), &Value::F64(5.0));
    }

    #[test]
    fn it_uses_offset_and_scale() {
        let mut t = message("gps_metadata").unwrap();
        let n = t.name();
        assert_eq!(n, "Gps Metadata");
        t.process_raw_value(3, &[Value::U32(5000)]);
        let v: Option<&Value> = t.read_value(3);
        assert_eq!(v.unwrap(), &Value::F64(500.0));
    }

}

// impl From<&u8> for &u32 {
//     fn from(f: &u8) -> Self {
//         *f as u32
//     }
// }
