use log::warn;

use crate::value::Value;
use crate::DataField;

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
    fn convert_value(&self, val: &Value) -> Option<Value> {
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

/// The name and parsed value of a message field.
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

/// A trait representing all the different message types as defined in the FIT SDK.
pub trait DefinedMessageType {
    // public
    fn new() -> Self
    where
        Self: Sized;

    /// The name of the underlying message, as defined in the SDK.
    ///
    /// For example, "Record", "Session", "Device Settings", etc
    fn name(&self) -> &str;

    fn process_raw_value(&mut self, data: &DataField) {
        if let Some(field) = self.defined_message_field(data.id) {
            if let Some(val) = &data.value {
                field
                    .convert_value(&val)
                    .map(|v| self.write_value(data.id, v));
            }
        }
    }

    /// Extract the name and value of a specific field number, if used.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    fn field_name_and_value(&self, num: u16) -> Option<FieldNameAndValue> {
        self.defined_message_field(num).map(|f| FieldNameAndValue {
            name: f.name,
            value: self.value(num),
        })
    }

    /// Extract a collection of the names and values of all used fields.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    fn all_values(&self) -> Vec<FieldNameAndValue> {
        self.inner()
            .iter()
            .filter_map(|(k, v)| match self.defined_message_field(*k) {
                Some(f) => Some(FieldNameAndValue {
                    name: f.name,
                    value: Some(v),
                }),
                None => None,
            })
            .collect()
    }

    /// Expose the internal field value store. Should be private, but is necessary for trait
    /// objects.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    fn inner(&self) -> &Vec<(u16, Value)>;

    /// Extract the field definition of a specific field number, if used.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    fn defined_message_field(&self, num: u16) -> Option<&DefinedMessageField>;

    /// Extract the value of a specific field number, if used.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    fn value(&self, num: u16) -> Option<&Value> {
        match self.inner().iter().find(|x| x.0 == num) {
            Some(x) => Some(&x.1),
            None => None,
        }
    }

    /// Writes a [`Value`] directly to the internal HashMap. Should not be used directly, rather
    /// Values should be inserted via `#process_raw_value`.
    ///
    /// [`Value`]: enum.Value.html
    /// # Example
    ///
    /// ```rust
    ///
    ///
    /// ```
    fn write_value(&mut self, num: u16, val: Value);
}
