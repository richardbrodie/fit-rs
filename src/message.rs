use super::data_record;

#[derive(Debug)]
pub struct Message {
    pub msg_type: MessageType,
    pub msg_num: u8,
    pub values: Vec<MessageValue>,
}

#[derive(Debug)]
pub enum MessageType {
    FileId,
    Capabilities,
    DeviceSettings,
}

#[derive(Debug)]
pub struct MessageValue {
    name: String,
    id: u8,
    value: data_record::Value,
}
