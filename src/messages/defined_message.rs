use super::{DefinedMessageField, Field};
use crate::value::Value;
use crate::DataField;

/// A trait representing all the different message types as defined in the FIT SDK.
pub trait DefinedMessage {
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
    fn field_name_and_value(&self, num: u16) -> Option<Field> {
        self.defined_message_field(num).map(|f| Field {
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
    fn all_values(&self) -> Vec<Field> {
        self.inner()
            .iter()
            .filter_map(|(k, v)| match self.defined_message_field(*k) {
                Some(f) => Some(Field {
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
