use memory::MemoryBuffer;
use isa::Opcode;
use error::MvmError;

mod isa;
mod memory;
mod error;

// Registers Indexes
// -----------------
pub const R0: u64 = 0;
pub const R1: u64 = 1;
pub const R2: u64 = 2;
pub const R3: u64 = 3;
pub const R4: u64 = 4;
pub const R5: u64 = 5;
pub const R6: u64 = 6;
pub const R7: u64 = 7;
pub const R8: u64 = 8;
pub const R_SYSTEM_CALL: u64 = 9;
pub const R_ACCUMULATOR: u64 = 10;
pub const R_INSTRUCTION_POINTER: u64 = 11;
pub const R_STACK_POINTER: u64 = 12;
pub const R_FRAME_POINTER: u64 = 13;
pub const R_MEMORY_POINTER: u64 = 14;
// -----------------

pub struct VM {
    pub memory: MemoryBuffer,

    /// Registers:
    /// R0 .. R8 - General Purpose
    /// R9 - System Call
    /// R10 - Accumulator
    /// R11 - Instruction Pointer
    /// R12 - Stack Pointer
    /// R13 - Frame Pointer
    /// R14 - Memory Pointer (next address after program)
    /// R15 - Zero Flag
    pub registers: MemoryBuffer,

    pub running: bool,
    pub exit_code: u8
}

impl VM {
    pub fn new(memsize: usize, stack_size: usize) -> Result<Self, MvmError> {
        let mut memory = MemoryBuffer::new(memsize);
        memory.set_u8(0, Opcode::Halt as u8);

        let mut vm = Self {
            memory,
            registers: MemoryBuffer::new(64),
            running: false,
            exit_code: 1,
        };

        if stack_size >= memsize {
            return Err(MvmError::OutOfBounds);
        }

        Ok(vm)
    }

    pub fn insert_program(&mut self, program: &[u8]) -> Result<(), MvmError> {
        if self.memory.get_u8(0)? != Opcode::Halt as u8 {
            return Err(MvmError::WriteEntryRejected);
        }

        if program.len() >= (self.memory.len() - self.get_register(R_STACK_POINTER)? as usize) {
            return Err(MvmError::OutOfBounds);
        }

        if program.len() < 1 { return Ok(()) };

        for (index, instruction) in program.iter().enumerate() {
            self.memory.set_u8(index as u64, *instruction)?;
        }

        let mut memptr = program.len();

        if self.memory.get_u8((program.len() - 1) as u64)? != Opcode::Halt as u8 {
            memptr += 1;
            self.memory.set_u8((program.len()) as u64, Opcode::Halt as u8)?;
        }

        self.set_register(R_MEMORY_POINTER, memptr as u64);

        Ok(())
    }
}

impl VM {
    pub fn get_register(&mut self, index: u64) -> Result<u64, MvmError> {
        self.registers.get_u64(index)
    }

    pub fn set_register(&mut self, index: u64, value: u64) -> Result<(), MvmError> {
        self.registers.set_u64(index, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_init_test() -> Result<(), MvmError> {
        let vm = VM::new(128, 16)?;

        assert!(vm.memory.len() == 128);
        assert!(vm.registers.len() == 64);
        assert!(!vm.running);

        Ok(())
    }

    #[test]
    fn vm_insert_program_test() -> Result<(), MvmError> {
        let mut vm = VM::new(128, 16)?;

        assert!(vm.memory.len() == 128);
        assert!(vm.registers.len() == 64);
        assert!(!vm.running);

        let program = [
            Opcode::Mov8 as u8,
            R0 as u8,
            5
        ];

        vm.insert_program(&program)?;

        assert_eq!(vm.memory.get_u8(0)?, program[0]);
        assert_eq!(vm.memory.get_u8(1)?, program[1]);
        assert_eq!(vm.memory.get_u8(2)?, program[2]);
        assert_eq!(vm.memory.get_u8(3)?, Opcode::Halt as u8);

        assert_eq!(vm.get_register(R_MEMORY_POINTER)?, 4);

        Ok(())
    }

    #[test]
    fn vm_insert_program_with_halt_test() -> Result<(), MvmError> {
        let mut vm = VM::new(128, 16)?;

        assert!(vm.memory.len() == 128);
        assert!(vm.registers.len() == 64);
        assert!(!vm.running);

        let program = [
            Opcode::Mov8 as u8,
            R0 as u8,
            5,
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;

        assert_eq!(vm.memory.get_u8(0)?, program[0]);
        assert_eq!(vm.memory.get_u8(1)?, program[1]);
        assert_eq!(vm.memory.get_u8(2)?, program[2]);
        assert_eq!(vm.memory.get_u8(3)?, program[3]);

        assert_eq!(vm.get_register(R_MEMORY_POINTER)?, 4);

        Ok(())
    }
}
