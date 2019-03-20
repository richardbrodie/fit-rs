use crate::Value;
use log::warn;

/// A collection of information about a specific message field, as defined in the FIT SDK.
#[derive(Debug)]
pub struct DefinedMessageField {
    pub num: u16,
    pub name: &'static str,
    pub kind: &'static str,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
}
impl DefinedMessageField {
    pub fn convert_value(&self, val: &Value) -> Option<Value> {
        match self.kind {
            x if x.starts_with("uint") || x.starts_with("sint") => self.uint_sint(val),
            x if x.starts_with("str") => Some(val.clone()),
            x if x.starts_with("man") => Self::manufacturer(val),
            x if x.starts_with("dat") => Self::date_time(val),
            x if x.starts_with("dev") => Self::device_index(val),
            x if x.starts_with("bat") => Self::battery_status(val),
            x if x.starts_with("mes") => Self::message_index(val),
            x if x.starts_with("local_") => Self::local_date_time(val),
            x if x.starts_with("localt") => Some(val.clone()),
            _ => Self::catchall(val, self.kind),
        }
    }
    fn uint_sint(&self, val: &Value) -> Option<Value> {
        if self.name.ends_with("_lat") || self.name.ends_with("_long") {
            if let Value::I32(inner) = val {
                let coord = *inner as f32 * super::COORD_SEMICIRCLES_CALC;
                Some(Value::F32(coord))
            } else {
                warn!("wrong type for coordinate");
                None
            }
        } else {
            Some(val.clone().scale(self.scale).offset(self.offset))
        }
    }
    fn manufacturer(val: &Value) -> Option<Value> {
        if let Value::U16(inner) = val {
            super::type_value("manufacturer", u32::from(*inner)).map(|s| s.into())
        } else {
            warn!("wrong type for manfacturer: {:?}", val);
            None
        }
    }
    fn date_time(val: &Value) -> Option<Value> {
        if let Value::U32(inner) = val {
            Some(Value::Time(inner + super::PSEUDO_EPOCH))
        } else {
            warn!("wrong type for timestamp");
            None
        }
    }
    fn device_index(val: &Value) -> Option<Value> {
        if let Value::U8(inner) = val {
            super::types::type_value("device_index", u32::from(*inner)).map(|s| s.into())
        } else {
            warn!("wrong type for device index: {:?}", val);
            None
        }
    }
    fn battery_status(val: &Value) -> Option<Value> {
        if let Value::U8(inner) = *val {
            super::types::type_value("battery_status", u32::from(inner)).map(|s| s.into())
        } else {
            warn!("wrong type for battery_status: {:?}", val);
            None
        }
    }
    fn message_index(val: &Value) -> Option<Value> {
        if let Value::U16(inner) = val {
            super::types::type_value("message_index", u32::from(*inner)).map(|s| s.into())
        } else {
            warn!("wrong type for message_index: {:?}", val);
            None
        }
    }
    fn local_date_time(val: &Value) -> Option<Value> {
        if let Value::U32(inner) = val {
            Some(Value::Time(inner + super::PSEUDO_EPOCH - 3600)) // hardcoded to +0100
        } else {
            warn!("wrong type for timestamp");
            None
        }
    }

    fn catchall(val: &Value, kind: &str) -> Option<Value> {
        if let Value::Enum(inner) = val {
            super::types::type_value(kind, u32::from(*inner)).map(|e| e.into())
        } else {
            warn!("wrong type for `{}`: {:?}", kind, &val);
            None
        }
    }
}
