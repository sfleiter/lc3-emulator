use crate::hardware::memory;
use crate::numbers;
use std::fmt::{Debug, Formatter};

#[must_use]
pub const fn from_binary(val: u16) -> Register {
    Register::from_binary(val)
}
#[must_use]
pub fn from_decimal(val: i16) -> Register {
    Register::from_decimal(val)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register(u16);
impl Register {
    #[must_use]
    pub const fn from_binary(val: u16) -> Self {
        Self(val)
    }
    #[must_use]
    pub fn from_decimal(val: i16) -> Self {
        Self::from_binary(numbers::decimal_to_twos_complement(val))
    }
    #[must_use]
    pub const fn as_binary(self) -> u16 {
        self.0
    }
    #[must_use]
    pub(crate) fn as_binary_u32(self) -> u32 {
        u32::from(self.0)
    }
    #[must_use]
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
pub struct Registers {
    general_purpose: [Register; 8],
    pc: Register,
    cond: ConditionFlag,
}
impl Registers {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            general_purpose: [Register(0); 8],
            pc: Register(memory::PROGRAM_SECTION_START),
            cond: ConditionFlag::Zero,
        }
    }
    #[must_use]
    pub const fn pc(&self) -> Register {
        self.pc
    }
    pub fn inc_pc(&mut self) {
        self.set_pc(self.pc.0 + 1);
    }
    pub fn set_pc(&mut self, val: u16) {
        debug_assert!(
            // one behind valid addresses allowed since the PC is incremented
            // before executing the current instruction
            (memory::PROGRAM_SECTION_START..=(memory::PROGRAM_SECTION_END + 1)).contains(&val),
            "Program Counter (PC) must be between 0x3000 and 0xFE00, but is: {val:#06X}"
        );
        self.pc = Register::from_binary(val);
    }
    #[must_use]
    pub fn get(&self, r: u8) -> Register {
        debug_assert!(
            r <= 7,
            "Invalid general purpose register get {r}, must be 0 to 7"
        );
        self.general_purpose[usize::from(r)]
    }
    pub fn set(&mut self, r: u8, value: Register) {
        debug_assert!(
            r <= 7,
            "Invalid general purpose register set {r}, must be 0 to 7"
        );
        self.general_purpose[usize::from(r)] = value;
    }
    #[must_use]
    pub const fn get_conditional_register(&self) -> ConditionFlag {
        self.cond
    }
    pub fn update_conditional_register(&mut self, r: u8) {
        let val = self.get(r);
        self.cond = ConditionFlag::from(val);
    }
}
impl Default for Registers {
    fn default() -> Self {
        Self::new()
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
