use super::convert_value;
use crate::value::Value;

/// A collection of information about a specific message field, as defined in the FIT SDK.
#[derive(Debug)]
pub struct DefinedMessageField {
    pub num: u16,
    pub name: &'static str,
    pub kind: &'static str,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
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

    fn process_raw_value(&mut self, num: u16, val: &Value) {
        if let Some(field) = self.defined_message_field(num) {
            if let Some(v) = convert_value(val, field) {
                self.write_value(num, v)
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
            .map(|(k, v)| FieldNameAndValue {
                name: self.defined_message_field(*k).unwrap().name,
                value: Some(v),
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
