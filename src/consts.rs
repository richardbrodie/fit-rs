pub const COMPRESSED_HEADER_MASK: u8 = 0b1000_0000; // MASK: determine if the header has compressed timestamp
pub const COMPRESSED_HEADER_LOCAL_MESSAGE_NUMBER_MASK: u8 = 0b0110_0000; // MASK: Extract message number from a compressed header
pub const COMPRESSED_HEADER_TIME_OFFSET_MASK: u8 = 0b0001_1111; // MASK: Extract timestamp offset from a compressed header
pub const COMPRESSED_HEADER_TIME_OFFSET_ROLLOVER: u32 = 0b0010_0000; // Compressed header: rollover to eventually add when computing the new timestamp
pub const COMPRESSED_HEADER_LAST_TIMESTAMP_MASK: u32 = 0xFFFF_FFE0; // Compressed header: mask to apply to the previous timestamp before adding the time offset

pub const DEFINITION_HEADER_MASK: u8 = 0x40;
pub const DEVELOPER_FIELDS_MASK: u8 = 0x20;
pub const LOCAL_MESSAGE_NUMBER_MASK: u8 = 0x0F;

pub const _FIELD_DEFINITION_ARCHITECTURE: u8 = 0b10_000_000;
pub const FIELD_DEFINITION_BASE_NUMBER: u8 = 0b00_011_111;

pub const COORD_SEMICIRCLES_CALC: f32 = (180f64 / (std::u32::MAX as u64 / 2 + 1) as f64) as f32;
pub const PSEUDO_EPOCH: u32 = 631_065_600;
