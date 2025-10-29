use crate::{
    Emulator,
    instructions::fields::Register,
    memory::{Bus, Endian},
    prelude::MemoryAccessInstruction,
};

// A hypothetical test setup function
fn setup_memory_test(
    initial_regs: &[(Register, u32)],
    ram: Vec<u8>,
) -> Emulator {
    let mut emulator = Emulator {
        cpu: Default::default(),
        memory_bus: Bus::with_ram(ram),
        endian: Endian::Little,
    };
    for &(reg, val) in initial_regs {
        emulator.cpu.set_register(reg as _, val);
    }
    emulator
}

#[test]
fn test_ldr_pre_indexed_immediate_offset_no_writeback() {
    // LDR R0, [R1, #4]
    // This has U=1 (add) and W=0 (no writeback).
    // The correct big-endian byte representation is [0xE5, 0x91, 0x00, 0x04].
    let instr = MemoryAccessInstruction::from(u32::from_be_bytes([
        0xE5, 0x91, 0x00, 0x04,
    ]));

    // Setup: R1 = 0x40. Memory address to read from is 0x44.
    // We will place 0xDEADBEEF at address 0x44.
    let mut ram = vec![0; 0x50];
    let value_to_load = 0xDEADBEEF_u32;
    // Place the value in memory in little-endian format to match the bus.
    ram[0x44..0x48].copy_from_slice(&value_to_load.to_le_bytes());

    let mut emulator = setup_memory_test(&[(Register::R1, 0x40)], ram);
    // Make sure we are actually using little endian.
    emulator.set_endian(Endian::Little);

    // Make sure that we actually have 0xDEADBEEF at this address.
    assert_eq!(emulator.read32(0x44).unwrap(), 0xDEADBEEF);

    // Set R0 to a sentinel value to ensure it gets overwritten.
    emulator.cpu.set_register(Register::R0 as _, 0xFFFFFFFF);

    assert_eq!(instr.rn(), Register::R1);
    assert_eq!(instr.rd(), Register::R0);
    emulator.execute_memory_access_instruction(instr).unwrap();

    assert_eq!(emulator.cpu[Register::R0 as _], value_to_load);
    assert_eq!(
        emulator.cpu.register(Register::R1 as _),
        0x40,
        "Base register R1 should not be changed"
    );
}

#[test]
fn test_ldr_post_indexed_negative_immediate_offset() {
    // LDR R0, [R1], #-8
    // Loads from address R1. Then, updates R1 to R1-8.
    let instr = MemoryAccessInstruction::from(u32::from_be_bytes([
        0xE4, 0x11, 0x00, 0x08,
    ]));

    // Setup: R1 = 0x20. Load will happen at 0x20. R1 will be updated to 0x18.
    // We will place 0xFEEDFACE at address 0x20.
    let mut ram = vec![0; 0x30];
    let value_to_load = 0xFEEDFACE_u32;
    ram[0x20..0x24].copy_from_slice(&value_to_load.to_le_bytes());

    let mut emulator = setup_memory_test(&[(Register::R1, 0x20)], ram);

    emulator.execute_memory_access_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), value_to_load);
    assert_eq!(
        emulator.cpu.register(Register::R1 as _),
        0x18,
        "Base register R1 should be updated by post-indexing"
    );
}

#[test]
fn test_ldr_shifted_register_offset() {
    // LDR R0, [R1, R2, LSL #2]
    // Loads from address R1 + (R2 << 2).
    let instr = MemoryAccessInstruction::from(u32::from_be_bytes([
        0xE7, 0x91, 0x01, 0x02,
    ]));

    // Setup: R1 = 0x80, R2 = 0x08.
    // Offset = 8 << 2 = 32 (0x20).
    // Memory address = 0x80 + 0x20 = 0xA0.
    // We will place 0x12345678 at address 0xA0.
    let mut ram = vec![0; 0xB0];
    let value_to_load = 0x12345678_u32;
    ram[0xA0..0xA4].copy_from_slice(&value_to_load.to_le_bytes());

    let mut emulator = setup_memory_test(
        &[(Register::R1, 0x80), (Register::R2, 0x08)],
        ram,
    );

    emulator.execute_memory_access_instruction(instr).unwrap();

    assert_eq!(emulator.cpu.register(Register::R0 as _), value_to_load);
    assert_eq!(
        emulator.cpu.register(Register::R1 as _),
        0x80,
        "Base register R1 should not be changed"
    );
    assert_eq!(
        emulator.cpu.register(Register::R2 as _),
        0x08,
        "Offset register R2 should not be changed"
    );
}

#[test]
fn test_str_pre_indexed_immediate_with_writeback() {
    // STR R0, [R1, #12]!
    // Stores R0 at address R1+12. Then, updates R1 to R1+12.
    let instr = MemoryAccessInstruction::from(u32::from_be_bytes([
        0xE5, 0xA1, 0x00, 0x0C,
    ]));

    // Setup: R1 = 0x50. R0 = 0xCAFEBABE.
    // The store address will be 0x50 + 12 = 0x5C.
    let ram = vec![0; 0x70];
    let value_to_store = 0xCAFEBABE_u32;

    let mut emulator = setup_memory_test(
        &[(Register::R0, value_to_store), (Register::R1, 0x50)],
        ram,
    );

    emulator.execute_memory_access_instruction(instr).unwrap();

    // Verify memory was written correctly.
    // The bus reads the bytes and converts them to a native u32.
    let stored_value = emulator.read32(0x5C).unwrap();
    assert_eq!(stored_value, value_to_store);

    // Verify R1 was updated due to writeback.
    assert_eq!(
        emulator.cpu.register(Register::R1 as _),
        0x5C,
        "Base register R1 should be updated"
    );
}

#[test]
fn test_str_subtracted_shifted_register_offset() {
    // STR R0, [R1, -R2, ROR #4]
    // Stores R0 at address R1 - (R2 ROR 4). R1 is not updated.
    // Encoded with U=0 for subtraction.
    let instr = MemoryAccessInstruction::from(u32::from_be_bytes([
        0xE7, 0x01, 0x02, 0x62,
    ]));

    // Setup: R1 = 0x103. R2 = 0xF0. R0 = 0xABCD.
    // Offset = 0xF0 ROR 4 = 0x0F.
    // Memory address = 0x103 - 0x0F = 0xF4.
    let ram = vec![0; 0x110];
    let value_to_store = 0xABCD_u32;

    let mut emulator = setup_memory_test(
        &[
            (Register::R0, value_to_store),
            (Register::R1, 0x103),
            (Register::R2, 0xF0),
        ],
        ram,
    );

    emulator.execute_memory_access_instruction(instr).unwrap();

    let stored_value = emulator.read32(0xF4).unwrap();
    assert_eq!(stored_value, value_to_store);
    assert_eq!(
        emulator.cpu.register(Register::R1 as _),
        0x103,
        "Base register R1 should not be updated"
    );
}
