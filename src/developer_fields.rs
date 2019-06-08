use super::io::u8;

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

pub struct DeveloperDataIdMsg {
    application_id: [u8; 16],
    developer_data_index: u8,
}
impl DeveloperDataIdMsg {
    fn new(map: &mut &[u8]) -> Self {
        let (buf, rest) = map.split_at(16);
        *map = rest;
        let mut a: [u8; 16] = Default::default();
        a.copy_from_slice(buf);
        Self {
            application_id: a,
            developer_data_index: u8(map),
        }
    }
}

pub struct FieldDescriptionMsg {
    developer_data_index: u8,
    field_definition_number: u8,
    fit_base_type_id: u8,
    field_name: String,
    units: String,
    native_field_num: u8,
}
impl FieldDescriptionMsg {
    fn new(map: &mut &[u8]) -> Self {
        let (buf, mut rest) = map.split_at(3);
        let (field_name_buf, mut rest) = rest.split_at(64);
        let (units_buf, mut rest) = rest.split_at(16);
        *map = rest;

        Self {
            developer_data_index: buf[0],
            field_definition_number: buf[1],
            fit_base_type_id: buf[2],
            field_name: std::str::from_utf8(field_name_buf).unwrap().to_string(),
            units: std::str::from_utf8(units_buf).unwrap().to_string(),
            native_field_num: u8(map),
        }
    }
}
