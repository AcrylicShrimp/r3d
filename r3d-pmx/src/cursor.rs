use crate::parse::ParseError;

pub struct Cursor<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn has_bytes(&self, len: usize) -> bool {
        self.position + len <= self.buffer.len()
    }

    pub fn ensure_bytes<E: ParseError>(&self, len: usize) -> Result<(), E> {
        if !self.has_bytes(len) {
            return Err(E::error_unexpected_eof());
        }

        Ok(())
    }

    pub fn read<E: ParseError, const L: usize>(&mut self) -> Result<&[u8; L], E> {
        let result = &self.buffer[self.position..self.position + L];
        self.position += L;
        Ok(unsafe { &*(result as *const [u8] as *const [u8; L]) })
    }

    pub fn read_dynamic<E: ParseError>(&mut self, len: usize) -> Result<&[u8], E> {
        let result = &self.buffer[self.position..self.position + len];
        self.position += len;
        Ok(result)
    }
}
