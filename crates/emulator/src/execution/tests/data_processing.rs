//! Note: <InstructionType>::from(u32) is not affected by endianness.

use crate::{
    Emulator,
    cpu::Cpu,
    instructions::{DataProcessingInstruction, fields::Register},
    memory::{Bus, Endian},
};

fn ramless_emulator(endian: Endian) -> Emulator {
    Emulator::new(Cpu::new(), Bus::new(0), endian)
}

#[test]
fn simple_mov_test_with_immediate() {
    // MOV R0, #45
    let instr = DataProcessingInstruction::from(0xE3A0002D);
    let mut emulator = ramless_emulator(Endian::Little);
    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(emulator.cpu[Register::R0 as _], 45);
}

#[test]
fn simple_mov_test_with_shifted_register() {
    // mov r0, r1, lsl #2
    let instr = DataProcessingInstruction::from(0xE1A00101);
    let mut emulator = ramless_emulator(Endian::Little);

    emulator.cpu[Register::R1 as _] = 16;
    assert_eq!(emulator.cpu[Register::R1 as _], 16);

    emulator.execute_data_processing_instruction(instr).unwrap();

    assert_eq!(emulator.cpu[Register::R0 as _], 16 << 2);
}
