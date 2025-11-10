pub struct Registers {
    general_purpose: [u16; 8],
    pub pc: u16,
    cond: ConditionFlag,
}

impl Registers {
    pub const fn new() -> Self {
        Self {
            general_purpose: [0u16; 8],
            pc: crate::hardware::memory::PROGRAM_SECTION_START,
            cond: ConditionFlag::Zero,
        }
    }

    pub fn get(&self, r: u8) -> u16 {
        assert!(r <= 7, "Invalid general purpose register get");
        self.general_purpose[usize::from(r)]
    }
    pub fn set(&mut self, r: u8, value: u16) {
        assert!(r <= 7, "Invalid general purpose register set");
        self.general_purpose[usize::from(r)] = value;
    }

    pub const fn get_conditional_register(&self) -> ConditionFlag {
        self.cond
    }
    fn update_conditional_register(&mut self, r: u8) {
        let val = self.get(r);
        self.cond = ConditionFlag::from(val);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConditionFlag {
    Pos = 1 << 0, // Positive
    Zero = 1 << 1,
    Neg = 1 << 2, // Negative
}

impl From<u16> for ConditionFlag {
    fn from(value: u16) -> Self {
        if value == 0 {
            Self::Zero
        } else if value >> 15 == 1 {
            // leftmost bit is 1 for negative numbers
            Self::Neg
        } else {
            Self::Pos
        }
    }
}
