use crate::{DataField, Value};
use fit_sdk::{new_record, type_value, DefinedMessage, DefinedMessageField};
use log::warn;

const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
const PSEUDO_EPOCH: u32 = 631_065_600;

pub struct Message {
    defined_message: Box<dyn DefinedMessage>,
    pub values: Vec<(u16, Value)>,
}
impl Message {
    pub fn new(num: u16) -> Option<Self> {
        new_record(num).map(|r| Self {
            values: Vec::with_capacity((&r).size()),
            defined_message: r,
        })
    }

    pub fn name(&self) -> &str {
        self.defined_message.name()
    }

    pub fn add_value(&mut self, data: &DataField) {
        if let Some(field) = self.defined_message.defined_message_field(data.id) {
            if let Some(val) = &data.value {
                if let Some(v) = convert_value(&field, &val) {
                    self.values.push((data.id, v))
                }
            }
        }
    }

    pub fn get_value(&self, num: u16) -> Option<&Value> {
        match self.values.iter().find(|x| x.0 == num) {
            Some(x) => Some(&x.1),
            None => None,
        }
    }
}

pub fn convert_value(field: &DefinedMessageField, val: &Value) -> Option<Value> {
    match field.kind {
        x if x.starts_with("uint") || x.starts_with("sint") => uint_sint(field, val),
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

fn uint_sint(field: &DefinedMessageField, val: &Value) -> Option<Value> {
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
        type_value("manufacturer", u32::from(*inner)).map(|s| s.into())
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
        type_value("device_index", u32::from(*inner)).map(|s| s.into())
    } else {
        warn!("wrong type for device index: {:?}", val);
        None
    }
}
fn battery_status(val: &Value) -> Option<Value> {
    if let Value::U8(inner) = *val {
        type_value("battery_status", u32::from(inner)).map(|s| s.into())
    } else {
        warn!("wrong type for battery_status: {:?}", val);
        None
    }
}
fn message_index(val: &Value) -> Option<Value> {
    if let Value::U16(inner) = val {
        type_value("message_index", u32::from(*inner)).map(|s| s.into())
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
        type_value(kind, u32::from(*inner)).map(|e| e.into())
    } else {
        warn!("wrong type for `{}`: {:?}", kind, &val);
        None
    }
}
