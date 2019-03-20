use crate::Value;

/// The name and parsed value of a message field.
pub struct Field<'a> {
    pub name: &'a str,
    pub value: Option<&'a Value>,
}
impl<'a> std::fmt::Display for Field<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.value {
            Some(val) => write!(f, "{}: {}", self.name, val),
            None => write!(f, "{}: None", self.name),
        }
    }
}
impl<'a> std::fmt::Debug for Field<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.value {
            Some(val) => write!(f, "{}: {:?}", self.name, val),
            None => write!(f, "{}: None", self.name),
        }
    }
}
