use crate::Endianness;

use std::convert::TryInto;

pub fn u8(map: &mut &[u8]) -> u8 {
    let (val, rest) = map.split_first().unwrap();
    *map = rest;
    *val
}
pub fn i8(map: &mut &[u8]) -> i8 {
    u8(map) as i8
}
pub fn u16(map: &mut &[u8], endianness: Endianness) -> u16 {
    let arr = arr2(map);
    if endianness == Endianness::Little {
        u16::from_le_bytes(arr)
    } else {
        u16::from_be_bytes(arr)
    }
}
pub fn i16(map: &mut &[u8], endianness: Endianness) -> i16 {
    let arr = arr2(map);
    if endianness == Endianness::Little {
        i16::from_le_bytes(arr)
    } else {
        i16::from_be_bytes(arr)
    }
}
pub fn u32(map: &mut &[u8], endianness: Endianness) -> u32 {
    let arr = arr4(map);
    if endianness == Endianness::Little {
        u32::from_le_bytes(arr)
    } else {
        u32::from_be_bytes(arr)
    }
}
pub fn i32(map: &mut &[u8], endianness: Endianness) -> i32 {
    let arr = arr4(map);
    if endianness == Endianness::Little {
        i32::from_le_bytes(arr)
    } else {
        i32::from_be_bytes(arr)
    }
}
pub fn u64(map: &mut &[u8], endianness: Endianness) -> u64 {
    let arr = arr8(map);
    if endianness == Endianness::Little {
        u64::from_le_bytes(arr)
    } else {
        u64::from_be_bytes(arr)
    }
}
pub fn i64(map: &mut &[u8], endianness: Endianness) -> i64 {
    let arr = arr8(map);
    if endianness == Endianness::Little {
        i64::from_le_bytes(arr)
    } else {
        i64::from_be_bytes(arr)
    }
}

fn arr2(map: &mut &[u8]) -> [u8; 2] {
    let (buf, rest) = map.split_at(2);
    *map = rest;
    buf.try_into().unwrap()
}

pub fn arr4(map: &mut &[u8]) -> [u8; 4] {
    let (buf, rest) = map.split_at(4);
    *map = rest;
    buf.try_into().unwrap()
}

fn arr8(map: &mut &[u8]) -> [u8; 8] {
    let (buf, rest) = map.split_at(8);
    *map = rest;
    buf.try_into().unwrap()
}

#[inline(always)]
pub fn skip_bytes(map: &mut &[u8], s: u8) {
    let (_, rest) = map.split_at(s as usize);
    *map = rest;
}
