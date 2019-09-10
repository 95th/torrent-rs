use crate::error::{Error, Result};
use std::fmt;

pub struct Reader<'a> {
    buf: &'a [u8],
    curr_idx: usize,
}

impl fmt::Debug for Reader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reader")
            .field("curr_idx", &self.curr_idx)
            .finish()
    }
}

impl<'a> Reader<'a> {
    pub fn new(buf: &[u8]) -> Reader {
        Reader { buf, curr_idx: 0 }
    }

    pub fn next_byte(&mut self) -> Option<u8> {
        let byte = self.buf.get(self.curr_idx)?;
        self.curr_idx += 1;
        Some(*byte)
    }

    pub fn move_back(&mut self) {
        debug_assert!(self.curr_idx > 0);
        self.curr_idx -= 1;
    }

    pub fn read_until(&mut self, stop_byte: u8) -> Result<&'a [u8]> {
        if self.curr_idx >= self.buf.len() {
            self.curr_idx = self.buf.len();
            return Err(Error::EOF);
        }

        let slice = &self.buf[self.curr_idx..];
        let pos = slice
            .iter()
            .position(|&b| b == stop_byte)
            .ok_or_else(|| Error::ExpectedChar(stop_byte))?;
        self.curr_idx += pos + 1; // Plus one to ignore the stop byte
        Ok(&slice[..pos])
    }

    pub fn read_int_until(&mut self, stop_byte: u8) -> Result<i64> {
        let buf = self.read_until(stop_byte)?;
        let s = std::str::from_utf8(buf).map_err(|_| Error::ParseInt)?;
        s.parse().map_err(|_| Error::ParseInt)
    }

    pub fn read_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        let end_index = self.curr_idx + len;
        if self.curr_idx >= self.buf.len() || end_index > self.buf.len() {
            self.curr_idx = self.buf.len();
            return Err(Error::EOF);
        }

        let slice = &self.buf[self.curr_idx..end_index];
        self.curr_idx = end_index;
        Ok(slice)
    }
}
