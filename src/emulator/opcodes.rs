use crate::emulator::instruction::Instruction;
use crate::hardware::memory::Memory;
use crate::hardware::registers::{ConditionFlag, Registers};

#[allow(
    clippy::cast_possible_truncation,
    reason = "truncation is what is specified for the LC-3 add opcode"
)]
pub fn add(i: Instruction, r: &mut Registers) {
    r.set_binary(
        i.dr_number(),
        (r.get_binary(i.sr1_number()).as_binary_u32()
            + (if i.is_immediate() {
                u32::from(i.get_immediate())
            } else {
                r.get_binary(i.sr2_number()).as_binary_u32()
            })) as u16,
    );
    r.update_conditional_register(i.dr_number());
}
pub fn and(i: Instruction, r: &mut Registers) {
    r.set_binary(
        i.dr_number(),
        r.get_binary(i.sr1_number()).as_binary_u16()
            & (if i.is_immediate() {
                i.get_immediate()
            } else {
                r.get_binary(i.sr2_number()).as_binary_u16()
            }),
    );
    r.update_conditional_register(i.dr_number());
}
pub fn not(i: Instruction, r: &mut Registers) {
    r.set_binary(i.dr_number(), !r.get_binary(i.sr1_number()).as_binary_u16());
    r.update_conditional_register(i.dr_number());
}
pub fn br(i: Instruction, r: &mut Registers) {
    let none_set = i.get_bit_range(9, 11) == 0;
    let do_break = none_set
        || match r.get_conditional_register() {
            ConditionFlag::Pos => i.get_bit(9),
            ConditionFlag::Zero => i.get_bit(10),
            ConditionFlag::Neg => i.get_bit(11),
        };
    if do_break {
        r.set_pc((r.pc().as_decimal() + i.pc_offset(9)).cast_unsigned());
    }
}
pub fn jmp_or_ret(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn jsr(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn ld(i: Instruction, r: &mut Registers, memory: &Memory) {
    let value = memory[address_by_pc_offset(r, i.pc_offset(9))];
    r.set_binary(i.dr_number(), value);
    r.update_conditional_register(i.dr_number());
}

fn address_by_pc_offset(r: &Registers, offset: i16) -> u16 {
    let address = r.pc().as_decimal() + offset;
    address.cast_unsigned()
}

pub fn ldi(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn ldr(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn lea(i: Instruction, r: &mut Registers) {
    r.set_binary(
        i.dr_number(),
        (r.pc().as_binary_u16().cast_signed() + i.pc_offset(9)).cast_unsigned(),
    );
    r.update_conditional_register(i.dr_number());
}
pub fn st(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn sti(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn str(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn rti(_i: Instruction, _r: &Registers) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::registers::ConditionFlag;
    use googletest::prelude::*;

    #[gtest]
    pub fn test_opcode_add() {
        let mut regs = Registers::new();
        regs.set_binary(0, 22);
        regs.set_binary(1, 128);
        // Add: DR: 2, SR1: 0: 22, Immediate: false, SR2: 1: 128 => R2: 150
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        // Add: DR: 3, SR1: 2: 150, Immediate: true, imm5: 14 => R3: 164
        add(0b0001_011_010_1_01110.into(), &mut regs);
        expect_that!(regs.get_binary(0), eq(22));
        expect_that!(regs.get_binary(1), eq(128));
        expect_that!(regs.get_binary(2), eq(150));
        expect_that!(regs.get_binary(3), eq(164));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));
    }
    #[gtest]
    pub fn test_opcode_add_negative() {
        let mut regs = Registers::new();
        regs.set_binary(0, 22);
        regs.set_decimal(1, -128);
        // Add: DR: 2, SR1: 0: 22, Immediate: false, SR2: 1: -128 => R2: -106
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        // Add: DR: 3, SR1: 2: -106, Immediate: true, imm5: -2 => R3: -108
        add(0b0001_011_010_1_11110.into(), &mut regs);
        expect_that!(regs.get_binary(0), eq(22));
        expect_that!(regs.get_binary(1), eq(0b1111_1111_1000_0000));
        expect_that!(regs.get_binary(1).as_decimal(), eq(-128));
        expect_that!(regs.get_binary(2).as_decimal(), eq(-106));
        expect_that!(regs.get_binary(3).as_decimal(), eq(-108),);
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_add_underflow() {
        let mut regs = Registers::new();
        regs.set_binary(0, 0x7FFF); // largest positive number in 2's complement
        regs.set_binary(1, 1);
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 32768
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        expect_that!(regs.get_binary(0), eq(0x7FFF));
        expect_that!(regs.get_binary(1), eq(1));
        expect_that!(regs.get_binary(2), eq(32768));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_add_result_0() {
        let mut regs = Registers::new();
        regs.set_binary(0, 0x7FFF); // largest positive number in 2's complement
        regs.set_binary(1, !0x7FFF + 1);
        regs.set_binary(2, 1); // to be sure opcode was executed
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 0
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        expect_that!(regs.get_binary(0), eq(0x7FFF));
        expect_that!(regs.get_binary(1), eq(!0x7FFF + 1));
        expect_that!(regs.get_binary(2), eq(0));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Zero));
    }
    #[gtest]
    pub fn test_opcode_and() {
        let mut regs = Registers::new();
        regs.set_binary(0, 0b1101_1001_0111_0101);
        regs.set_binary(1, 0b0100_1010_0010_1001);
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 0
        and(0b0101_010_000_0_00_001.into(), &mut regs);
        expect_that!(regs.get_binary(0), eq(0b1101_1001_0111_0101));
        expect_that!(regs.get_binary(1), eq(0b0100_1010_0010_1001));
        expect_that!(regs.get_binary(2), eq(0b0100_1000_0010_0001));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));
    }
    #[gtest]
    pub fn test_opcode_and_immediate() {
        let mut regs = Registers::new();
        regs.set_binary(0, 0b1101_1001_0111_0101);
        // Add: DR: 2, SR1: 0, Immediate: true: 21, 0xFFF5 => R2: 0
        expect_that!(regs.get_binary(0), eq(0b1101_1001_0111_0101));
        // Immediate sign extended:           0b1111_1111_1111_0101
        and(0b0101_010_000_1_10101.into(), &mut regs);
        expect_that!(regs.get_binary(2), eq(0b1101_1001_0111_0101));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_not() {
        let mut regs = Registers::new();
        regs.set_binary(0, 0x7FFF); // largest positive number in 2's complement
        // Add: DR: 1, SR1: 0 => R1: 0xFFFE
        super::not(0b1001_001_000_111111.into(), &mut regs);
        expect_that!(regs.get_binary(0), eq(0x7FFF));
        expect_that!(regs.get_binary(1), eq(0x8000));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_lea() {
        let mut regs = Registers::new();
        regs.set_pc(0x3045);
        // Lea: DR: 3, SR1: 0 => R1: 0xFFFE
        lea(0b1110_011_0_0101_0101.into(), &mut regs);
        expect_that!(regs.get_binary(3), eq(0x3045 + 0b0_0101_0101));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));
    }
    #[gtest]
    pub fn test_opcode_ld() {
        let mut regs = Registers::new();
        regs.set_pc(0x3045);
        let raw = vec![4711u16, 815];
        let memory = Memory::with_program(&raw).expect("Error loading program");
        // LD - DR: 4, PC_OFFSET9: -0x44
        ld(0b0010_100_1_1011_1100.into(), &mut regs, &memory);
        expect_that!(regs.get_binary(4).as_decimal(), eq(815));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));

        // LD - DR: 4, PC_OFFSET9: -0x45
        ld(0b0010_100_1_1011_1011.into(), &mut regs, &memory);
        expect_that!(regs.get_binary(4).as_decimal(), eq(4711));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));
    }
}
