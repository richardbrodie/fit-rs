#[derive(Debug)]
pub enum BaseType {
    ENUM,
    SINT8,
    UINT8,
    SINT16,
    UINT16,
    SINT32,
    UINT32,
    STRING,
    FLOAT32,
    FLOAT64,
    UINT8Z,
    UINT16Z,
    UINT32Z,
    BYTE,
    SINT64,
    UINT64,
    UINT64Z,
}
impl BaseType {
    pub fn get(num: u8) -> Self {
        match num {
            0 => BaseType::ENUM,
            1 => BaseType::SINT8,
            2 => BaseType::UINT8,
            3 => BaseType::SINT16,
            4 => BaseType::UINT16,
            5 => BaseType::SINT32,
            6 => BaseType::UINT32,
            7 => BaseType::STRING,
            8 => BaseType::FLOAT32,
            9 => BaseType::FLOAT64,
            10 => BaseType::UINT8Z,
            11 => BaseType::UINT16Z,
            12 => BaseType::UINT32Z,
            13 => BaseType::BYTE,
            14 => BaseType::SINT64,
            15 => BaseType::UINT64,
            16 => BaseType::UINT64Z,
            _ => panic!("not an option"),
        }
    }
}
