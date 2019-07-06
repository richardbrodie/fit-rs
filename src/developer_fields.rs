use super::{DataField, Value};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct DeveloperFieldDefinition {
    pub field_number: u8,
    pub size: u8,
    pub developer_data_index: u8,
}
impl DeveloperFieldDefinition {
    pub fn new(map: &mut &[u8]) -> Self {
        let (buf, rest) = map.split_at(3);
        *map = rest;
        Self {
            field_number: buf[0],
            size: buf[1],
            developer_data_index: buf[2],
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeveloperFieldDescription {
    pub developer_data_index: u8,
    pub field_definition_number: u8,
    pub fit_base_type: u8,
    pub field_name: String,
    pub units: String,
}
impl DeveloperFieldDescription {
    pub fn new(values: Vec<DataField>, global_string_map: &mut HashMap<u8, String>) -> Self {
        let mut hmap: HashMap<usize, Value> = HashMap::with_capacity(6);
        values.into_iter().for_each(|v| {
            hmap.insert(v.field_num, v.value);
        });
        Self {
            developer_data_index: hmap.remove(&0).unwrap().into(),
            field_definition_number: hmap.remove(&1).unwrap().into(),
            fit_base_type: match hmap.remove(&2).unwrap() {
                Value::U8(s) => s,
                _ => panic!("can't call this on a non-u8 variant"),
            },
            field_name: match hmap.remove(&3).unwrap() {
                Value::String(ref v) => global_string_map.remove(v).unwrap(),
                Value::Enum(s) => s.to_owned(),
                _ => panic!("can't call this on a non-string/enum variant"),
            },
            units: match hmap.remove(&8).unwrap() {
                Value::String(ref v) => global_string_map.remove(v).unwrap(),
                Value::Enum(s) => s.to_owned(),
                _ => panic!("can't call this on a non-string/enum variant"),
            },
        }
    }
}
