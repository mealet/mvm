#[derive(Debug, PartialEq)]
pub struct Label {
    ptr: u64,
}

impl Label {
    pub fn new(ptr: u64) -> Self {
        Self {
            ptr
        }
    }
}
