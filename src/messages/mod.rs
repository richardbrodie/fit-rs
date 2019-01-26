use std::collections::HashMap;

use crate::Value;
mod types;
pub use self::types::{message_name, type_value};

include!(concat!(env!("OUT_DIR"), "/message_definitions.rs"));
include!(concat!(env!("OUT_DIR"), "/messages.rs"));

pub fn new_record(num: &u16) -> Option<Box<dyn MessageType>> {
    message_name(num).and_then(|name| message(name))
}

#[derive(Debug)]
pub struct FitField {
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
    fn new() -> Self
    where
        Self: Sized;

    // public
    fn name(&self) -> &str;
    fn add_value(&mut self, num: u16, val: Value) {
        if let Some(field) = self.get_fit_field(num) {
            let val = val.offset(field.offset).scale(field.scale);
            self.insert_value(num, val);
        }
    }
    fn get_field(&self, num: u16) -> Option<Field> {
        match self.get_fit_field(num) {
            Some(f) => Some(Field {
                name: f.name,
                value: self.get_value(num),
            }),
            None => None,
        }
    }

    // internal
    fn get_fit_field(&self, num: u16) -> Option<&FitField>;
    fn get_value(&self, num: u16) -> Option<&Value>;
    fn insert_value(&mut self, num: u16, val: Value);
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
        t.insert_value(0, Value::U32(12));
        let v: Option<&Value> = t.get_value(0);
        assert_eq!(v.unwrap(), &Value::U32(12));
    }

    #[test]
    fn it_uses_scale() {
        let mut t = message("device_settings").unwrap();
        let n = t.name();
        assert_eq!(n, "Device Settings");
        t.add_value(5, Value::U32(20));
        let v: Option<&Value> = t.get_value(5);
        assert_eq!(v.unwrap(), &Value::U32(80));
    }

    #[test]
    fn it_uses_offset_and_scale() {
        let mut t = message("gps_metadata").unwrap();
        let n = t.name();
        assert_eq!(n, "Gps Metadata");
        t.add_value(3, Value::U32(5));
        let v: Option<&Value> = t.get_value(3);
        assert_eq!(v.unwrap(), &Value::U32(2525));
    }
}
