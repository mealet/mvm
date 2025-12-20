#[derive(Debug, PartialEq)]
pub struct Label {
    pub ptr: u64,
    pub data_section: bool,
}

impl Label {
    pub fn new(ptr: u64, data_section: bool) -> Self {
        Self { ptr, data_section }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Constant {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl Constant {
    pub fn new(value: u64) -> Self {
        if u8::try_from(value).is_ok() {
            return Self::U8(value as u8);
        }

        if u16::try_from(value).is_ok() {
            return Self::U16(value as u16);
        }

        if u32::try_from(value).is_ok() {
            return Self::U32(value as u32);
        }

        Self::U64(value)
    }
}
