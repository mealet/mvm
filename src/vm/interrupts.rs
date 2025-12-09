use super::*;

impl VM {
    pub fn init_interrupts(&mut self) {
        // WARNING: Don't forget to add implemented interrupt here!

        self.interrupt_handlers[0] = Some(Self::handle_int0);
        self.interrupt_handlers[80] = Some(Self::handle_int80);
    }
}

impl VM {
    // increment accumulator
    fn handle_int0(&mut self) -> Result<(), MvmError> {
        let acc = self.get_register(R_ACCUMULATOR)?;
        self.set_register(R_ACCUMULATOR, acc.wrapping_add(1))?;

        dbg!(self.get_register(R_ACCUMULATOR));

        // return instruction
        self.pop_state()?;
        Ok(())
    }

    // system call
    fn handle_int80(&mut self) -> Result<(), MvmError> {
        // TODO: Implement system call       
        
        // return instruction
        self.pop_state()?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupt_0_test() -> Result<(), MvmError> {
        let mut vm = VM::new(256, 128)?;

        vm.set_register(R_ACCUMULATOR, 0);

        let program = [
            Opcode::DataSection as u8,
            // -- data section --
            0, 0, 0, 0, 0, 0, 0, 0,
            // -- data section end --
            0xff,
            Opcode::TextSection as u8,
            // -- program --

            // int $1
            Opcode::Interrupt as u8,
            0, 0, 0, 0, 0, 0, 0, 1,

            // -- program end --
            Opcode::Halt as u8
        ];

        vm.insert_program(&program)?;
        vm.run()?;

        assert_eq!(vm.get_register(R_ACCUMULATOR)?, 1);

        Ok(())
    }
}
