use fitsdk::MessageType;

use super::{data_field::DataField, dev_data_field::DevDataField};

//////////
//// Message
//////////

#[derive(Clone, Debug)]
pub struct Message {
    pub kind: MessageType,
    pub values: Vec<DataField>,
    pub dev_values: Option<Vec<DevDataField>>,
}
