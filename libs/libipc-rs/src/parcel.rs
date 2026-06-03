use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct Parcel {
    data: Vec<u8>,
    read_pos: usize,
}

impl Parcel {
    /// Creates a new empty `Parcel` with default capacity.
    pub fn new() -> Self {
        Parcel {
            data: Vec::with_capacity(64),
            read_pos: 0,
        }
    }

    /// Creates a `Parcel` from a raw vector of bytes.
    pub fn from_vec(data: Vec<u8>) -> Self {
        Parcel {
            data,
            read_pos: 0,
        }
    }

    /// Returns a reference to the inner raw byte buffer.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the length of the written data in the parcel.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns the current read cursor position.
    pub fn read_pos(&self) -> usize {
        self.read_pos
    }

    /// Resets the read cursor back to the beginning.
    pub fn reset_read(&mut self) {
        self.read_pos = 0;
    }

    /// Clears all data and resets both size and read position.
    pub fn reset_write(&mut self) {
        self.data.clear();
        self.read_pos = 0;
    }

    /// Writes raw bytes to the parcel.
    pub fn write_raw(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Writes an `i32` to the parcel in native-endian format.
    pub fn write_i32(&mut self, val: i32) {
        self.write_raw(&val.to_ne_bytes());
    }

    /// Writes a `u32` to the parcel in native-endian format.
    pub fn write_u32(&mut self, val: u32) {
        self.write_raw(&val.to_ne_bytes());
    }

    /// Writes a string to the parcel. Matches C's wire protocol representation
    /// (length as a `u32` including null-terminator, followed by bytes, followed by null-terminator).
    pub fn write_string(&mut self, val: &str) {
        let len = (val.len() + 1) as u32;
        self.write_u32(len);
        self.write_raw(val.as_bytes());
        self.write_raw(&[0]); // Null terminator
    }

    /// Reads raw bytes of specified length from the parcel.
    pub fn read_raw(&mut self, len: usize) -> Result<&[u8], &'static str> {
        if self.read_pos + len > self.data.len() {
            return Err("Out of bounds read");
        }
        let start = self.read_pos;
        self.read_pos += len;
        Ok(&self.data[start..self.read_pos])
    }

    /// Reads an `i32` from the parcel.
    pub fn read_i32(&mut self) -> Result<i32, &'static str> {
        let bytes = self.read_raw(4)?;
        let array: [u8; 4] = bytes.try_into().map_err(|_| "Failed to parse i32")?;
        Ok(i32::from_ne_bytes(array))
    }

    /// Reads a `u32` from the parcel.
    pub fn read_u32(&mut self) -> Result<u32, &'static str> {
        let bytes = self.read_raw(4)?;
        let array: [u8; 4] = bytes.try_into().map_err(|_| "Failed to parse u32")?;
        Ok(u32::from_ne_bytes(array))
    }

    /// Reads a string from the parcel.
    pub fn read_string(&mut self) -> Result<String, &'static str> {
        let len = self.read_u32()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        
        let bytes = self.read_raw(len)?;
        if bytes.last() != Some(&0) {
            return Err("String is not null-terminated");
        }
        
        // Exclude the null terminator from the final Rust String
        let str_bytes = &bytes[0..len - 1];
        String::from_utf8(str_bytes.to_vec()).map_err(|_| "Invalid UTF-8 sequence")
    }
}

impl Default for Parcel {
    fn default() -> Self {
        Self::new()
    }
}
