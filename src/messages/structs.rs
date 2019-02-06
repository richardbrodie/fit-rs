use super::convert_value;
use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DefinedMessageField {
    pub num: u16,
    pub name: &'static str,
    pub kind: &'static str,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
}

pub struct FieldNameAndValue<'a> {
    pub name: &'static str,
    pub value: Option<&'a Value>,
}
impl<'a> std::fmt::Display for FieldNameAndValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.value {
            Some(val) => write!(f, "{}: {}", self.name, val),
            None => write!(f, "{}: None", self.name),
        }
    }
}
impl<'a> std::fmt::Debug for FieldNameAndValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.value {
            Some(val) => write!(f, "{}: {:?}", self.name, val),
            None => write!(f, "{}: None", self.name),
        }
    }
}

pub trait DefinedMessageType: Sync + Send {
    // public
    fn new() -> Self
    where
        Self: Sized;

    fn name(&self) -> &str;

    fn process_raw_value(&mut self, num: u16, val: Value) {
        match self.defined_message_field(num) {
            Some(field) => {
                convert_value(val, field).map(|v| self.write_value(num, v));
            }
            None => (),
        }
    }

    fn field(&self, num: u16) -> Option<FieldNameAndValue> {
        self.defined_message_field(num).map(|f| FieldNameAndValue {
            name: f.name,
            value: self.read_value(num),
        })
    }

    fn fields(&self) -> Vec<FieldNameAndValue> {
        self.inner()
            .iter()
            .map(|(k, v)| FieldNameAndValue {
                name: self.defined_message_field(*k).unwrap().name,
                value: Some(v),
            })
            .collect()
    }
    // internal
    fn inner(&self) -> &HashMap<u16, Value>;

    fn defined_message_field(&self, num: u16) -> Option<&DefinedMessageField>;

    fn read_value(&self, num: u16) -> Option<&Value>;

    fn write_value(&mut self, num: u16, val: Value);
}
