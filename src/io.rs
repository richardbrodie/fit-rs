use std::io::{Read, Seek, SeekFrom};

#[derive(Copy, Clone,Debug, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

pub fn read_u8<R>(map: &mut R) -> u8
where
    R: Read,
{
    let mut buf: [u8; 1] = [0];
    let _ = map.read(&mut buf);
    buf[0]
}
pub fn read_i8<R>(map: &mut R) -> i8
where
    R: Read,
{
    read_u8(map) as i8
}
pub fn read_u16<R>(map: &mut R, endianness: Endianness) -> u16
where
    R: Read,
{
    let arr = arr2(map);
    if endianness == Endianness::Little {
        u16::from_le_bytes(arr)
    } else {
        u16::from_be_bytes(arr)
    }
}
pub fn read_i16<R>(map: &mut R, endianness: Endianness) -> i16
where
    R: Read,
{
    let arr = arr2(map);
    if endianness == Endianness::Little {
        i16::from_le_bytes(arr)
    } else {
        i16::from_be_bytes(arr)
    }
}
pub fn read_u32<R>(map: &mut R, endianness: Endianness) -> u32
where
    R: Read,
{
    let arr = arr4(map);
    if endianness == Endianness::Little {
        u32::from_le_bytes(arr)
    } else {
        u32::from_be_bytes(arr)
    }
}
pub fn read_i32<R>(map: &mut R, endianness: Endianness) -> i32
where
    R: Read,
{
    let arr = arr4(map);
    if endianness == Endianness::Little {
        i32::from_le_bytes(arr)
    } else {
        i32::from_be_bytes(arr)
    }
}
pub fn read_u64<R>(map: &mut R, endianness: Endianness) -> u64
where
    R: Read,
{
    let arr = arr8(map);
    if endianness == Endianness::Little {
        u64::from_le_bytes(arr)
    } else {
        u64::from_be_bytes(arr)
    }
}
pub fn read_i64<R>(map: &mut R, endianness: Endianness) -> i64
where
    R: Read,
{
    let arr = arr8(map);
    if endianness == Endianness::Little {
        i64::from_le_bytes(arr)
    } else {
        i64::from_be_bytes(arr)
    }
}

fn arr2<R>(map: &mut R) -> [u8; 2]
where
    R: Read,
{
    let mut buf: [u8; 2] = [0; 2];
    let _ = map.read(&mut buf);
    buf
}

pub fn arr4<R>(map: &mut R) -> [u8; 4]
where
    R: Read,
{
    let mut buf: [u8; 4] = [0; 4];
    let _ = map.read(&mut buf);
    buf
}

fn arr8<R>(map: &mut R) -> [u8; 8]
where
    R: Read,
{
    let mut buf: [u8; 8] = [0; 8];
    let _ = map.read(&mut buf);
    buf
}

pub fn skip_bytes<R>(map: &mut R, s: u8)
where
    R: Seek,
{
    map.seek(SeekFrom::Current(s.into())).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_reads_u8() {
        let a = [42];
        assert_eq!(read_u8(&mut a.as_ref()), 42);
    }
    #[test]
    fn it_reads_u16_le() {
        let endianness = Endianness::Little;
        let a = [42, 151];
        assert_eq!(read_u16(&mut a.as_ref(), endianness), 38698);
    }
    #[test]
    fn it_reads_u16_be() {
        let endianness = Endianness::Big;
        let a = [42, 151];
        assert_eq!(read_u16(&mut a.as_ref(), endianness), 10903);
    }
    #[test]
    fn it_reads_u32_le() {
        let endianness = Endianness::Little;
        let a = [42, 151, 138, 217];
        assert_eq!(read_u32(&mut a.as_ref(), endianness), 3649738538);
    }
    #[test]
    fn it_reads_u32_be() {
        let endianness = Endianness::Big;
        let a = [42, 151, 138, 217];
        assert_eq!(read_u32(&mut a.as_ref(), endianness), 714574553);
    }
    #[test]
    fn it_reads_u64_le() {
        let endianness = Endianness::Little;
        let a = [42, 151, 138, 217, 59, 205, 235, 102];
        assert_eq!(read_u64(&mut a.as_ref(), endianness), 7416246868332156714);
    }
    #[test]
    fn it_reads_u64_be() {
        let endianness = Endianness::Big;
        let a = [42, 151, 138, 217, 59, 205, 235, 102];
        assert_eq!(read_u64(&mut a.as_ref(), endianness), 3069074336692169574);
    }
    #[test]
    fn it_reads_i8() {
        let a = [234];
        assert_eq!(read_i8(&mut a.as_ref()), -22);
    }
    #[test]
    fn it_reads_i16_le() {
        let endianness = Endianness::Little;
        let a = [234, 151];
        assert_eq!(read_i16(&mut a.as_ref(), endianness), -26646);
    }
    #[test]
    fn it_reads_i16_be() {
        let endianness = Endianness::Big;
        let a = [234, 151];
        assert_eq!(read_i16(&mut a.as_ref(), endianness), -5481);
    }
    #[test]
    fn it_reads_i32_le() {
        let endianness = Endianness::Little;
        let a = [234, 151, 138, 217];
        assert_eq!(read_i32(&mut a.as_ref(), endianness), -645228566);
    }
    #[test]
    fn it_reads_i32_be() {
        let endianness = Endianness::Big;
        let a = [234, 151, 138, 217];
        assert_eq!(read_i32(&mut a.as_ref(), endianness), -359167271);
    }
    #[test]
    fn it_reads_i64_le() {
        let endianness = Endianness::Little;
        let a = [234, 151, 138, 217, 59, 205, 235, 102];
        assert_eq!(read_i64(&mut a.as_ref(), endianness), 7416246868332156906);
    }
    #[test]
    fn it_reads_i64_be() {
        let endianness = Endianness::Big;
        let a = [234, 151, 138, 217, 59, 205, 235, 102];
        assert_eq!(read_i64(&mut a.as_ref(), endianness), -1542611681735218330);
    }
    #[test]
    fn it_consumes_bytes() {
        let endianness = Endianness::Little;
        let a = [234, 151, 138, 217, 59];
        let mut c = Cursor::new(a);
        let _ = read_u32(&mut c, endianness);
        assert_eq!(read_u8(&mut c), 59);
    }
    #[test]
    fn it_skips_bytes() {
        let a = [234, 151, 138, 217, 59, 205, 235, 102];
        let mut c = Cursor::new(a);
        skip_bytes(&mut c, 5);
        assert_eq!(read_u8(&mut c), 205);
    }
}
