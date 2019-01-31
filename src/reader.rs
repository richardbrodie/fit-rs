use std::fs::File;
use std::io::{BufReader, Error, Read, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Endian {
    Big,
    Little,
}

pub struct Reader {
    pub inner: BufReader<File>,
}
impl Reader {
    pub fn new(filename: PathBuf) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        Reader { inner: reader }
    }
    pub fn byte(&mut self) -> Result<u8, Error> {
        let mut buf = [0; 1];
        self.inner.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    pub fn bytes(&mut self, num: usize) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; num];
        self.inner.read_exact(&mut buffer)?;
        Ok(buffer)
    }
    pub fn u16(&mut self, endianness: &Endian) -> Result<u16, Error> {
        let mut buf = [0; 2];
        self.inner.read_exact(&mut buf)?;
        Ok(match endianness {
            Endian::Little => u16::from_le_bytes(buf),
            Endian::Big => u16::from_be_bytes(buf),
        })
    }
    pub fn u32(&mut self, endianness: &Endian) -> Result<u32, Error> {
        let mut buf = [0; 4];
        self.inner.read_exact(&mut buf)?;
        Ok(match endianness {
            Endian::Little => u32::from_le_bytes(buf),
            Endian::Big => u32::from_be_bytes(buf),
        })
    }
    pub fn u64(&mut self, endianness: &Endian) -> Result<u64, Error> {
        let mut buf = [0; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(match endianness {
            Endian::Little => u64::from_le_bytes(buf),
            Endian::Big => u64::from_be_bytes(buf),
        })
    }
    pub fn i8(&mut self) -> Result<i8, Error> {
        self.byte().map(|b| b as i8)
    }
    pub fn i16(&mut self, endianness: &Endian) -> Result<i16, Error> {
        let mut buf = [0; 2];
        self.inner.read_exact(&mut buf)?;
        Ok(match endianness {
            Endian::Little => i16::from_le_bytes(buf),
            Endian::Big => i16::from_be_bytes(buf),
        })
    }
    pub fn i32(&mut self, endianness: &Endian) -> Result<i32, Error> {
        let mut buf = [0; 4];
        self.inner.read_exact(&mut buf)?;
        Ok(match endianness {
            Endian::Little => i32::from_le_bytes(buf),
            Endian::Big => i32::from_be_bytes(buf),
        })
    }
    pub fn i64(&mut self, endianness: &Endian) -> Result<i64, Error> {
        let mut buf = [0; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(match endianness {
            Endian::Little => i64::from_le_bytes(buf),
            Endian::Big => i64::from_be_bytes(buf),
        })
    }
    pub fn f32(&mut self, endianness: &Endian) -> Result<f32, Error> {
        let mut buf = [0; 4];
        self.inner.read_exact(&mut buf)?;
        Ok(f32::from_bits(match endianness {
            Endian::Little => u32::from_le_bytes(buf),
            Endian::Big => u32::from_be_bytes(buf),
        }))
    }
    pub fn f64(&mut self, endianness: &Endian) -> Result<f64, Error> {
        let mut buf = [0; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(f64::from_bits(match endianness {
            Endian::Little => u64::from_le_bytes(buf),
            Endian::Big => u64::from_le_bytes(buf),
        }))
    }
    pub fn skip(&mut self, index: i64) {
        match self.inner.seek(SeekFrom::Current(index)) {
            Ok(_) => (),
            Err(_) => panic!("could not skip byte"),
        }
    }
    pub fn pos(&mut self) -> Result<u64, Error> {
        self.inner.seek(SeekFrom::Current(0))
    }
}
