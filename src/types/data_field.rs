use crate::Value;

//////////
//// DataField
//////////

#[derive(Clone, Debug, PartialEq)]
pub struct DataField {
    pub field_num: usize,
    pub value: Value,
}
impl DataField {
    pub fn new(fnum: usize, v: Value) -> Self {
        Self {
            field_num: fnum,
            value: v,
        }
    }
}
