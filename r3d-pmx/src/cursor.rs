use crate::parse::ParseError;

pub trait Cursor {
    fn checked<'a>(&'a mut self) -> CheckedCursor<'a>;
    fn read<E: ParseError, const L: usize>(&mut self) -> Result<&[u8; L], E>;
    fn read_dynamic<E: ParseError>(&mut self, len: usize) -> Result<&[u8], E>;
}

/// A cursor that does not check the length of the buffer for every read operation.
pub struct UncheckedCursor {
    buffer: Vec<u8>,
    position: usize,
}

impl UncheckedCursor {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }
}

impl Cursor for UncheckedCursor {
    fn checked<'a>(&'a mut self) -> CheckedCursor<'a> {
        CheckedCursor::new(self)
    }

    fn read<E: ParseError, const L: usize>(&mut self) -> Result<&[u8; L], E> {
        let result = &self.buffer[self.position..self.position + L];
        self.position += L;
        Ok(unsafe { &*(result as *const [u8] as *const [u8; L]) })
    }

    fn read_dynamic<E: ParseError>(&mut self, len: usize) -> Result<&[u8], E> {
        let result = &self.buffer[self.position..self.position + len];
        self.position += len;
        Ok(result)
    }
}

/// A cursor that checks the length of the buffer for every read operation.
pub struct CheckedCursor<'a> {
    cursor: &'a mut UncheckedCursor,
}

impl<'a> CheckedCursor<'a> {
    pub fn new(cursor: &'a mut UncheckedCursor) -> Self {
        Self { cursor }
    }

    pub fn has_bytes(&self, len: usize) -> bool {
        self.cursor.position + len <= self.cursor.buffer.len()
    }

    pub fn ensure_bytes<E: ParseError>(&self, len: usize) -> Result<(), E> {
        if !self.has_bytes(len) {
            return Err(E::error_unexpected_eof());
        }

        Ok(())
    }
}

impl<'a> Cursor for CheckedCursor<'a> {
    fn checked<'aa>(&'aa mut self) -> CheckedCursor<'aa> {
        CheckedCursor::new(self.cursor)
    }

    fn read<E: ParseError, const L: usize>(&mut self) -> Result<&[u8; L], E> {
        self.ensure_bytes(L)?;

        let result = self.cursor.read::<E, L>()?;
        Ok(result)
    }

    fn read_dynamic<E: ParseError>(&mut self, len: usize) -> Result<&[u8], E> {
        self.ensure_bytes(len)?;

        let result = self.cursor.read_dynamic::<E>(len)?;
        Ok(result)
    }
}
