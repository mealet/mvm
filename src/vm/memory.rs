use super::error::MvmError;

pub struct MemoryBuffer {
    pub inner: Vec<u8>
}

impl MemoryBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            inner: vec![0; size]
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl MemoryBuffer {
    // ----| u8 |----

    pub fn get_u8(&self, address: u64) -> Result<u8, MvmError> {
        const BYTES_LENGTH: u64 = 1;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        Ok(self.inner[address as usize])
    }

    pub fn set_u8(&mut self, address: u64, value: u8) -> Result<(), MvmError> {
        const BYTES_LENGTH: u64 = 1;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        self.inner[address as usize] = value;

        Ok(())
    }

    // ----| u16 |----

    pub fn get_u16(&self, address: u64) -> Result<u16, MvmError> {
        const BYTES_LENGTH: u64 = 2;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        Ok(
            u16::from_le_bytes([
                self.inner[address as usize],
                self.inner[(address + 1) as usize]
            ])
        )
    }

    pub fn set_u16(&mut self, address: u64, value: u16) -> Result<(), MvmError> {
        const BYTES_LENGTH: u64 = 1;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        let bytes = value.to_le_bytes();

        (
            self.inner[address as usize],
            self.inner[(address + 1) as usize]
        ) = (
            bytes[0],
            bytes[1]
        );

        Ok(())
    }

    // ----| u32 |----

    pub fn get_u32(&self, address: u64) -> Result<u32, MvmError> {
        const BYTES_LENGTH: u64 = 4;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        Ok(
            u32::from_le_bytes([
                self.inner[address as usize],
                self.inner[(address + 1) as usize],
                self.inner[(address + 2) as usize],
                self.inner[(address + 3) as usize],
            ])
        )
    }

    pub fn set_u32(&mut self, address: u64, value: u32) -> Result<(), MvmError> {
        const BYTES_LENGTH: u64 = 1;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        let bytes = value.to_le_bytes();

        (
            self.inner[address as usize],
            self.inner[(address + 1) as usize],
            self.inner[(address + 2) as usize],
            self.inner[(address + 3) as usize],
        ) = (
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3]
        );

        Ok(())
    }

    // ----| u64 |----

    pub fn get_u64(&self, address: u64) -> Result<u64, MvmError> {
        const BYTES_LENGTH: u64 = 8;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        Ok(
            u64::from_le_bytes([
                self.inner[address as usize],
                self.inner[(address + 1) as usize],
                self.inner[(address + 2) as usize],
                self.inner[(address + 3) as usize],
                self.inner[(address + 4) as usize],
                self.inner[(address + 5) as usize],
                self.inner[(address + 6) as usize],
                self.inner[(address + 7) as usize],
            ])
        )
    }

    pub fn set_u64(&mut self, address: u64, value: u64) -> Result<(), MvmError> {
        const BYTES_LENGTH: u64 = 1;

        if (address + BYTES_LENGTH - 1) as usize > self.len() - 1 {
            return Err(MvmError::SegmentationFault(address));
        }

        let bytes = value.to_le_bytes();

        (
            self.inner[address as usize],
            self.inner[(address + 1) as usize],
            self.inner[(address + 2) as usize],
            self.inner[(address + 3) as usize],
            self.inner[(address + 4) as usize],
            self.inner[(address + 5) as usize],
            self.inner[(address + 6) as usize],
            self.inner[(address + 7) as usize],
        ) = (
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_management_test() -> Result<(), MvmError> {
        let mut memory = MemoryBuffer::new(8);

        memory.set_u8(0, 8)?;
        assert_eq!(memory.get_u8(0)?, 8);

        memory.set_u16(0, 123)?;
        assert_eq!(memory.get_u16(0)?, 123);

        memory.set_u32(0, 9999)?;
        assert_eq!(memory.get_u32(0)?, 9999);

        memory.set_u64(0, 1234567890)?;
        assert_eq!(memory.get_u64(0)?, 1234567890);
        
        Ok(())
    }
}
