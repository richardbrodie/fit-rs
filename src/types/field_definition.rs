use std::io::Read;

use crate::consts::FIELD_DEFINITION_BASE_NUMBER;

//////////
//// FieldDefinition
//////////

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FieldDefinition {
    pub definition_number: usize,
    pub size: u8,
    pub base_type: u8,
}
impl FieldDefinition {
    pub fn new<R>(map: &mut R) -> Self
    where
        R: Read,
    {
        let mut buf: [u8; 3] = [0; 3];
        let _ = map.read(&mut buf);
        println!("{:?}", buf);
        Self {
            definition_number: buf[0].into(),
            size: buf[1],
            base_type: buf[2] & FIELD_DEFINITION_BASE_NUMBER,
        }
    }
}
