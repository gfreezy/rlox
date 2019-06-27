use byteorder::ByteOrder;
use byteorder::LittleEndian;
use byteorder::{ReadBytesExt, WriteBytesExt};

pub struct LineEncoding {
    buf: Vec<u8>,
}

impl LineEncoding {
    pub fn new() -> Self {
        LineEncoding {
            buf: Vec::with_capacity(100),
        }
    }

    pub fn add(&mut self, line_number: u32) {
        let len = self.buf.len();
        if len > 0 {
            let count_index = len - 1;
            let line_number_index = count_index - 4;
            let count = self.buf[count_index];
            let last_line_number =
                LittleEndian::read_u32(&self.buf[line_number_index..count_index]);
            if line_number == last_line_number && count < std::u8::MAX {
                self.buf[count_index] += 1;
                return;
            }
        }
        self.buf
            .write_u32::<LittleEndian>(line_number)
            .expect("write line number");
        self.buf.write_u8(1).expect("write count");
    }

    pub fn get(&self, index: usize) -> u32 {
        let mut buf = self.buf.as_slice();

        let mut i = 0;
        let mut line_number;
        loop {
            line_number = buf.read_u32::<LittleEndian>().expect("read line number");
            let count = buf.read_u8().expect("read count");

            i += count as usize;

            if i >= index + 1 {
                break;
            }
        }
        return line_number;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut encoding = LineEncoding::new();
        encoding.add(1);
        encoding.add(10);
        encoding.add(10);
        encoding.add(10);
        encoding.add(12);
        encoding.add(12);
        encoding.add(15);

        assert_eq!(
            encoding.buf,
            vec![1, 0, 0, 0, 1, 10, 0, 0, 0, 3, 12, 0, 0, 0, 2, 15, 0, 0, 0, 1]
        );
    }

    #[test]
    fn test_get() {
        let mut encoding = LineEncoding::new();
        encoding.buf = vec![
            1, 0, 0, 0, 1, 10, 0, 0, 0, 3, 12, 0, 0, 0, 2, 15, 0, 0, 0, 1,
        ];
        assert_eq!(encoding.get(0), 1);
        assert_eq!(encoding.get(4), 12);
        assert_eq!(encoding.get(5), 12);

        for i in 1..=3 {
            assert_eq!(encoding.get(i), 10);
        }
        assert_eq!(encoding.get(6), 15);
    }
}
