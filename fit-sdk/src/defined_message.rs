/// A trait representing all the different message types as defined in the FIT SDK.
pub trait DefinedMessage {
    fn new() -> Self
    where
        Self: Sized;

    /// The name of the underlying message, as defined in the SDK.
    ///
    /// For example, "Record", "Session", "Device Settings", etc
    fn name(&self) -> &str;

    fn defined_message_field(&self, num: u16) -> Option<&DefinedMessageField>;

    fn size(&self) -> usize;
}

/// A collection of information about a specific message field, as defined in the FIT SDK.
#[derive(Debug)]
pub struct DefinedMessageField {
    pub num: u16,
    pub name: &'static str,
    pub kind: &'static str,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
}
