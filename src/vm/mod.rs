mod isa;
mod memory;
mod error;

pub struct VM {
    registers: [u64; 16],
}
