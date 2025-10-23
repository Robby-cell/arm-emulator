use crate::{
    execution::tests::ramless_emulator, instructions::BranchInstruction,
    memory::Endian,
};

#[test]
fn test_branch_forward() {
    let instr = BranchInstruction::from(u32::from_be_bytes([
        0xEA, 0x00, 0x00, 0x06,
    ]));
    let mut emulator = ramless_emulator(Endian::Little);

    assert_eq!(emulator.cpu.pc(), 0);

    emulator.execute_branch_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.pc(), 0x20);
    assert_eq!(emulator.cpu.lr(), 0);
}

#[test]
fn test_branch_backward() {
    let instr = BranchInstruction::from(u32::from_be_bytes([
        0xEA, 0xFF, 0xFF, 0xBE,
    ]));
    let mut emulator = ramless_emulator(Endian::Little);
    emulator.cpu.set_pc(0x1000);

    assert_eq!(emulator.cpu.pc(), 0x1000);

    emulator.execute_branch_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.pc(), 0xF00);
    assert_eq!(emulator.cpu.lr(), 0);
}

#[test]
fn test_branch_with_link_forward() {
    let instr = BranchInstruction::from(u32::from_be_bytes([
        0xEB, 0x00, 0x03, 0xFE,
    ]));
    let mut emulator = ramless_emulator(Endian::Little);
    emulator.cpu.set_pc(0x8000);

    assert_eq!(emulator.cpu.pc(), 0x8000);

    emulator.execute_branch_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.pc(), 0x9000);
    assert_eq!(emulator.cpu.lr(), 0x8004);
}

#[test]
fn test_branch_with_link_backward() {
    let instr = BranchInstruction::from(u32::from_be_bytes([
        0xEB, 0xFF, 0xFF, 0xFF,
    ]));
    let mut emulator = ramless_emulator(Endian::Little);
    emulator.cpu.set_pc(0x1000);

    assert_eq!(emulator.cpu.pc(), 0x1000);

    emulator.execute_branch_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.pc(), 0x1004);
    assert_eq!(emulator.cpu.lr(), 0x1004);
}
