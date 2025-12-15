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
        r.set_pc(address_by_pc_offset(i, r));
    }
}
/// JSR: Jump to Sub-Routine.
/// Two variants:
/// - JSR to `PCOffset11`
/// ```text
///  15__12__11_10_________0
/// | 0100 | 1 | PCOffset11 |
///  -----------------------
/// ```
/// - JSRR: JSR to location in `BaseR`
/// ```text
///  15__12__11_9__8___6___5____0_
/// | 0100 | 000 | BaseR | 000000 |
///  -----------------------------
/// ```
/// The former PC is saved in R7.
pub fn jsr(i: Instruction, r: &mut Registers) {
    let temp_pc = r.pc();
    r.set_pc(if i.get_bit_range(11, 11) == 1 {
        (r.pc().as_decimal() + i.pc_offset(11)).cast_unsigned()
    } else {
        r.get(i.get_bit_range_u8(6, 8, "Error in JSR")).as_binary()
    });
    r.set(7, temp_pc);
}
/// JMP or RET operation.
/// - JMP sets the PC to the value of register `BaseR`
/// ```text
///  15__12__11_9___8_6____5____0_
/// | 1100 | 000 | BaseR | 000000 |
///  -----------------------------
/// ```
/// - RET same as JMP, but special case for returning from JSR where former PC is saved in R7.
/// ```text
///  15__12__11_9__8_6___5____0_
/// | 1100 | 000 | 111 | 000000 |
///  ---------------------------
/// ```
pub fn jmp_or_ret(i: Instruction, r: &mut Registers) {
    r.set_pc(
        r.get(i.get_bit_range_u8(6, 8, "Error in jmp_or_ret"))
            .as_binary(),
    );
}

/// LD: Loads content of memory address of PC + sign extended offset into DR.
/// ```text
///  15__12__11_9___8_______0_
/// | 0010 |  DR  | PCoffset9 |
///  -------------------------
/// ```
pub fn ld(i: Instruction, r: &mut Registers, memory: &Memory) {
    let value = memory[address_by_pc_offset(i, r)];
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
    let address_address = address_by_pc_offset(i, r);
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
    let value_address = address_by_baser_offset(i, r);
    r.set(i.dr_number(), from_binary(memory[value_address]));
    r.update_conditional_register(i.dr_number());
}

fn address_by_pc_offset(i: Instruction, r: &Registers) -> u16 {
    let address = r.pc().as_decimal() + i.pc_offset(9);
    address.cast_unsigned()
}
fn address_by_baser_offset(i: Instruction, r: &Registers) -> u16 {
    let base_r = i.get_bit_range_u8(6, 8, "Error in address_by_baser_offset");
    (r.get(base_r).as_decimal() + i.pc_offset(6)).cast_unsigned()
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
        Register::from_binary(address_by_pc_offset(i, r)),
    );
    r.update_conditional_register(i.dr_number());
}
/// ST: Store. The contents of the SR are written to memory address PC + sign extended offset.
/// ```text
///  15__12__11_9___8_______0_
/// | 0011 |  SR  | PCoffset9 |
///  -------------------------
/// ```
pub fn st(i: Instruction, r: &Registers, memory: &mut Memory) {
    let store_address = address_by_pc_offset(i, r);
    memory[store_address] = r.get(i.dr_number()).as_binary();
}
/// STI: Store Indirect. The contents of the SR are written to the address which is loaded from
/// memory address PC + sign extended offset.
/// ```text
///  15__12__11_9___8_______0_
/// | 1011 |  SR  | PCoffset9 |
///  -------------------------
/// ```
pub fn sti(i: Instruction, r: &Registers, memory: &mut Memory) {
    let address_of_store_address = address_by_pc_offset(i, r);
    let store_address = memory[address_of_store_address];
    memory[store_address] = r.get(i.dr_number()).as_binary();
}
/// STR: Store contents of SR to memory address of base register plus sign extended offset.
/// ```text
///  15__12__11_9__8___6____5____0_
/// | 0111 |  SR | BaseR | offset6 |
///  ------------------------------
/// ```
pub fn str(i: Instruction, r: &Registers, memory: &mut Memory) {
    let store_address = address_by_baser_offset(i, r);
    memory[store_address] = r.get(i.dr_number()).as_binary();
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
    use crate::emulator::test_helpers::FakeKeyboardInputProvider;
    use crate::hardware::registers::{ConditionFlag, from_decimal};
    use googletest::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn create_memory(data: &[u16]) -> Memory {
        let kip = FakeKeyboardInputProvider::new("");
        let mut mem = Memory::new(Rc::new(RefCell::new(kip)));
        mem.load_program(data).expect("Error loading program");
        mem
    }

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
        let memory = create_memory(&raw);
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
        let memory = create_memory(&raw);
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
        let memory = create_memory(&raw);
        regs.set_pc(0x3065);
        // LDR - DR: 1, - PC_OFFSET9: -96 = -0x60
        ldi(0b1010_001_110100000.into(), &mut regs, &memory);
        expect_that!(regs.get(1), eq(from_binary(val_to_load_in_register)));
        expect_that!(regs.get_conditional_register(), eq(ConditionFlag::Neg));
    }
    #[gtest]
    pub fn test_opcode_st() {
        let mut regs = Registers::new();
        let raw = vec![0; 0xC4];
        let mut memory = create_memory(&raw);
        regs.set(5, from_decimal(4760));
        regs.set_pc(0x3065);
        // ST - SR: 5, - PC_OFFSET9: -95 = -0x5F
        st(0b0011_101_110100001.into(), &regs, &mut memory);
        expect_that!(memory[0x3006], eq(4760));
    }
    #[gtest]
    pub fn test_opcode_sti() {
        let mut regs = Registers::new();
        let raw = vec![0; 0xC4];
        let mut memory = create_memory(&raw);
        memory[0x300A] = 0x3006;
        regs.set(7, from_decimal(1234));
        regs.set_pc(0x3067);
        // STI - SR: 7, - PC_OFFSET9: -0x5D
        sti(0b1011_111_110100011.into(), &regs, &mut memory);
        expect_that!(memory[0x3006], eq(1234));
    }
    #[gtest]
    pub fn test_opcode_str() {
        let mut regs = Registers::new();
        let raw = vec![0; 0xC4];
        let mut memory = create_memory(&raw);
        regs.set(2, from_decimal(2345));
        regs.set(6, from_binary(0x3005));
        // STR - SR: 2, - BaseR: 6, offset6: 0x1
        str(0b0111_010_110_000001.into(), &regs, &mut memory);
        expect_that!(memory[0x3006], eq(2345));
    }
    #[gtest]
    pub fn test_opcode_jsr() {
        let mut regs = Registers::new();
        regs.set_pc(0x3099);
        // JSR - PC_OFFSET11: 0x1A1
        jsr(0b0100_1_00110100001.into(), &mut regs);
        expect_that!(regs.pc(), eq(from_decimal(0x323A)));
        expect_that!(regs.get(7), eq(from_decimal(0x3099)));

        let mut regs = Registers::new();
        regs.set_pc(0x3100);
        regs.set(6, from_decimal(0x3456));
        // JSR - BaseR: 6
        jsr(0b0100_000_110_000000.into(), &mut regs);
        expect_that!(regs.pc(), eq(from_decimal(0x3456)));
        expect_that!(regs.get(7), eq(from_decimal(0x3100)));
    }
    #[gtest]
    pub fn test_opcode_ret() {
        let mut regs = Registers::new();
        regs.set_pc(0x3020);
        regs.set(1, from_decimal(0x3022));
        // JMP - BaseR: 1
        jmp_or_ret(0b1100_000_001_000000.into(), &mut regs);
        expect_that!(regs.pc(), eq(from_decimal(0x3022)));
    }
}
