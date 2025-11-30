use super::error::MvmError;

#[repr(u8)]
pub enum Opcode {
    // ---| System |---
    
    Halt = 0xf0,         // halt
    Return = 0xf1,       // ret
    Call = 0x27,         // call &label
    Interrupt = 0xf2,    // int u8
    
    // ---| Sections |---
    
    DataSection = 0x01,  // section .data
    TextSection = 0x02,  // section .text

    // ---| Values Management |---

    Mov8 = 0x03,         // mov %dest, $u8
    Mov16 = 0x04,        // mov %dest, $u16
    Mov32 = 0x05,        // mov %dest, $u32
    Mov64 = 0x06,        // mov %dest, $u64
    MovR2R = 0x07,       // mov %dest, %src
    
    // ---| Arithmetics |---

    Add8 = 0x08,         // add %dest, $u8
    Add16 = 0x09,        // add %dest, $u16
    Add32 = 0x0a,        // add %dest, $u32
    Add64 = 0x0b,        // add %dest, $u64
    AddR2R = 0x0c,       // add %dest, %src (dest + src)
    XAdd = 0x1c,         // xadd %dest, %src
    
    Sub8 = 0x0d,         // sub %dest, $u8
    Sub16 = 0x0e,        // sub %dest, $u16
    Sub32 = 0x0f,        // sub %dest, $u32
    Sub64 = 0x10,        // sub %dest, $u64
    SubR2R = 0x11,       // sub %dest, %src (dest - src)
    
    Mul8 = 0x12,         // mul %dest, $u8
    Mul16 = 0x13,        // mul %dest, $u16
    Mul32 = 0x14,        // mul %dest, $u32
    Mul64 = 0x15,        // mul %dest, $u64
    MulR2R = 0x16,       // mul %dest, %reg (dest * src)
    
    Div8 = 0x17,         // div %dest, $u8
    Div16 = 0x18,        // div %dest, $u16
    Div32 = 0x19,        // div %dest, $u32
    Div64 = 0x1a,        // div %dest, $u64
    DivR2R = 0x1b,       // div %dest, %src (dest / src)
    
    Cmp8 = 0x20,         // cmp %reg, $u8
    Cmp16 = 0x23,        // cmp %reg, $u16
    Cmp32 = 0x24,        // cmp %reg, $u32
    Cmp64 = 0x25,        // cmp %reg, $u64
    CmpR2R = 0x26,       // cmp %reg, %reg
    
    // Comparison result goes to accumulator:
    // * left value bigger = -1
    // * right value bigger = 1
    // * both are equal = 0 (zero flag will be turned on)

    // ---| Movement |---

    Jmp = 0x1d,          // jmp &label
    Jz = 0x1e,           // jz &label
    Jnz = 0x1f,          // jnz &label
    Je = 0x21,           // je $u64 &label
    Jne = 0x22,          // jne $u64 &label
}

impl TryFrom<u8> for Opcode {
    type Error = MvmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        return match value {
            0xf0 => Ok(Opcode::Halt),
            0xf1 => Ok(Opcode::Return),
            0x27 => Ok(Opcode::Call),
            0xf2 => Ok(Opcode::Interrupt),

            0x01 => Ok(Opcode::DataSection),
            0x02 => Ok(Opcode::TextSection),

            0x03 => Ok(Opcode::Mov8),
            0x04 => Ok(Opcode::Mov16),
            0x05 => Ok(Opcode::Mov32),
            0x06 => Ok(Opcode::Mov64),
            0x07 => Ok(Opcode::MovR2R),

            0x08 => Ok(Opcode::Add8),
            0x09 => Ok(Opcode::Add16),
            0x0a => Ok(Opcode::Add32),
            0x0b => Ok(Opcode::Add64),
            0x0c => Ok(Opcode::AddR2R),
            0x1c => Ok(Opcode::XAdd),

            0x0d => Ok(Opcode::Sub8),
            0x0e => Ok(Opcode::Sub16),
            0x0f => Ok(Opcode::Sub32),
            0x10 => Ok(Opcode::Sub64),
            0x11 => Ok(Opcode::SubR2R),

            0x12 => Ok(Opcode::Mul8),
            0x13 => Ok(Opcode::Mul16),
            0x14 => Ok(Opcode::Mul32),
            0x15 => Ok(Opcode::Mul64),
            0x16 => Ok(Opcode::MulR2R),

            0x17 => Ok(Opcode::Div8),
            0x18 => Ok(Opcode::Div16),
            0x19 => Ok(Opcode::Div32),
            0x1a => Ok(Opcode::Div64),
            0x1b => Ok(Opcode::DivR2R),

            0x20 => Ok(Opcode::Cmp8),
            0x23 => Ok(Opcode::Cmp16),
            0x24 => Ok(Opcode::Cmp32),
            0x25 => Ok(Opcode::Cmp64),
            0x26 => Ok(Opcode::CmpR2R),

            0x1d => Ok(Opcode::Jmp),
            0x1e => Ok(Opcode::Jz),
            0x1f => Ok(Opcode::Jnz),
            0x21 => Ok(Opcode::Je),
            0x22 => Ok(Opcode::Jne),

            _ => Err(MvmError::InvalidOpcode),
        };
    }
}
