use super::{
    VM, Opcode, MvmError,
    R0, R1, R2, R3, R4, R5, R6, R7, R8,
    R_SYSTEM_CALL, R_ACCUMULATOR, R_INSTRUCTION_POINTER,
    R_STACK_POINTER, R_FRAME_POINTER, R_MEMORY_POINTER
};

impl VM {
    pub fn execute_instruction(&mut self, instruction: u8) -> Result<(), MvmError> {
        let opcode = Opcode::try_from(instruction)?;

        match opcode {
            Opcode::Halt => {
                self.running = false;
                let _ = self.step_back()?;
            },
            Opcode::Return => todo!(),
            Opcode::Call => todo!(),
            Opcode::Interrupt => todo!(),

            Opcode::DataSection => {
                while let Ok(instr) = self.fetch_u8() {
                    if instr == 0xff &&
                    let Ok(next_instr) = self.fetch_u8()
                    && next_instr == Opcode::TextSection as u8 {
                        let _ = self.step_back()?;
                        return Ok(())
                    }
                }

                if self.peek_byte().is_err() {
                    return Err(MvmError::NoTextSection)
                }
            },
            Opcode::TextSection => {},

            Opcode::Mov8 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let value = self.memory.get_u8(address)?;
                self.set_register(destination as u64, value as u64)?;
            },
            Opcode::Mov16 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let value = self.memory.get_u16(address)?;
                self.set_register(destination as u64, value as u64)?;
            },
            Opcode::Mov32 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let value = self.memory.get_u32(address)?;
                self.set_register(destination as u64, value as u64)?;
            },
            Opcode::Mov64 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let value = self.memory.get_u64(address)?;
                self.set_register(destination as u64, value as u64)?;
            },
            Opcode::MovR2R => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let value = self.get_register(src as u64)?;
                self.set_register(destination as u64, value as u64)?;
            },

            Opcode::Push8 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                self.stack_push_u8(value as u8)?;
            },
            Opcode::Push16 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                self.stack_push_u16(value as u16)?;
            },
            Opcode::Push32 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                self.stack_push_u32(value as u32)?;
            },
            Opcode::Push64 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                self.stack_push_u32(value as u32)?;
            },

            Opcode::Pop8 => {
                let dest = self.fetch_u8()?;
                let value = self.stack_pop_u8()?;

                self.set_register(dest as u64, value as u64)?;
            },
            Opcode::Pop16 => {
                let dest = self.fetch_u8()?;
                let value = self.stack_pop_u16()?;

                self.set_register(dest as u64, value as u64)?;
            },
            Opcode::Pop32 => {
                let dest = self.fetch_u8()?;
                let value = self.stack_pop_u32()?;

                self.set_register(dest as u64, value as u64)?;
            },
            Opcode::Pop64 => {
                let dest = self.fetch_u8()?;
                let value = self.stack_pop_u64()?;

                self.set_register(dest as u64, value)?;
            },

            Opcode::Frame8 => todo!(),
            Opcode::Frame16 => todo!(),
            Opcode::Frame32 => todo!(),
            Opcode::Frame64 => todo!(),
            
            Opcode::Peek8 => todo!(),
            Opcode::Peek16 => todo!(),
            Opcode::Peek32 => todo!(),
            Opcode::Peek64 => todo!(),

            Opcode::Add8 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u8(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value + value as u64);
            },
            Opcode::Add16 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u16(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value + value as u64);
            },
            Opcode::Add32 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u32(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value + value as u64);
            },
            Opcode::Add64 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u64(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value + value as u64);
            },
            Opcode::AddR2R => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let left = self.get_register(destination as u64)?;
                let right = self.get_register(src as u64)?;

                self.set_register(destination as u64, left + right)?;
            },
            Opcode::XAdd => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let left = self.get_register(destination as u64)?;
                let right = self.get_register(src as u64)?;

                self.set_register(destination as u64, left + right)?;
                self.set_register(src as u64, left)?;
            },
            
            Opcode::Sub8 => todo!(),
            Opcode::Sub16 => todo!(),
            Opcode::Sub32 => todo!(),
            Opcode::Sub64 => todo!(),
            Opcode::SubR2R => todo!(),

            Opcode::Mul8 => todo!(),
            Opcode::Mul16 => todo!(),
            Opcode::Mul32 => todo!(),
            Opcode::Mul64 => todo!(),
            Opcode::MulR2R => todo!(),

            Opcode::Div8 => todo!(),
            Opcode::Div16 => todo!(),
            Opcode::Div32 => todo!(),
            Opcode::Div64 => todo!(),
            Opcode::DivR2R => todo!(),

            Opcode::Cmp8 => todo!(),
            Opcode::Cmp16 => todo!(),
            Opcode::Cmp32 => todo!(),
            Opcode::Cmp64 => todo!(),
            Opcode::CmpR2R => todo!(),

            Opcode::Jmp => todo!(),
            Opcode::Jz => todo!(),
            Opcode::Jnz => todo!(),
            Opcode::Je => todo!(),
            Opcode::Jne => todo!(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_skip_data_section_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        let program = [
            Opcode::DataSection as u8,
            1, 2, 3, 4,
            0xff,
            Opcode::TextSection as u8,
            Opcode::Halt as u8,
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.peek_byte()?, Opcode::Halt as u8);
        Ok(())
    }

    #[test]
    fn vm_skip_data_section_error_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        let program = [
            Opcode::DataSection as u8,
            1, 2, 3, 4,
            Opcode::Halt as u8,
        ];

        vm.insert_program(&program)?;

        assert!(vm.run().is_err());

        Ok(())
    }

    #[test]
    fn instruction_mov8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov %r0 $123
            Opcode::Mov8 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_mov16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0,
            123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov %r0 $123
            Opcode::Mov16 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_mov32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0,
            0,
            0,
            123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov %r0 $123
            Opcode::Mov32 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_mov64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov %r0 $123
            Opcode::Mov64 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_mov_r2r_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R1, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov %r1 %r0
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_add8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // add %r0 $123
            Opcode::Add8 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 + 123);

        Ok(())
    }

    #[test]
    fn instruction_add16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // add %r0 $123
            Opcode::Add16 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 + 123);

        Ok(())
    }

    #[test]
    fn instruction_add32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // add %r0 $123
            Opcode::Add32 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 + 123);

        Ok(())
    }

    #[test]
    fn instruction_add64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // add %r0 $123
            Opcode::Add64 as u8,
            R0 as u8,
            0, // -|
            0, //  |
            0, //  |
            0, //  |=| 64-bit address
            0, //  |=| to data section
            0, //  |
            0, //  |
            1, // -|
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 + 123);

        Ok(())
    }

    #[test]
    fn instruction_add_r2r_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);
        vm.set_register(R1, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // add %r0 $123
            Opcode::AddR2R as u8,
            R0 as u8,
            R1 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 + 123);

        Ok(())
    }

    #[test]
    fn instruction_xadd_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 2);
        vm.set_register(R1, 1);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // xadd %r0 $123
            Opcode::XAdd as u8,
            R0 as u8,
            R1 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 2 + 1);
        assert_eq!(vm.get_register(R1)?, 2);

        Ok(())
    }

    #[test]
    fn instruction_push8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // push8 %r0
            Opcode::Push8 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.stack_get_u8(0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_push16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // push16 %r0
            Opcode::Push16 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.stack_get_u16(0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_push32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // push32 %r0
            Opcode::Push32 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.stack_get_u32(0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_push64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // push64 %r0
            Opcode::Push64 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.stack_get_u64(0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_pop8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;

        dbg!(&vm.memory.inner);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // pop8 %r0
            Opcode::Pop8 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_pop16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u16(123)?;

        dbg!(&vm.memory.inner);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // pop16 %r0
            Opcode::Pop16 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_pop32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u32(123)?;

        dbg!(&vm.memory.inner);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // pop32 %r0
            Opcode::Pop32 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_pop64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u64(123)?;

        dbg!(&vm.memory.inner);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // pop64 %r0
            Opcode::Pop64 as u8,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123);

        Ok(())
    }
}
