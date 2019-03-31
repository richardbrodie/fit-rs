mod base_type;
pub mod consts;
mod defined_message;
mod types;
mod value;

pub use base_type::BaseType;
pub use defined_message::{DefinedMessage, DefinedMessageField};
use types::message_name;
pub use types::type_value;
pub use value::Value;

include!(concat!(env!("OUT_DIR"), "/message_definitions.rs"));
include!(concat!(env!("OUT_DIR"), "/messages.rs"));

pub fn new_record(num: u16) -> Option<Box<dyn DefinedMessage>> {
    message_name(num).and_then(|name| message(name))
}
