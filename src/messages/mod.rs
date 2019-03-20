use crate::Value;

mod defined_message;
mod defined_message_field;
mod field;
mod types;
pub use self::types::{message_name, type_value};
pub use defined_message::DefinedMessage;
pub use defined_message_field::DefinedMessageField;
pub use field::Field;

include!(concat!(env!("OUT_DIR"), "/message_definitions.rs"));
include!(concat!(env!("OUT_DIR"), "/messages.rs"));

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

pub fn new_record(num: u16) -> Option<Box<dyn DefinedMessage>> {
    message_name(num).and_then(|name| message(name))
}

#[cfg(test)]
mod tests {
    use super::{message, message_name, type_value};
    use crate::DataField;
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
        let d = DataField {
            id: 5,
            value: Some(Value::U64(20)),
        };
        t.process_raw_value(&d);
        let v: Option<&Value> = t.value(5);
        assert_eq!(v.unwrap(), &Value::F64(5.0));
    }

    #[test]
    fn it_uses_offset_and_scale() {
        let mut t = message("gps_metadata").unwrap();
        let n = t.name();
        assert_eq!(n, "Gps Metadata");
        let d = DataField {
            id: 3,
            value: Some(Value::U32(5000)),
        };
        t.process_raw_value(&d);
        let v: Option<&Value> = t.value(3);
        assert_eq!(v.unwrap(), &Value::F64(500.0));
    }

}
