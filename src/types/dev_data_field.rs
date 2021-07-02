use crate::Value;

//////////
//// DevDataField
//////////

#[derive(Clone, Debug, PartialEq)]
pub struct DevDataField {
    pub data_index: u8,
    pub field_num: u8,
    pub value: Value,
}
impl DevDataField {
    pub fn new(ddi: u8, fnum: u8, v: Value) -> Self {
        Self {
            data_index: ddi,
            field_num: fnum,
            value: v,
        }
    }
}
