use crate::{
    execution::tests::ramless_emulator,
    instructions::{
        MultiplyInstruction, MultiplyLongInstruction, fields::Register,
    },
    memory::Endian,
};

// 32-bit Multiply (MUL, MLA)

#[test]
fn test_mul_simple() {
    // MUL R0, R1, R2  (R0 = R1 * R2)
    // Enc: Cond=AL, 0000000S, Rd=0, Rn=0, Rs=2, 1001, Rm=1
    // Hex: E0000291
    let instr = MultiplyInstruction::from(u32::from_be_bytes([
        0xE0, 0x00, 0x02, 0x91,
    ]));

    let mut emulator = ramless_emulator(Endian::Little);
    emulator.cpu.set_register(Register::R1 as _, 10);
    emulator.cpu.set_register(Register::R2 as _, 20);

    emulator.execute_multiply_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), 200);
}

#[test]
fn test_muls_flags() {
    // MULS R0, R1, R2 (Set flags)
    // Enc: E0100291 (S bit set)
    let instr = MultiplyInstruction::from(u32::from_be_bytes([
        0xE0, 0x10, 0x02, 0x91,
    ]));

    let mut emulator = ramless_emulator(Endian::Little);

    // Case 1: Zero Result
    emulator.cpu.set_register(Register::R1 as _, 0);
    emulator.cpu.set_register(Register::R2 as _, 50);
    emulator.cpu.set_z(false); // Pre-clear

    emulator.execute_multiply_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0);
    assert!(emulator.cpu.z(), "Z flag should be set");
    assert!(!emulator.cpu.n(), "N flag should be clear");

    // Case 2: Negative Result (Signed interpretation)
    // -1 * 1 = -1 (0xFFFFFFFF)
    emulator.cpu.set_register(Register::R1 as _, 0xFFFFFFFF);
    emulator.cpu.set_register(Register::R2 as _, 1);
    emulator.cpu.set_n(false); // Pre-clear

    emulator.execute_multiply_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0xFFFFFFFF);
    assert!(emulator.cpu.n(), "N flag should be set for negative result");
}

#[test]
fn test_mla_accumulate() {
    // MLA R0, R1, R2, R3 (R0 = R1 * R2 + R3)
    // Enc: Cond=AL, 0000001S, Rd=0, Rn=3, Rs=2, 1001, Rm=1
    // A bit (bit 21) is 1.
    // Hex: E0203291
    let instr = MultiplyInstruction::from(u32::from_be_bytes([
        0xE0, 0x20, 0x32, 0x91,
    ]));

    let mut emulator = ramless_emulator(Endian::Little);
    emulator.cpu.set_register(Register::R1 as _, 10);
    emulator.cpu.set_register(Register::R2 as _, 5);
    emulator.cpu.set_register(Register::R3 as _, 3); // Accumulator

    emulator.execute_multiply_instruction(instr).unwrap();

    // 10 * 5 + 3 = 53
    assert_eq!(emulator.cpu.register(Register::R0 as _), 53);
}

// 64-bit Multiply Long (UMULL, SMULL, UMLAL, SMLAL)

#[test]
fn test_umull_unsigned_long() {
    // UMULL R0, R1, R2, R3 (R1:R0 = R2 * R3)
    // R0 = RdLo, R1 = RdHi
    // Enc: Cond=AL, 0000100S, RdHi=1, RdLo=0, Rs=3, 1001, Rm=2
    // Hex: E0810392
    let instr = MultiplyLongInstruction::from(u32::from_be_bytes([
        0xE0, 0x81, 0x03, 0x92,
    ]));

    let mut emulator = ramless_emulator(Endian::Little);

    // Multiply 0x80000000 * 4
    // Expected: 0x200000000 (Fits in 34 bits)
    // Lo: 0, Hi: 2
    emulator.cpu.set_register(Register::R2 as _, 0x80000000);
    emulator.cpu.set_register(Register::R3 as _, 4);

    emulator.execute_multiply_long_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), 0); // Lo
    assert_eq!(emulator.cpu.register(Register::R1 as _), 2); // Hi
}

#[test]
fn test_smull_signed_long() {
    // SMULL R0, R1, R2, R3 (R1:R0 = R2 * R3) signed
    // Enc: Cond=AL, 0000110S, RdHi=1, RdLo=0, Rs=3, 1001, Rm=2
    // U bit (bit 22) is 1 (Signed).
    // Hex: E0C10392
    let instr = MultiplyLongInstruction::from(u32::from_be_bytes([
        0xE0, 0xC1, 0x03, 0x92,
    ]));

    let mut emulator = ramless_emulator(Endian::Little);

    // Multiply -1 (0xFFFFFFFF) * 5
    // Expected: -5
    // 64-bit hex: 0xFFFFFFFFFFFFFFFB
    // Lo: 0xFFFFFFFB, Hi: 0xFFFFFFFF
    emulator.cpu.set_register(Register::R2 as _, 0xFFFFFFFF);
    emulator.cpu.set_register(Register::R3 as _, 5);

    emulator.execute_multiply_long_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), 0xFFFFFFFB); // Lo
    assert_eq!(emulator.cpu.register(Register::R1 as _), 0xFFFFFFFF); // Hi
}

#[test]
fn test_umlal_accumulate_long() {
    // UMLAL R0, R1, R2, R3 (R1:R0 = R2 * R3 + R1:R0)
    // Enc: Cond=AL, 0000101S, RdHi=1, RdLo=0, Rs=3, 1001, Rm=2
    // A bit (bit 21) is 1.
    // Hex: E0A10392
    let instr = MultiplyLongInstruction::from(u32::from_be_bytes([
        0xE0, 0xA1, 0x03, 0x92,
    ]));

    let mut emulator = ramless_emulator(Endian::Little);

    // Initial value in R1:R0 = 0x00000000_00000005
    emulator.cpu.set_register(Register::R0 as _, 5);
    emulator.cpu.set_register(Register::R1 as _, 0);

    // Multiply 10 * 10 = 100
    emulator.cpu.set_register(Register::R2 as _, 10);
    emulator.cpu.set_register(Register::R3 as _, 10);

    emulator.execute_multiply_long_instruction(instr).unwrap();

    // Result: 100 + 5 = 105
    assert_eq!(emulator.cpu.register(Register::R0 as _), 105);
    assert_eq!(emulator.cpu.register(Register::R1 as _), 0);
}
