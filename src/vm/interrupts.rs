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

        match self.get_register(R_SYSTEM_CALL)? {
            // exit
            0 => {
                self.exit_code = self.get_register(R0)? as u8;
                self.running = false;
            }

            2 => {
                let fd = self.get_register(R0)? as libc::c_int;
                let len = self.get_register(R2)? as usize;

                let in_mem_buffer = self.get_register(R1)? as usize;
                let buffer = unsafe {
                    self.memory.inner.as_ptr().add(in_mem_buffer) as *const libc::c_void
                };

                self.set_register(R_ACCUMULATOR, unsafe { libc::write(fd, buffer, len) } as u64);
            }

            unknown => {
                return Err(MvmError::UnknownSystemCall(unknown));
            }
        }
        
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
