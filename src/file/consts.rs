pub struct BaseTypeStruct {
    _id: u8,
    _typefield: u8,
    pub invalidvalue: u64,
    pub byte_size: u8,
}

pub const ENUM_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 0,
    _typefield: 0x00,
    invalidvalue: 0xFF,
    byte_size: 1,
};
pub const SINT8_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 1,
    _typefield: 0x01,
    invalidvalue: 0x7F,
    byte_size: 1,
};
pub const UINT8_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 2,
    _typefield: 0x02,
    invalidvalue: 0xFF,
    byte_size: 1,
};
pub const SINT16_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 3,
    _typefield: 0x83,
    invalidvalue: 0x7FFF,
    byte_size: 2,
};
pub const UINT16_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 4,
    _typefield: 0x84,
    invalidvalue: 0xFFFF,
    byte_size: 2,
};
pub const SINT32_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 5,
    _typefield: 0x85,
    invalidvalue: 0x7FFFFFFF,
    byte_size: 4,
};
pub const UINT32_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 6,
    _typefield: 0x86,
    invalidvalue: 0xFFFFFFFF,
    byte_size: 4,
};
pub const STRING_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 7,
    _typefield: 0x07,
    invalidvalue: 0x00,
    byte_size: 1,
};
pub const FLOAT32_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 8,
    _typefield: 0x88,
    invalidvalue: 0xFFFFFFFF,
    byte_size: 4,
};
pub const FLOAT64_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 9,
    _typefield: 0x89,
    invalidvalue: 0xFFFFFFFFFFFFFFFF,
    byte_size: 8,
};
pub const UINT8Z_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 10,
    _typefield: 0x0A,
    invalidvalue: 0x00,
    byte_size: 1,
};
pub const UINT16Z_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 11,
    _typefield: 0x8B,
    invalidvalue: 0x0000,
    byte_size: 2,
};
pub const UINT32Z_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 12,
    _typefield: 0x8C,
    invalidvalue: 0x00000000,
    byte_size: 4,
};
pub const BYTE_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 13,
    _typefield: 0x0D,
    invalidvalue: 0xFF,
    byte_size: 1,
};
pub const SINT64_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 14,
    _typefield: 0x8E,
    invalidvalue: 0x7FFFFFFFFFFFFFFF,
    byte_size: 8,
};
pub const UINT64_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 15,
    _typefield: 0x8F,
    invalidvalue: 0xFFFFFFFFFFFFFFFF,
    byte_size: 8,
};
pub const UINT64Z_TYPE: BaseTypeStruct = BaseTypeStruct {
    _id: 16,
    _typefield: 0x90,
    invalidvalue: 0x0000000000000000,
    byte_size: 8,
};
