use crate::numbers;
use std::fmt::{Debug, Formatter};

/// Wrapper for LC-3 u16 instruction.
/// format is: `OOOO_DDD_P_PPPP_PPPP`
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Instruction(u16);

impl Instruction {
    /// Gives the value of only the specified bit range.
    ///
    /// # Parameters
    /// - `from`: starting index
    /// - `to`: end index (inclusive), mut be greater or equal to `from`
    ///
    /// # Panics
    /// - asserts that to is greater or equal from and both are valid indexes
    #[must_use]
    pub fn get_bit_range(self, from: u8, to: u8) -> u16 {
        debug_assert!(
            to >= from,
            "wrong direction of from: {from:?} and to: {to:?}"
        );
        debug_assert!(
            (00..u16::BITS).contains(&u32::from(to)),
            "index: {to:?} to u16 is greater than maximum value {:?}",
            u16::BITS - 1
        );
        (self.0 >> from) & ((0b1 << (to - from + 1)) - 1)
    }
    /// Gives the value of only the specified bit range and converts that to u8.
    /// See [`Instruction::get_bit_range()`]
    /// # Panics
    /// - value does not fit into u8 with message from `expect`
    #[must_use]
    pub fn get_bit_range_u8(self, from: u8, to: u8, expect: &str) -> u8 {
        u8::try_from(self.get_bit_range(from, to)).expect(expect)
    }
    #[must_use]
    pub fn get_bit(self, index: u8) -> bool {
        self.get_bit_range(index, index) & 1 != 0
    }
    #[must_use]
    pub fn op_code(self) -> u8 {
        self.get_bit_range_u8(12, 15, "Error parsing op_code")
    }
    #[must_use]
    pub fn dr_number(self) -> u8 {
        self.get_bit_range_u8(9, 11, "Error parsing dr")
    }
    #[must_use]
    pub fn sr1_number(self) -> u8 {
        self.get_bit_range_u8(6, 8, "Error parsing sr1")
    }
    #[must_use]
    pub fn sr2_number(self) -> u8 {
        self.get_bit_range_u8(0, 2, "Error parsing sr2")
    }
    #[must_use]
    pub fn is_immediate(self) -> bool {
        self.get_bit_range(5, 5) == 1
    }
    pub fn get_immediate(self) -> u16 {
        Self::sign_extend(self.get_bit_range(0, 4), 5)
    }
    /// Offset to add to program counter PC.
    /// Can be positive or negative.
    #[must_use]
    pub fn pc_offset(self, len: u8) -> i16 {
        let bin_rep = Self::sign_extend(self.get_bit_range(0, len - 1), len);
        let res = numbers::twos_complement_to_decimal(bin_rep);
        #[expect(clippy::cast_possible_truncation)]
        {
            debug_assert!(
                ((-(2 << (len - 1))) as i16..(2 << (len - 1))).contains(&res),
                "pc_offset out of range"
            );
        }
        res
    }
    /// Implements sign extension as described at [Sign extension](https://en.wikipedia.org/wiki/Sign_extension).
    #[must_use]
    const fn sign_extend(bits: u16, valid_bits: u8) -> u16 {
        let most_significant_bit = bits >> (valid_bits - 1);
        if most_significant_bit == 1 {
            // negative: 1-extend
            bits | (0xFFFF << valid_bits)
        } else {
            // positive, already 0-extended
            bits
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Op: {:04b}, DR: {:03b}, PC_Off: {:09b}",
            self.op_code(),
            self.dr_number(),
            self.pc_offset(9)
        )
    }
}

impl From<u16> for Instruction {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[gtest]
    pub fn test_instr_get_bit_range_valid() {
        let sut = Instruction::from(0b1010_101_001010101);
        expect_that!(sut.op_code(), eq(0b1010));
        expect_that!(sut.dr_number(), eq(0b101));
        expect_that!(sut.pc_offset(9), eq(0b0_0101_0101));

        // Add: DR: 3, SR1: 2, Immediate: false, SR2: 1
        let sut = Instruction::from(0b0001_011_010_0_00_001);
        expect_that!(sut.op_code(), eq(1));
        expect_that!(sut.dr_number(), eq(3));
        expect_that!(sut.sr1_number(), eq(2));
        expect_that!(sut.sr2_number(), eq(1));
        expect_that!(sut.is_immediate(), eq(false));
        expect_that!(sut.sr2_number(), eq(1));

        // Add: DR: 7, SR1: 0, Immediate: true, imm5: 30
        let sut = Instruction::from(0b0001_111_000_1_01110);
        expect_that!(sut.op_code(), eq(1));
        expect_that!(sut.dr_number(), eq(7));
        expect_that!(sut.sr1_number(), eq(0));
        expect_that!(sut.is_immediate(), eq(true));
        expect_that!(sut.get_immediate(), eq(14));
    }
    #[gtest]
    #[should_panic(expected = "wrong direction of from: 2 and to: 1")]
    pub fn test_instr_get_bit_range_wrong_order() {
        let sut = Instruction::from(0b1010_101_101010101);
        let _ = sut.get_bit_range(2, 1);
    }
    #[gtest]
    #[should_panic(expected = "index: 16 to u16 is greater than maximum value 15")]
    pub fn test_instr_get_bit_range_index_too_large() {
        let sut = Instruction::from(0b1010_101_101010101);
        let _ = sut.get_bit_range(2, 16);
    }
}
