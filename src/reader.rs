use std::fs::File;
use std::io::{BufReader, Error, Read, Seek, SeekFrom, Take};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Endian {
    Big,
    Little,
}

pub struct Reader {
    buf: [u8; 8],
    pub inner: BufReader<File>,
}
impl Reader {
    pub fn new(filename: PathBuf) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        Reader {
            buf: [0; 8],
            inner: reader,
        }
    }
    pub fn byte(&mut self) -> Result<u8, Error> {
        self.inner.read_exact(&mut self.buf[..1])?;
        Ok(self.buf[0])
    }
    pub fn bytes(&mut self, num: usize) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; num];
        self.inner.read_exact(&mut buffer)?;
        Ok(buffer)
    }
    pub fn u16(&mut self, endianness: &Endian) -> Result<u16, Error> {
        self.inner.read_exact(&mut self.buf[..2])?;
        Ok(match endianness {
            Endian::Little => u16_little(&self.buf),
            Endian::Big => u16_big(&self.buf),
        })
    }
    pub fn u32(&mut self, endianness: &Endian) -> Result<u32, Error> {
        self.inner.read_exact(&mut self.buf[..4])?;
        Ok(match endianness {
            Endian::Little => u32_little(&self.buf),
            Endian::Big => u32_big(&self.buf),
        })
    }
    pub fn u64(&mut self, endianness: &Endian) -> Result<u64, Error> {
        self.inner.read_exact(&mut self.buf)?;
        Ok(match endianness {
            Endian::Little => u64_little(&self.buf),
            Endian::Big => u64_big(&self.buf),
        })
    }
    pub fn i8(&mut self) -> Result<i8, Error> {
        self.byte().map(|b| b as i8)
    }
    pub fn i16(&mut self, endianness: &Endian) -> Result<i16, Error> {
        self.inner.read_exact(&mut self.buf[..2])?;
        Ok(match endianness {
            Endian::Little => i16_little(&self.buf),
            Endian::Big => i16_big(&self.buf),
        })
    }
    pub fn i32(&mut self, endianness: &Endian) -> Result<i32, Error> {
        self.inner.read_exact(&mut self.buf[..4])?;
        Ok(match endianness {
            Endian::Little => i32_little(&self.buf),
            Endian::Big => i32_big(&self.buf),
        })
    }
    pub fn i64(&mut self, endianness: &Endian) -> Result<i64, Error> {
        self.inner.read_exact(&mut self.buf)?;
        Ok(match endianness {
            Endian::Little => i64_little(&self.buf),
            Endian::Big => i64_big(&self.buf),
        })
    }
    pub fn f32(&mut self, endianness: &Endian) -> Result<f32, Error> {
        self.inner.read_exact(&mut self.buf[..4])?;
        Ok(match endianness {
            Endian::Little => f32_little(&self.buf),
            Endian::Big => f32_big(&self.buf),
        })
    }
    pub fn f64(&mut self, endianness: &Endian) -> Result<f64, Error> {
        self.inner.read_exact(&mut self.buf)?;
        Ok(match endianness {
            Endian::Little => f64_little(&self.buf),
            Endian::Big => f64_big(&self.buf),
        })
    }
    pub fn skip(&mut self, index: i64) {
        self.inner.seek(SeekFrom::Current(index));
    }
    pub fn pos(&mut self) -> Result<u64, Error> {
        self.inner.seek(SeekFrom::Current(0))
    }
}

fn u16_little(buf: &[u8]) -> u16 {
    buf[0] as u16 | (buf[1] as u16) << 8
}
fn u16_big(buf: &[u8]) -> u16 {
    buf[1] as u16 | (buf[0] as u16) << 8
}

fn u32_little(buf: &[u8]) -> u32 {
    let (left, right) = buf.split_at(2);
    u16_little(left) as u32 | (u16_little(right) as u32) << 16
}
fn u32_big(buf: &[u8]) -> u32 {
    let (left, right) = buf.split_at(2);
    u16_big(right) as u32 | (u16_big(left) as u32) << 16
}

fn u64_little(buf: &[u8]) -> u64 {
    let (left, right) = buf.split_at(4);
    u32_little(left) as u64 | (u32_little(right) as u64) << 32
}
fn u64_big(buf: &[u8]) -> u64 {
    let (left, right) = buf.split_at(4);
    u32_big(right) as u64 | (u32_big(left) as u64) << 32
}

fn i16_little(buf: &[u8]) -> i16 {
    buf[0] as i16 | (buf[1] as i16) << 8
}
fn i16_big(buf: &[u8]) -> i16 {
    buf[1] as i16 | (buf[0] as i16) << 8
}

fn i32_little(buf: &[u8]) -> i32 {
    let (left, right) = buf.split_at(2);
    i16_little(left) as i32 | (i16_little(right) as i32) << 16
}
fn i32_big(buf: &[u8]) -> i32 {
    let (left, right) = buf.split_at(2);
    i16_big(right) as i32 | (i16_big(left) as i32) << 16
}

fn i64_little(buf: &[u8]) -> i64 {
    let (left, right) = buf.split_at(4);
    i32_little(left) as i64 | (i32_little(right) as i64) << 32
}
fn i64_big(buf: &[u8]) -> i64 {
    let (left, right) = buf.split_at(4);
    i32_big(right) as i64 | (i32_big(left) as i64) << 32
}
fn f32_little(buf: &[u8]) -> f32 {
    f32::from_bits(u32_little(buf))
}
fn f32_big(buf: &[u8]) -> f32 {
    f32::from_bits(u32_little(buf))
}
fn f64_little(buf: &[u8]) -> f64 {
    f64::from_bits(u64_little(buf))
}
fn f64_big(buf: &[u8]) -> f64 {
    f64::from_bits(u64_big(buf))
}
