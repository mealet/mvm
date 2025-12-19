use super::{
    VM, Opcode, MvmError,
    R0, R1, R2, R3, R4, R5, R6, R7, R8,
    R_SYSTEM_CALL, R_ACCUMULATOR, R_INSTRUCTION_POINTER,
    R_STACK_POINTER, R_FRAME_POINTER, R_MEMORY_POINTER
};

impl VM {
    pub fn execute_instruction(&mut self, instruction: u8) -> Result<(), MvmError> {
        if self.text_section && instruction == 0xFF {
            let address = self.fetch_u64()?;
            self.set_register(R_INSTRUCTION_POINTER, address)?;

            return Ok(());
        }

        let opcode = Opcode::try_from(instruction)?;

        match opcode {
            Opcode::Halt => {
                self.running = false;
                let _ = self.step_back()?;
            },
            Opcode::Return => {
                self.pop_state()?;
            },
            Opcode::Call => {
                let address = self.fetch_u64()?;

                self.push_state()?;
                self.set_register(R_INSTRUCTION_POINTER, address)?;
            },
            Opcode::Interrupt => {
                let address = self.fetch_u64()?;
                let vector = self.memory.get_u8(address)?;

                if let Some(handler) = self.interrupt_handlers[vector as usize] {
                    self.push_state()?;
                    handler(self);
                } else {
                    return Err(MvmError::UnknownInterrupt);
                }
            },

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
            Opcode::TextSection => {
                self.text_section = true;
            },

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

            Opcode::MovR2M8 => {
                let address = self.fetch_u64()?;
                let src = self.fetch_u8()?;

                let value = self.get_register(src as u64)?;
                self.memory.set_u8(address, value as u8);
            },

            Opcode::MovR2M16 => {
                let address = self.fetch_u64()?;
                let src = self.fetch_u8()?;

                let value = self.get_register(src as u64)?;
                self.memory.set_u16(address, value as u16);
            }

            Opcode::MovR2M32 => {
                let address = self.fetch_u64()?;
                let src = self.fetch_u8()?;

                let value = self.get_register(src as u64)?;
                self.memory.set_u32(address, value as u32);
            },

            Opcode::MovR2M64 => {
                let address = self.fetch_u64()?;
                let src = self.fetch_u8()?;

                let value = self.get_register(src as u64)?;
                self.memory.set_u64(address, value);
            },

            Opcode::Push8 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                let stack_ptr = self.get_register(R_STACK_POINTER)?;
                let frame_ptr= self.get_register(R_FRAME_POINTER)?;

                let offset = stack_ptr - frame_ptr;

                self.stack_push_u8(value as u8)?;
                self.set_register(src as u64, offset)?;
            },
            Opcode::Push16 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                let stack_ptr = self.get_register(R_STACK_POINTER)?;
                let frame_ptr= self.get_register(R_FRAME_POINTER)?;

                let offset = stack_ptr - frame_ptr;

                self.stack_push_u16(value as u16)?;
                self.set_register(src as u64, offset)?;
            },
            Opcode::Push32 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                let stack_ptr = self.get_register(R_STACK_POINTER)?;
                let frame_ptr= self.get_register(R_FRAME_POINTER)?;

                let offset = stack_ptr - frame_ptr;

                self.stack_push_u32(value as u32)?;
                self.set_register(src as u64, offset)?;
            },
            Opcode::Push64 => {
                let src = self.fetch_u8()?;
                let value = self.get_register(src as u64)?;

                let stack_ptr = self.get_register(R_STACK_POINTER)?;
                let frame_ptr= self.get_register(R_FRAME_POINTER)?;

                let offset = stack_ptr - frame_ptr;

                self.stack_push_u32(value as u32)?;
                self.set_register(src as u64, offset)?;
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

            Opcode::Frame8 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.frame_get_u8(offset)?;

                self.set_register(dest as u64, value as u64);
            },
            Opcode::Frame16 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.frame_get_u16(offset)?;

                self.set_register(dest as u64, value as u64);
            },
            Opcode::Frame32 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.frame_get_u32(offset)?;

                self.set_register(dest as u64, value as u64);
            },
            Opcode::Frame64 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.frame_get_u64(offset)?;

                self.set_register(dest as u64, value as u64);
            },
            
            Opcode::Peek8 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.stack_get_u8(offset)?;

                self.set_register(dest as u64, value as u64);
            },
            Opcode::Peek16 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.stack_get_u16(offset)?;

                self.set_register(dest as u64, value as u64);
            },
            Opcode::Peek32 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.stack_get_u32(offset)?;

                self.set_register(dest as u64, value as u64);
            },
            Opcode::Peek64 => {
                let dest = self.fetch_u8()?;
                let address = self.fetch_u64()?;

                let offset = self.memory.get_u16(address)?;
                let value = self.stack_get_u64(offset)?;

                self.set_register(dest as u64, value as u64);
            },

            Opcode::Add8 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u8(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_add(value as u64));
            },
            Opcode::Add16 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u16(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_add(value as u64));
            },
            Opcode::Add32 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u32(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_add(value as u64));
            },
            Opcode::Add64 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u64(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_add(value as u64));
            },
            Opcode::AddR2R => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let left = self.get_register(destination as u64)?;
                let right = self.get_register(src as u64)?;

                self.set_register(destination as u64, left.wrapping_add(right))?;
            },
            Opcode::XAdd => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let left = self.get_register(destination as u64)?;
                let right = self.get_register(src as u64)?;

                self.set_register(destination as u64, left.wrapping_add(right))?;
                self.set_register(src as u64, left)?;
            },
            
            Opcode::Sub8 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u8(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_sub(value as u64));
            },
            Opcode::Sub16 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u16(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_sub(value as u64));
            },
            Opcode::Sub32 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u32(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_sub(value as u64));
            },
            Opcode::Sub64 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u64(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_sub(value as u64));
            },
            Opcode::SubR2R => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let left = self.get_register(destination as u64)?;
                let right = self.get_register(src as u64)?;

                self.set_register(destination as u64, left.wrapping_sub(right))?;
            },

            Opcode::Mul8 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u8(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_mul(value as u64));
            },
            Opcode::Mul16 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u16(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_mul(value as u64));
            },
            Opcode::Mul32 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u32(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_mul(value as u64));
            },
            Opcode::Mul64 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u64(address)?;

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_mul(value as u64));
            },
            Opcode::MulR2R => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let left = self.get_register(destination as u64)?;
                let right = self.get_register(src as u64)?;

                self.set_register(destination as u64, left.wrapping_mul(right))?;
            },

            Opcode::Div8 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u8(address)?;

                if value == 0 {
                    return Err(MvmError::DivisionByZero);
                }

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_div(value as u64));
            },
            Opcode::Div16 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u16(address)?;

                if value == 0 {
                    return Err(MvmError::DivisionByZero);
                }

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_div(value as u64));
            },
            Opcode::Div32 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u32(address)?;

                if value == 0 {
                    return Err(MvmError::DivisionByZero);
                }

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_div(value as u64));
            },
            Opcode::Div64 => {
                let destination = self.fetch_u8()?;
                let address = self.fetch_u64()?;
                let value = self.memory.get_u64(address)?;

                if value == 0 {
                    return Err(MvmError::DivisionByZero);
                }

                let dest_value = self.get_register(destination as u64)?;
                self.set_register(destination as u64, dest_value.wrapping_div(value as u64));
            },
            Opcode::DivR2R => {
                let destination = self.fetch_u8()?;
                let src = self.fetch_u8()?;

                let left = self.get_register(destination as u64)?;
                let right = self.get_register(src as u64)?;

                if right == 0 {
                    return Err(MvmError::DivisionByZero);
                }

                self.set_register(destination as u64, left.wrapping_div(right))?;
            },

            Opcode::Cmp8 => {
                let reg = self.fetch_u8()?;
                let addr = self.fetch_u64()?;

                let reg_value = self.get_register(reg as u64)?;
                let addr_value = self.memory.get_u8(addr)? as u64;

                let cmp_result = if reg_value > addr_value { 1 } else if reg_value < addr_value { 2 } else { 0 };

                self.set_register(R_ACCUMULATOR, cmp_result)?;
            },
            Opcode::Cmp16 => {
                let reg = self.fetch_u8()?;
                let addr = self.fetch_u64()?;

                let reg_value = self.get_register(reg as u64)?;
                let addr_value = self.memory.get_u16(addr)? as u64;

                let cmp_result = if reg_value > addr_value { 1 } else if reg_value < addr_value { 2 } else { 0 };

                self.set_register(R_ACCUMULATOR, cmp_result)?;
            },
            Opcode::Cmp32 => {
                let reg = self.fetch_u8()?;
                let addr = self.fetch_u64()?;

                let reg_value = self.get_register(reg as u64)?;
                let addr_value = self.memory.get_u32(addr)? as u64;

                let cmp_result = if reg_value > addr_value { 1 } else if reg_value < addr_value { 2 } else { 0 };

                self.set_register(R_ACCUMULATOR, cmp_result)?;
            },
            Opcode::Cmp64 => {
                let reg = self.fetch_u8()?;
                let addr = self.fetch_u64()?;

                let reg_value = self.get_register(reg as u64)?;
                let addr_value = self.memory.get_u64(addr)? as u64;

                let cmp_result = if reg_value > addr_value { 1 } else if reg_value < addr_value { 2 } else { 0 };

                self.set_register(R_ACCUMULATOR, cmp_result)?;
            },
            Opcode::CmpR2R => {
                let left_reg = self.fetch_u8()?;
                let right_reg= self.fetch_u8()?;

                let left_value = self.get_register(left_reg as u64)?;
                let right_value = self.get_register(right_reg as u64)?;

                let cmp_result = if left_reg > right_reg { 1 } else if left_reg < right_reg { 2 } else { 0 };

                self.set_register(R_ACCUMULATOR, cmp_result)?;
            },

            Opcode::Jmp => {
                let addr = self.fetch_u64()?;
                self.set_register(R_INSTRUCTION_POINTER, addr)?;
            },
            Opcode::Jz => {
                let addr = self.fetch_u64()?;
                let acc_value = self.get_register(R_ACCUMULATOR)?;

                if acc_value == 0 {
                    self.set_register(R_INSTRUCTION_POINTER, addr)?;
                }
            },
            Opcode::Jnz => {
                let addr = self.fetch_u64()?;
                let acc_value = self.get_register(R_ACCUMULATOR)?;

                if acc_value != 0 {
                    self.set_register(R_INSTRUCTION_POINTER, addr)?;
                }
            },
            Opcode::Je => {
                let val_addr = self.fetch_u64()?;
                let label_addr = self.fetch_u64()?;

                let data_value = self.memory.get_u64(val_addr)?;
                let acc_value = self.get_register(R_ACCUMULATOR)?;

                if acc_value == data_value {
                    self.set_register(R_INSTRUCTION_POINTER, label_addr)?;
                }
            },
            Opcode::Jne => {
                let val_addr = self.fetch_u64()?;
                let label_addr = self.fetch_u64()?;

                let data_value = self.memory.get_u64(val_addr)?;
                let acc_value = self.get_register(R_ACCUMULATOR)?;

                if acc_value != data_value {
                    self.set_register(R_INSTRUCTION_POINTER, label_addr)?;
                }
            },
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
    fn instruction_mov_r2m8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov 30 %r0
            Opcode::MovR2M8 as u8,
            0, 0, 0, 0, 0, 0, 0, 30,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.memory.get_u8(30)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_mov_r2m16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov 30 %r0
            Opcode::MovR2M16 as u8,
            0, 0, 0, 0, 0, 0, 0, 30,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.memory.get_u16(30)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_mov_r2m32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov 30 %r0
            Opcode::MovR2M32 as u8,
            0, 0, 0, 0, 0, 0, 0, 30,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.memory.get_u32(30)?, 123);

        Ok(())
    }

    #[test]
    fn instruction_mov_r2m64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // mov 30 %r0
            Opcode::MovR2M64 as u8,
            0, 0, 0, 0, 0, 0, 0, 30,
            R0 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.memory.get_u64(30)?, 123);

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
            // add %r0 %r1
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
    fn instruction_sub8_test() -> Result<(), MvmError> {
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
            // sub %r0 $123
            Opcode::Sub8 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 - 123);

        Ok(())
    }

    #[test]
    fn instruction_sub16_test() -> Result<(), MvmError> {
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
            // sub %r0 $123
            Opcode::Sub16 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 - 123);

        Ok(())
    }

    #[test]
    fn instruction_sub32_test() -> Result<(), MvmError> {
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
            // sub %r0 $123
            Opcode::Sub32 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 - 123);

        Ok(())
    }

    #[test]
    fn instruction_sub64_test() -> Result<(), MvmError> {
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
            // sub %r0 $123
            Opcode::Sub64 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 - 123);

        Ok(())
    }

    #[test]
    fn instruction_sub_r2r_test() -> Result<(), MvmError> {
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
            // sub %r0 %r1
            Opcode::SubR2R as u8,
            R0 as u8,
            R1 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 - 123);

        Ok(())
    }

    #[test]
    fn instruction_mul8_test() -> Result<(), MvmError> {
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
            // mul %r0 $123
            Opcode::Mul8 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 * 123);

        Ok(())
    }

    #[test]
    fn instruction_mul16_test() -> Result<(), MvmError> {
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
            // mul %r0 $123
            Opcode::Mul16 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 * 123);

        Ok(())
    }

    #[test]
    fn instruction_mul32_test() -> Result<(), MvmError> {
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
            // mul %r0 $123
            Opcode::Mul32 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 * 123);

        Ok(())
    }

    #[test]
    fn instruction_mul64_test() -> Result<(), MvmError> {
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
            // mul %r0 $123
            Opcode::Mul64 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 * 123);

        Ok(())
    }

    #[test]
    fn instruction_mul_r2r_test() -> Result<(), MvmError> {
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
            // mul %r0 %r1
            Opcode::MulR2R as u8,
            R0 as u8,
            R1 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 * 123);

        Ok(())
    }

    #[test]
    fn instruction_div8_test() -> Result<(), MvmError> {
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
            // div %r0 $123
            Opcode::Div8 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 / 123);

        Ok(())
    }

    #[test]
    fn instruction_div16_test() -> Result<(), MvmError> {
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
            // div %r0 $123
            Opcode::Div16 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 / 123);

        Ok(())
    }

    #[test]
    fn instruction_div32_test() -> Result<(), MvmError> {
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
            // div %r0 $123
            Opcode::Div32 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 / 123);

        Ok(())
    }

    #[test]
    fn instruction_div64_test() -> Result<(), MvmError> {
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
            // div %r0 $123
            Opcode::Div64 as u8,
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

        assert_eq!(vm.get_register(R0)?, 123 / 123);

        Ok(())
    }

    #[test]
    fn instruction_div_r2r_test() -> Result<(), MvmError> {
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
            // div %r0 %r1
            Opcode::DivR2R as u8,
            R0 as u8,
            R1 as u8,
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 123 / 123);

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
        assert_eq!(vm.get_register(R0)?, 0);

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
        assert_eq!(vm.get_register(R0)?, 0);

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
        assert_eq!(vm.get_register(R0)?, 0);

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
        assert_eq!(vm.get_register(R0)?, 0);

        Ok(())
    }

    #[test]
    fn instruction_pop8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;

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

    #[test]
    fn instruction_frame8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u8(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 1,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // frame8 %r0 $0
            Opcode::Frame8 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }

    #[test]
    fn instruction_frame16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u16(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 1,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // frame16 %r0 $0
            Opcode::Frame16 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }

    #[test]
    fn instruction_frame32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u32(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 1,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // frame32 %r0 $0
            Opcode::Frame32 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }

    #[test]
    fn instruction_frame64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u64(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 1,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // frame64 %r0 $0
            Opcode::Frame64 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }

    #[test]
    fn instruction_peek8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u8(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // peek8 %r0 $0
            Opcode::Peek8 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }

    #[test]
    fn instruction_peek16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u16(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // peek16 %r0 $0
            Opcode::Peek16 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }

    #[test]
    fn instruction_peek32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u32(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // peek32 %r0 $0
            Opcode::Peek32 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }

    #[test]
    fn instruction_peek64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.stack_push_u8(123)?;
        vm.stack_push_u64(255)?;

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // peek64 %r0 $0
            Opcode::Peek64 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R0)?, 255);

        Ok(())
    }


    #[test]
    fn instruction_cmp8_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 50);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // cmp %r0 $123
            Opcode::Cmp8 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R_ACCUMULATOR)?, 2);

        Ok(())
    }

    #[test]
    fn instruction_cmp16_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 50);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // cmp %r0 $123
            Opcode::Cmp16 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R_ACCUMULATOR)?, 2);

        Ok(())
    }

    #[test]
    fn instruction_cmp32_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 50);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // cmp %r0 $123
            Opcode::Cmp32 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R_ACCUMULATOR)?, 2);

        Ok(())
    }

    #[test]
    fn instruction_cmp64_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 50);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --
            // cmp %r0 $123
            Opcode::Cmp64 as u8,
            R0 as u8,
            0, 0, 0, 0, 0, 0, 0, 1, // 64-bit address to data section
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R_ACCUMULATOR)?, 2);

        Ok(())
    }

    #[test]
    fn instruction_jmp_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 0);
        vm.set_register(R1, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // jmp label
            Opcode::Jmp as u8,
            0, 0, 0, 0, 0, 0, 0, 15,

            // mov %r0 %r1
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,

            // label:
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(0)?, 0);

        Ok(())
    }

    #[test]
    fn instruction_jz_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 0);
        vm.set_register(R1, 123);
        vm.set_register(R_ACCUMULATOR, 0);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // jz label
            Opcode::Jz as u8,
            0, 0, 0, 0, 0, 0, 0, 15,

            // mov %r0 %r1
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,

            // label:
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(0)?, 0);

        Ok(())
    }

    #[test]
    fn instruction_jnz_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 0);
        vm.set_register(R1, 123);
        vm.set_register(R_ACCUMULATOR, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // jnz label
            Opcode::Jnz as u8,
            0, 0, 0, 0, 0, 0, 0, 15,

            // mov %r0 %r1
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,

            // label:
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(0)?, 0);

        Ok(())
    }

    #[test]
    fn instruction_je_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 0);
        vm.set_register(R1, 123);
        vm.set_register(R_ACCUMULATOR, 123);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // jnz label
            Opcode::Je as u8,
            0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 31,

            // mov %r0 %r1
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,

            // label:
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(0)?, 0);

        Ok(())
    }

    #[test]
    fn instruction_jne_test() -> Result<(), MvmError> {
        let mut vm = VM::new(64, 16)?;

        vm.set_register(R0, 0);
        vm.set_register(R1, 123);
        vm.set_register(R_ACCUMULATOR, 0);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // jnz label
            Opcode::Jne as u8,
            0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 31,

            // mov %r0 %r1
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,

            // label:
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(0)?, 0);

        Ok(())
    }

    #[test]
    fn instruction_call_test() -> Result<(), MvmError> {
        let mut vm = VM::new(256, 128)?;

        vm.set_register(R0, 0);
        vm.set_register(R1, 123);
        vm.set_register(R_ACCUMULATOR, 0);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // call label
            Opcode::Call as u8,
            0, 0, 0, 0, 0, 0, 0, 23,

            // mov %r0 %r1
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,

            // label:
            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(0)?, 0);

        Ok(())
    }

    #[test]
    fn instruction_ret_test() -> Result<(), MvmError> {
        let mut vm = VM::new(256, 128)?;

        vm.set_register(R0, 0);
        vm.set_register(R1, 123);
        vm.set_register(R_ACCUMULATOR, 0);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 123,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // call label
            Opcode::Call as u8,
            0, 0, 0, 0, 0, 0, 0, 32,

            // mov %r0 %r1
            Opcode::MovR2R as u8,
            R0 as u8,
            R1 as u8,

            Opcode::Jmp as u8,
            0, 0, 0, 0, 0, 0, 0, 33,

            // label:
            Opcode::Return as u8,

            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(0)?, 123);

        Ok(())
    }
}
