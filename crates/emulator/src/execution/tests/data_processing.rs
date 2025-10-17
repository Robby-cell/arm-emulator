use crate::{
    execution::tests::ramless_emulator,
    instructions::{DataProcessingInstruction, fields::Register},
    memory::Endian,
    testing::little_endian_to_native,
};

#[test]
fn simple_mov_test_with_immediate() {
    // MOV R0, #45
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE3A0002D,
    ));
    let mut emulator = ramless_emulator(Endian::Little);
    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), 45);
}

#[test]
fn simple_mov_test_with_shifted_register() {
    // MOV R0, R1, LSL #2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE1A00101,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    emulator.cpu.set_register(Register::R1 as _, 16);
    assert_eq!(emulator.cpu.register(Register::R1 as _), 16);

    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), 16 << 2);
}

// --- Logical ---
#[test]
fn test_and_simple_register() {
    // AND R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0010002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    emulator.cpu.set_register(Register::R1 as _, 0b1100);
    emulator.cpu.set_register(Register::R2 as _, 0b1010);
    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), 0b1000);
}

#[test]
fn test_ands_flags_zero_and_negative() {
    // ANDS R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0110002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // --- Test for Zero Flag ---
    emulator.cpu.set_register(Register::R1 as _, 0b1010);
    emulator.cpu.set_register(Register::R2 as _, 0b0101);
    emulator.cpu.set_z(false); // Pre-clear the flag
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0);
    assert!(emulator.cpu.z(), "Z flag should be set for zero result");
    assert!(!emulator.cpu.n(), "N flag should be clear");

    // --- Test for Negative Flag ---
    let negative_val = 1 << 31; // 0x80000000
    emulator.cpu.set_register(Register::R1 as _, negative_val);
    emulator.cpu.set_register(Register::R2 as _, negative_val);
    emulator.cpu.set_n(false); // Pre-clear the flag
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), negative_val);
    assert!(!emulator.cpu.z(), "Z flag should be clear");
    assert!(emulator.cpu.n(), "N flag should be set for negative result");
}

#[test]
fn test_eors_carry_flag_from_shifter() {
    // EORS R0, R1, R2, LSR #1
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE03100A2,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // Set R2 to a value where bit 0 is 1. LSR #1 will cause a carry-out of 1.
    emulator.cpu.set_register(Register::R1 as _, 0);
    emulator.cpu.set_register(Register::R2 as _, 0b0101); // Ends in 1

    // Pre-condition: Ensure C flag is 0 so we can test that it gets set to 1.
    emulator.cpu.set_c(false);
    assert!(!emulator.cpu.c(), "C flag should be clear initially");

    emulator.execute_data_processing_instruction(instr).unwrap();

    // The result of the EOR is 0 ^ (5 >> 1) = 0 ^ 2 = 2
    assert_eq!(emulator.cpu.register(Register::R0 as _), 2);
    // CRITICAL: The C flag should be set by the shifter, not the EOR result.
    assert!(
        emulator.cpu.c(),
        "C flag should be set by the shifter's carry-out"
    );
}

// --- Arithmetic ---
#[test]
fn test_add_with_immediate() {
    // ADD R3, R4, #50
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE2843032,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    emulator.cpu.set_register(Register::R4 as _, 100);
    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R3 as _), 150);
}

#[test]
fn test_sub_does_not_alter_flags() {
    // SUB R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0410002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);
    emulator.cpu.set_register(Register::R1 as _, 20);
    emulator.cpu.set_register(Register::R2 as _, 5);

    // Pre-set the flags to a known state (all true)
    emulator.cpu.set_n(true);
    emulator.cpu.set_z(true);
    emulator.cpu.set_c(true);
    emulator.cpu.set_v(true);

    // Execute the non-flag-setting instruction
    emulator.execute_data_processing_instruction(instr).unwrap();

    // --- Assert the result is correct ---
    assert_eq!(emulator.cpu.register(Register::R0 as _), 15);

    // --- Assert that NO flags were changed ---
    assert!(
        emulator.cpu.n(),
        "N flag should NOT have been changed by SUB"
    );
    assert!(
        emulator.cpu.z(),
        "Z flag should NOT have been changed by SUB"
    );
    assert!(
        emulator.cpu.c(),
        "C flag should NOT have been changed by SUB"
    );
    assert!(
        emulator.cpu.v(),
        "V flag should NOT have been changed by SUB"
    );
}

#[test]
fn test_adds_flags_carry_and_overflow() {
    // ADDS R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0910002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // --- Test for C (unsigned carry) but not V (signed overflow) ---
    // 0xFFFFFFFF + 1 = 0, with a carry.
    emulator.cpu.set_register(Register::R1 as _, 0xFFFFFFFF);
    emulator.cpu.set_register(Register::R2 as _, 1);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0);
    assert!(emulator.cpu.z(), "Z flag should be set");
    assert!(
        emulator.cpu.c(),
        "C flag should be set on unsigned overflow"
    );
    assert!(!emulator.cpu.v(), "V flag should NOT be set");

    // --- Test for V (signed overflow) but not C (unsigned carry) ---
    // (MAX_I32) + 1 = (MIN_I32), which is a signed overflow.
    emulator.cpu.set_register(Register::R1 as _, 0x7FFFFFFF);
    emulator.cpu.set_register(Register::R2 as _, 1);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0x80000000);
    assert!(emulator.cpu.n(), "N flag should be set");
    assert!(!emulator.cpu.c(), "C flag should NOT be set");
    assert!(emulator.cpu.v(), "V flag should be set on signed overflow");
}

#[test]
fn test_subs_carry_flag_as_borrow() {
    // SUBS R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0510002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // --- Test for C=1 (no borrow) when Rn >= Rm ---
    emulator.cpu.set_register(Register::R1 as _, 10);
    emulator.cpu.set_register(Register::R2 as _, 5);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 5);
    assert!(emulator.cpu.c(), "C flag should be 1 (no borrow) for 10-5");

    // --- Test for C=0 (borrow) when Rn < Rm ---
    emulator.cpu.set_register(Register::R1 as _, 5);
    emulator.cpu.set_register(Register::R2 as _, 10);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _) as i32, -5);
    assert!(!emulator.cpu.c(), "C flag should be 0 (borrow) for 5-10");
    assert!(emulator.cpu.n(), "N flag should be set");
}

#[test]
fn test_rsb_reverse_subtract() {
    // RSB R0, R1, #100   (Reverse Subtract: R0 = 100 - R1)
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE2610064,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    emulator.cpu.set_register(Register::R1 as _, 40);
    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), 60);
}

#[test]
fn test_rsbs_reverse_subtract_with_flags() {
    // RSBS R0, R1, #100   (R0 = 100 - R1, and set flags)
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE2710064,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // --- Test for C=1 (no borrow) when #100 >= R1 ---
    emulator.cpu.set_register(Register::R1 as _, 40);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 60);
    assert!(
        emulator.cpu.c(),
        "C flag should be 1 (no borrow) for 100-40"
    );
    assert!(!emulator.cpu.z(), "Z flag should be clear");
    assert!(!emulator.cpu.n(), "N flag should be clear");

    // --- Test for C=0 (borrow) when #100 < R1 ---
    emulator.cpu.set_register(Register::R1 as _, 120);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _) as i32, -20);
    assert!(!emulator.cpu.c(), "C flag should be 0 (borrow) for 100-120");
    assert!(emulator.cpu.n(), "N flag should be set for negative result");
    assert!(!emulator.cpu.z(), "Z flag should be clear");

    // --- Test for Z=1 (zero result) when #100 == R1 ---
    emulator.cpu.set_register(Register::R1 as _, 100);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0);
    assert!(emulator.cpu.z(), "Z flag should be set for zero result");
    assert!(
        emulator.cpu.c(),
        "C flag should be 1 (no borrow) for 100-100"
    );
    assert!(!emulator.cpu.n(), "N flag should be clear");
}
