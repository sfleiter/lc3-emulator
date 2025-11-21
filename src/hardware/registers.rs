use crate::numbers;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Register(u16);
impl Register {
    pub const fn as_binary_u16(self) -> u16 {
        self.0
    }
    pub fn as_binary_u32(self) -> u32 {
        u32::from(self.0)
    }
    pub fn as_decimal(self) -> i16 {
        numbers::twos_complement_to_decimal(self.0)
    }
}
impl Debug for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:#06X} {:#018b} {})",
            self.0,
            self.0,
            self.as_decimal()
        )
    }
}
impl PartialEq<u16> for Register {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}
impl PartialOrd<u16> for Register {
    fn partial_cmp(&self, other: &u16) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}
impl From<u16> for Register {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<Register> for u16 {
    fn from(value: Register) -> Self {
        value.0
    }
}

pub struct Registers {
    general_purpose: [Register; 8],
    pc: Register,
    cond: ConditionFlag,
}
impl Registers {
    pub const fn new() -> Self {
        Self {
            general_purpose: [Register(0); 8],
            pc: Register(crate::hardware::memory::PROGRAM_SECTION_START),
            cond: ConditionFlag::Zero,
        }
    }
    pub const fn pc(&self) -> Register {
        self.pc
    }
    pub fn inc_pc(&mut self) {
        self.set_pc(self.pc.0 + 1);
    }
    pub fn set_pc(&mut self, val: u16) {
        debug_assert!(
            // 0xFE00 is only allowed since the PC is incremented before executing the current instruction
            (0x3000..=0xFE00).contains(&val),
            "Program Counter (PC) must be between 0x3000 and 0xFE00, but is: {val}"
        );
        self.pc = val.into();
    }
    pub fn get_binary(&self, r: u8) -> Register {
        debug_assert!(r <= 7, "Invalid general purpose register get");
        self.general_purpose[usize::from(r)]
    }
    pub fn set_binary(&mut self, r: u8, value: u16) {
        debug_assert!(r <= 7, "Invalid general purpose register set");
        self.general_purpose[usize::from(r)] = Register(value);
    }
    #[cfg(test)]
    pub fn set_decimal(&mut self, r: u8, value: i16) {
        let reg_value: u16 = if value >= 0 {
            u16::try_from(value)
                .expect("Impossible error during conversion from positive i16 to u16")
        } else {
            !value.abs().cast_unsigned() + 1
        };
        self.set_binary(r, reg_value);
    }
    #[cfg(test)]
    pub const fn get_conditional_register(&self) -> ConditionFlag {
        self.cond
    }
    pub fn update_conditional_register(&mut self, r: u8) {
        let val = self.get_binary(r);
        self.cond = ConditionFlag::from(val);
    }
}
impl Debug for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, val) in self.general_purpose.iter().enumerate() {
            writeln!(f, "R{index}:   {val:?}")?;
        }
        writeln!(f)?;
        writeln!(f, "PC:   {:?}", self.pc)?;
        writeln!(f, "Cond: {:?}", self.cond)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionFlag {
    Pos = 1 << 0, // Positive
    Zero = 1 << 1,
    Neg = 1 << 2, // Negative
}
impl From<Register> for ConditionFlag {
    fn from(value: Register) -> Self {
        if value.0 == 0 {
            Self::Zero
        } else if value.0 >> 15 == 1 {
            // leftmost bit is 1 for negative numbers in two's complement
            Self::Neg
        } else {
            Self::Pos
        }
    }
}
