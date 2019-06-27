use byteorder::BigEndian;
use byteorder::ByteOrder;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use line_encoding::LineEncoding;

use crate::value::{Value, ValueArray};
use std::fmt::Display;

mod line_encoding;

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OpCode {
    OpReturn = 0,
    OpConstant = 1,
    OpConstantLong = 2,
    OpNegate = 3,
    OpAdd = 4,
    OpSubtract = 5,
    OpMultiply = 6,
    OpDivide = 7,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: ValueArray,
    pub lines: LineEncoding,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::with_capacity(100),
            constants: ValueArray::new(),
            lines: LineEncoding::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.add(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.len() - 1
    }

    pub fn write_constant(&mut self, value: Value, line: u32) {
        use OpCode::*;

        let addr = self.add_constant(value);
        if addr <= u8::max_value() as usize {
            self.write(OpConstant as u8, line);
            self.write(addr as u8, line);
        } else {
            self.write(OpConstantLong as u8, line);
            for b in write_u24(addr as u32) {
                self.write(b, line)
            }
        }
    }

    pub fn get_line_number(&self, index: usize) -> u32 {
        self.lines.get(index)
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }
}

pub(crate) fn write_u24(n: u32) -> Vec<u8> {
    assert!(n <= 0xffffff);
    let mut buf = vec![0; 4];
    BigEndian::write_u32(&mut buf, n);
    buf[1..].to_vec()
}

pub(crate) fn read_u24(buf: &[u8]) -> u32 {
    assert!(buf.len() >= 3);
    let buf2 = vec![0, buf[0], buf[1], buf[2]];
    let n = BigEndian::read_u32(&buf2);
    assert!(n <= 0xffffff);
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u24() {
        let buf = vec![0x0, 0x11, 0x10, 0x4];
        assert_eq!(read_u24(&buf), 0x111004);
    }

    #[test]
    #[should_panic]
    fn test_read_u24_should_panic() {
        let buf = vec![0x1, 0x11, 0x10, 0x4];
        assert_eq!(read_u24(&buf), 0x111004);
    }

    #[test]
    fn test_write_u24() {
        assert_eq!(write_u24(30), vec![0, 0, 30]);
    }

    #[test]
    #[should_panic]
    fn test_write_u24_shoudl_panic() {
        assert_eq!(write_u24(0x1101010), vec![0, 0, 30]);
    }
}
