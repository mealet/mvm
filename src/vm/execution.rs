use super::{VM, Opcode, MvmError};

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

            Opcode::Mov8 => todo!(),
            Opcode::Mov16 => todo!(),
            Opcode::Mov32 => todo!(),
            Opcode::Mov64 => todo!(),
            Opcode::MovR2R => todo!(),

            Opcode::Add8 => todo!(),
            Opcode::Add16 => todo!(),
            Opcode::Add32 => todo!(),
            Opcode::Add64 => todo!(),
            Opcode::AddR2R => todo!(),
            Opcode::XAdd => todo!(),
            
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
