use crate::{
    execution::tests::ramless_emulator,
    instructions::{DataProcessingInstruction, fields::Register},
    memory::Endian,
    memory::little_endian_to_native,
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
fn test_subs_with_overflow() {
    // SUBS R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0510002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    emulator.cpu.set_register(Register::R1 as _, 0x80000000);
    emulator.cpu.set_register(Register::R2 as _, 1);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0x7FFFFFFF);
    assert!(emulator.cpu.c(), "C flag should be set");
    assert!(emulator.cpu.v(), "V flag should be set");
    assert!(!emulator.cpu.n(), "N flag should not be set");
    assert!(!emulator.cpu.z(), "Z flag should not be set");
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

#[test]
fn test_mvn_move_not() {
    // MVN R0, #0
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE3F00000,
    ));
    let mut emulator = ramless_emulator(Endian::Little);
    emulator.execute_data_processing_instruction(instr).unwrap();

    // NOT 0 = 0xFFFFFFFF
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0xFFFFFFFF);
}

#[test]
fn test_mvns_flags_and_shifter_carry() {
    // MVNS R0, R1, ROR #1
    // The key is that the carry flag is set by the SHIFTER, not the NOT operation.
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE1F100E1,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // Use a value where ROR #1 will definitely produce a carry of 1.
    // The last bit shifted out from ROR #1 is bit 0.
    emulator.cpu.set_register(Register::R1 as _, 0x00000001); // Bit 0 is 1
    emulator.cpu.set_c(false); // Pre-clear carry
    emulator.execute_data_processing_instruction(instr).unwrap();

    // The shifter does: 0x00000001 ROR 1 -> 0x80000000, with carry_out = 1
    let shifted_val = 0x80000000;
    let final_result = !shifted_val; // 0x7FFFFFFF

    assert_eq!(emulator.cpu.register(Register::R0 as _), final_result);
    assert!(
        emulator.cpu.c(),
        "C flag should be set to 1 by the shifter's ROR carry-out"
    );
    assert!(
        !emulator.cpu.n(),
        "N flag should be clear for the positive result"
    );
    assert!(!emulator.cpu.z(), "Z flag should be clear");
}

#[test]
fn test_cmp_compare_equal() {
    // CMP R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE1510002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // Set R0 to a known value to ensure it's not written to.
    emulator.cpu.set_register(Register::R0 as _, 999);
    emulator.cpu.set_register(Register::R1 as _, 50);
    emulator.cpu.set_register(Register::R2 as _, 50);

    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(
        emulator.cpu.register(Register::R0 as _),
        999,
        "CMP should not write to any GPR"
    );
    assert!(
        emulator.cpu.z(),
        "Z flag should be set for equal comparison"
    );
    assert!(
        emulator.cpu.c(),
        "C flag should be 1 (no borrow) for equal comparison"
    );
    assert!(!emulator.cpu.n(), "N flag should be clear");
    assert!(!emulator.cpu.v(), "V flag should be clear");
}

#[test]
fn test_cmn_compare_negative() {
    // CMN R1, R2  (Compare Negative, effectively CMP R1, -R2 or TST R1+R2)
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE1710002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // Test for a negative result: 50 + 1 = 51 (positive)
    emulator.cpu.set_register(Register::R1 as _, 50);
    emulator.cpu.set_register(Register::R2 as _, 1);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert!(!emulator.cpu.n(), "N flag should be clear for 50+1");

    // Test for a negative result: 50 + (-60) = -10
    emulator.cpu.set_register(Register::R1 as _, 50);
    emulator.cpu.set_register(Register::R2 as _, -60i32 as u32);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert!(emulator.cpu.n(), "N flag should be set for 50+(-60)");
}

#[test]
fn test_tst_test_bits() {
    // TST R1, #0b1000   (Test if bit 3 of R1 is set)
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE3110008,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // Test when bit is NOT set
    emulator.cpu.set_register(Register::R1 as _, 0b0101);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert!(
        emulator.cpu.z(),
        "Z flag should be set when tested bits are clear"
    );

    // Test when bit IS set
    emulator.cpu.set_register(Register::R1 as _, 0b1101);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert!(
        !emulator.cpu.z(),
        "Z flag should be clear when tested bits are set"
    );
}

#[test]
fn test_teq_test_equivalence() {
    // TEQ R1, R2  (Test Equivalence is a non-writing EOR)
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE1310002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // Test when R1 == R2 (EOR result is 0)
    emulator.cpu.set_register(Register::R1 as _, 123);
    emulator.cpu.set_register(Register::R2 as _, 123);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert!(
        emulator.cpu.z(),
        "Z flag should be set when operands are equal"
    );

    // Test when R1 != R2
    emulator.cpu.set_register(Register::R1 as _, 123);
    emulator.cpu.set_register(Register::R2 as _, 456);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert!(
        !emulator.cpu.z(),
        "Z flag should be clear when operands are not equal"
    );
}

#[test]
fn test_adc_add_with_carry() {
    // ADC R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0A10002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    emulator.cpu.set_register(Register::R1 as _, 10);
    emulator.cpu.set_register(Register::R2 as _, 20);

    // Test with C=0
    emulator.cpu.set_c(false);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 30);

    // Test with C=1
    emulator.cpu.set_c(true);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 31);
}

#[test]
fn test_sbc_subtract_with_carry() {
    // SBCS R0, R1, R2
    let instr = DataProcessingInstruction::from(little_endian_to_native(
        0xE0D10002,
    ));
    let mut emulator = ramless_emulator(Endian::Little);

    // Test 10 - 5 with C=1 (no borrow). Result should be 10 - 5 - (1-1) = 5
    emulator.cpu.set_register(Register::R1 as _, 10);
    emulator.cpu.set_register(Register::R2 as _, 5);
    emulator.cpu.set_c(true);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 5);
    assert!(emulator.cpu.c(), "C should be 1 (no borrow)");

    // Test 10 - 5 with C=0 (borrow). Result should be 10 - 5 - (1-0) = 4
    emulator.cpu.set_c(false);
    emulator.execute_data_processing_instruction(instr).unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 4);
    assert!(emulator.cpu.c(), "C should still be 1 (no borrow)");
}
