use log::warn;

mod structs;
mod types;
pub use self::structs::{DefinedMessageField, DefinedMessageType, FieldNameAndValue};
pub use self::types::{message_name, type_value};
use crate::value::Value;

include!(concat!(env!("OUT_DIR"), "/message_definitions.rs"));
include!(concat!(env!("OUT_DIR"), "/messages.rs"));

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

pub fn new_record(num: u16) -> Option<Box<dyn DefinedMessageType>> {
    message_name(num).and_then(|name| message(name))
}

fn convert_value(val: &Value, field: &DefinedMessageField) -> Option<Value> {
    match field.kind {
        x if x.starts_with("uint") || x.starts_with("sint") => uint_sint(val, field),
        x if x.starts_with("str") => Some(val.clone()),
        x if x.starts_with("man") => manufacturer(val),
        x if x.starts_with("dat") => date_time(val),
        x if x.starts_with("dev") => device_index(val),
        x if x.starts_with("bat") => battery_status(val),
        x if x.starts_with("mes") => message_index(val),
        x if x.starts_with("local_") => local_date_time(val),
        x if x.starts_with("localt") => Some(val.clone()),
        _ => catchall(val, field.kind),
    }
}
fn uint_sint(val: &Value, field: &DefinedMessageField) -> Option<Value> {
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
fn manufacturer(val: &Value) -> Option<Value> {
    if let Value::U16(inner) = val {
        types::type_value("manufacturer", u32::from(*inner)).map(|s| s.into())
    } else {
        warn!("wrong type for manfacturer: {:?}", val);
        None
    }
}
fn date_time(val: &Value) -> Option<Value> {
    if let Value::U32(inner) = val {
        Some(Value::Time(inner + PSEUDO_EPOCH))
    } else {
        warn!("wrong type for timestamp");
        None
    }
}
fn device_index(val: &Value) -> Option<Value> {
    if let Value::U8(inner) = val {
        types::type_value("device_index", u32::from(*inner)).map(|s| s.into())
    } else {
        warn!("wrong type for device index: {:?}", val);
        None
    }
}
fn battery_status(val: &Value) -> Option<Value> {
    if let Value::U8(inner) = *val {
        types::type_value("battery_status", u32::from(inner)).map(|s| s.into())
    } else {
        warn!("wrong type for battery_status: {:?}", val);
        None
    }
}
fn message_index(val: &Value) -> Option<Value> {
    if let Value::U16(inner) = val {
        types::type_value("message_index", u32::from(*inner)).map(|s| s.into())
    } else {
        warn!("wrong type for message_index: {:?}", val);
        None
    }
}
fn local_date_time(val: &Value) -> Option<Value> {
    if let Value::U32(inner) = val {
        Some(Value::Time(inner + PSEUDO_EPOCH - 3600)) // hardcoded to +0100
    } else {
        warn!("wrong type for timestamp");
        None
    }
}

fn catchall(val: &Value, kind: &str) -> Option<Value> {
    if let Value::Enum(inner) = val {
        types::type_value(kind, u32::from(*inner)).map(|e| e.into())
    } else {
        warn!("wrong type for `{}`: {:?}", kind, &val);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{message, message_name, type_value};
    use crate::Value;

    #[test]
    fn it_gets_message_name() {
        let t = message_name(1);
        assert_eq!(t, Some("capabilities"));
    }

    #[test]
    fn it_gets_type() {
        let t = type_value("file", 4);
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
        let v: Option<&Value> = t.value(0);
        assert_eq!(v.unwrap(), &Value::U32(12));
    }

    #[test]
    fn it_uses_scale() {
        let mut t = message("device_settings").unwrap();
        let n = t.name();
        assert_eq!(n, "Device Settings");
        t.process_raw_value(5, &Value::U32(20));
        let v: Option<&Value> = t.value(5);
        assert_eq!(v.unwrap(), &Value::F64(5.0));
    }

    #[test]
    fn it_uses_offset_and_scale() {
        let mut t = message("gps_metadata").unwrap();
        let n = t.name();
        assert_eq!(n, "Gps Metadata");
        t.process_raw_value(3, &Value::U32(5000));
        let v: Option<&Value> = t.value(3);
        assert_eq!(v.unwrap(), &Value::F64(500.0));
    }

}
