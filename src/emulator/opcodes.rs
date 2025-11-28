//! Implemented operations for the LC 3.
use crate::emulator::instruction::Instruction;
use crate::hardware::memory::Memory;
use crate::hardware::registers::{ConditionFlag, Register, Registers, from_binary};

/// ADD: Mathematical addition in 2 variants
/// - DR is set with result of SR 1 + SR 2
/// ```text
///  15__12__11_9__8_6___5___4_3__2_0_
/// | 0001 |  DR | SR1 | 0 | 00 | SR2 |
///  ---------------------------------
/// ```
/// - DR is set with result of SR 1 + sign extended immediate
/// ```text
///  15__12__11_9__8_6___5___4___0_
/// | 0001 |  DR | SR1 | 1 |  IMM5 |
///  ------------------------------
/// ```
#[allow(
    clippy::cast_possible_truncation,
    reason = "truncation is what is specified for the LC-3 add opcode"
)]
pub fn add(i: Instruction, r: &mut Registers) {
    r.set(
        i.dr_number(),
        from_binary(
            (r.get(i.sr1_number()).as_binary_u32()
                + (if i.is_immediate() {
                    u32::from(i.get_immediate())
                } else {
                    r.get(i.sr2_number()).as_binary_u32()
                })) as u16,
        ),
    );
    r.update_conditional_register(i.dr_number());
}
/// AND: bit-wise AND in 2 variants
/// - DR is set with result of SR 1 AND SR 2
/// ```text
///  15__12__11_9__8_6___5___4_3__2_0_
/// | 0101 |  DR | SR1 | 0 | 00 | SR2 |
///  ---------------------------------
/// ```
/// - DR is set with result of SR 1 AND sign extended immediate
/// ```text
///  15__12__11_9__8_6___5___4___0_
/// | 0101 |  DR | SR1 | 1 |  IMM5 |
///  ------------------------------
/// ```
pub fn and(i: Instruction, r: &mut Registers) {
    r.set(
        i.dr_number(),
        from_binary(
            r.get(i.sr1_number()).as_binary()
                & (if i.is_immediate() {
                    i.get_immediate()
                } else {
                    r.get(i.sr2_number()).as_binary()
                }),
        ),
    );
    r.update_conditional_register(i.dr_number());
}

/// NOT: bit-wise complement of the value in SR 1
/// ```text
///  15__12__11_9__8_6___5___0_
/// | 1001 |  DR | SR1 | 11111 |
///  --------------------------
/// ```
pub fn not(i: Instruction, r: &mut Registers) {
    r.set(
        i.dr_number(),
        from_binary(!r.get(i.sr1_number()).as_binary()),
    );
    r.update_conditional_register(i.dr_number());
}
/// BR: Conditional Branch
/// This opcode adds the value of the sign extended offset to PC if
/// - either none of the `nzp` bits are set
/// - or the current state of the `ConditionFlag` matches a set bit of `n`, `z` or `p`.
/// ```text
///  15__12__11_9___8_______0_
/// | 0000 |  nzp | PCoffset9 |
///  -------------------------
/// ```
/// See [`ConditionFlag`]
pub fn br(i: Instruction, r: &mut Registers) {
    let none_set = i.get_bit_range(9, 11) == 0;
    let do_break = none_set
        || match r.get_conditional_register() {
            ConditionFlag::Pos => i.get_bit(9),
            ConditionFlag::Zero => i.get_bit(10),
            ConditionFlag::Neg => i.get_bit(11),
        };
    if do_break {
        r.set_pc(address_by_offset(r.pc(), i.pc_offset(9)));
    }
}
pub fn jmp_or_ret(_i: Instruction, _r: &Registers) {
    todo!()
}
pub fn jsr(_i: Instruction, _r: &Registers) {
    todo!()
}
/// LD: Loads content of memory address of PC + sign extended offset into DR.
/// ```text
///  15__12__11_9___8_______0_
/// | 0010 |  DR  | PCoffset9 |
///  -------------------------
/// ```
pub fn ld(i: Instruction, r: &mut Registers, memory: &Memory) {
    let value = memory[address_by_offset(r.pc(), i.pc_offset(9))];
    r.set(i.dr_number(), from_binary(value));
    r.update_conditional_register(i.dr_number());
}

/// LDI: Load indirect.
/// Calculates memory address of PC + sign extended offset and reads another address from there,
/// the content of the memory at that indirectly loaded address is put into DR.
/// ```text
///  15__12__11_9___8_______0_
/// | 1010 |  DR  | PCoffset9 |
///  -------------------------
/// ```
pub fn ldi(i: Instruction, r: &mut Registers, memory: &Memory) {
    let address_address = address_by_offset(r.pc(), i.pc_offset(9));
    let value_address = memory[address_address];
    r.set(i.dr_number(), from_binary(memory[value_address]));
    r.update_conditional_register(i.dr_number());
}
/// LDR: Load address from base register and adds sign extended offset to load the memory content
/// from there into DR.
/// ```text
///  15__12__11_9__8___6____5____0_
/// | 0110 |  DR | BaseR | offset6 |
///  ------------------------------
/// ```
pub fn ldr(i: Instruction, r: &mut Registers, memory: &Memory) {
    let base_register = i.get_bit_range_u8(6, 8, "Error in ldr");
    let value_address = address_by_offset(r.get(base_register), i.pc_offset(6));
    r.set(i.dr_number(), from_binary(memory[value_address]));
    r.update_conditional_register(i.dr_number());
}

fn address_by_offset(r: Register, offset: i16) -> u16 {
    let address = r.as_decimal() + offset;
    address.cast_unsigned()
}

/// LEA: Load Effective Address loads PC + sign extended offset into DR.
/// ```text
///  15__12__11_9___8_______0_
/// | 1110 |  DR  | PCoffset9 |
///  -------------------------
/// ```
pub fn lea(i: Instruction, r: &mut Registers) {
    r.set(
        i.dr_number(),
        Register::from_binary(address_by_offset(r.pc(), i.pc_offset(9))),
    );
    r.update_conditional_register(i.dr_number());
}
/// ST: Store. The contents of the SR are written to memory address PC + sign extended offset.
/// ```text
///  15__12__11_9___8_______0_
/// | 0011 |  SR  | PCoffset9 |
///  -------------------------
/// ```
pub fn st(_i: Instruction, _r: &Registers) {
    todo!()
}
/// STI: Store Indirect. The contents of the SR are written to the address which is loaded from
/// memory address PC + sign extended offset.
/// ```text
///  15__12__11_9___8_______0_
/// | 1011 |  SR  | PCoffset9 |
///  -------------------------
/// ```
pub fn sti(_i: Instruction, _r: &Registers) {
    todo!()
}
/// STR: Store contents of SR to memory address of base register plus sign extended offset.
/// ```text
///  15__12__11_9__8___6____5____0_
/// | 0111 |  SR | BaseR | offset6 |
///  ------------------------------
/// ```
pub fn str(_i: Instruction, _r: &Registers) {
    todo!()
}
/// RTI: Return from Interrupt.
/// If the processor is running in Supervisor mode, the top two elements on the
/// Supervisor Stack are popped and loaded into PC, PSR. If the processor is running
/// in User mode, a privilege mode violation exception occurs.
/// ```text
///  15__12__11_____________0_
/// | 1000 | 0000000000000000 |
///  -------------------------
/// ```
pub fn rti(_i: Instruction, _r: &Registers) {
    todo!()
}

#[expect(clippy::unusual_byte_groupings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::registers::{ConditionFlag, from_decimal};
    use googletest::prelude::*;

    #[gtest]
    pub fn test_opcode_add() {
        let mut regs = Registers::new();
        regs.set(0, from_binary(22));
        regs.set(1, from_binary(128));
        // Add: DR: 2, SR1: 0: 22, Immediate: false, SR2: 1: 128 => R2: 150
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        // Add: DR: 3, SR1: 2: 150, Immediate: true, imm5: 14 => R3: 164
        add(0b0001_011_010_1_01110.into(), &mut regs);
        expect_that!(regs.get(0), eq(from_binary(22)));
        expect_that!(regs.get(1), eq(from_binary(128)));
        expect_that!(regs.get(2), eq(from_binary(150)));
        expect_that!(regs.get(3), eq(from_binary(164)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));
    }
    #[gtest]
    pub fn test_opcode_add_negative() {
        let mut regs = Registers::new();
        regs.set(0, from_binary(22));
        regs.set(1, from_decimal(-128));
        // Add: DR: 2, SR1: 0: 22, Immediate: false, SR2: 1: -128 => R2: -106
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        // Add: DR: 3, SR1: 2: -106, Immediate: true, imm5: -2 => R3: -108
        add(0b0001_011_010_1_11110.into(), &mut regs);
        expect_that!(regs.get(0), eq(from_binary(22)));
        expect_that!(regs.get(1), eq(from_binary(0b1111_1111_1000_0000)));
        expect_that!(regs.get(1), eq(from_decimal(-128)));
        expect_that!(regs.get(2).as_decimal(), eq(-106));
        expect_that!(regs.get(3).as_decimal(), eq(-108),);
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_add_underflow() {
        let mut regs = Registers::new();
        regs.set(0, from_binary(0x7FFF)); // largest positive number in 2's complement
        regs.set(1, from_binary(1));
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 32768
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        expect_that!(regs.get(0), eq(from_binary(0x7FFF)));
        expect_that!(regs.get(1), eq(from_binary(1)));
        expect_that!(regs.get(2), eq(from_binary(32768)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_add_result_0() {
        let mut regs = Registers::new();
        regs.set(0, from_binary(0x7FFF)); // largest positive number in 2's complement
        regs.set(1, from_binary(!0x7FFF + 1));
        regs.set(2, from_binary(1)); // to be sure opcode was executed
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 0
        add(0b0001_010_000_0_00_001.into(), &mut regs);
        expect_that!(regs.get(0), eq(from_binary(0x7FFF)));
        expect_that!(regs.get(1), eq(from_binary(!0x7FFF + 1)));
        expect_that!(regs.get(2), eq(from_binary(0)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Zero));
    }
    #[gtest]
    pub fn test_opcode_and() {
        let mut regs = Registers::new();
        regs.set(0, from_binary(0b1101_1001_0111_0101));
        regs.set(1, from_binary(0b0100_1010_0010_1001));
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 0
        and(0b0101_010_000_0_00_001.into(), &mut regs);
        expect_that!(regs.get(0), eq(from_binary(0b1101_1001_0111_0101)));
        expect_that!(regs.get(1), eq(from_binary(0b0100_1010_0010_1001)));
        expect_that!(regs.get(2), eq(from_binary(0b0100_1000_0010_0001)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));
    }
    #[gtest]
    pub fn test_opcode_and_immediate() {
        let mut regs = Registers::new();
        regs.set(0, from_binary(0b1101_1001_0111_0101));
        // Add: DR: 2, SR1: 0, Immediate: true: 21, 0xFFF5 => R2: 0
        expect_that!(regs.get(0), eq(from_binary(0b1101_1001_0111_0101)));
        // Immediate sign extended:           0b1111_1111_1111_0101
        and(0b0101_010_000_1_10101.into(), &mut regs);
        expect_that!(regs.get(2), eq(from_binary(0b1101_1001_0111_0101)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_not() {
        let mut regs = Registers::new();
        regs.set(0, from_binary(0x7FFF)); // largest positive number in 2's complement
        // Add: DR: 1, SR1: 0 => R1: 0xFFFE
        super::not(0b1001_001_000_111111.into(), &mut regs);
        expect_that!(regs.get(0), eq(from_binary(0x7FFF)));
        expect_that!(regs.get(1), eq(from_binary(0x8000)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_lea() {
        let mut regs = Registers::new();
        regs.set_pc(0x3045);
        // Lea: DR: 3, SR1: 0 => R1: 0xFFFE
        lea(0b1110_011_0_0101_0101.into(), &mut regs);
        expect_that!(regs.get(3), eq(from_binary(0x3045 + 0b0_0101_0101)));
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
        expect_that!(regs.get(4), eq(from_decimal(815)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));

        // LD - DR: 4, PC_OFFSET9: -0x45
        ld(0b0010_100_1_1011_1011.into(), &mut regs, &memory);
        expect_that!(regs.get(4), eq(from_decimal(4711)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Pos));
    }
    #[gtest]
    pub fn test_opcode_ldr() {
        let mut regs = Registers::new();
        let mut raw = vec![0; 6];
        let mem_val = 0b1111_1111_1111_0110; // -10
        raw[5] = mem_val;
        let memory = Memory::with_program(&raw).expect("Error loading program");
        regs.set(6, from_binary(0x3025));
        // LDR - DR: 2, - BaseR: 6, OFFSET6: -32 = -0x20
        ldr(0b0110_010_110_100000.into(), &mut regs, &memory);
        expect_that!(regs.get(2), eq(from_binary(mem_val)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_ldi() {
        let mut regs = Registers::new();
        let mut raw = vec![0; 10];
        let val_to_load_in_register = 0b1111_1111_1111_0110; // -10
        raw[3] = val_to_load_in_register;
        raw[5] = 0x3003; // absolute address of value above
        let memory = Memory::with_program(&raw).expect("Error loading program");
        regs.set_pc(0x3065);
        // LDR - DR: 1, - PC_OFFSET9: -96 = -0x60
        ldi(0b1010_001_110100000.into(), &mut regs, &memory);
        expect_that!(regs.get(1), eq(from_binary(val_to_load_in_register)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
}
