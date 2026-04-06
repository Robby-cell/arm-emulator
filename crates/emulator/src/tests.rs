use crate::{
    Emulator,
    cpu::Cpu,
    instructions::fields::Register,
    memory::{Bus, Endian, Ram},
};

#[test]
fn test_basic_execution() {
    let mut ram = Ram::default();

    // MOV R0, #32
    ram.extend_from_slice(&[0xE3, 0xA0, 0x00, 0x20]);
    // MOV R1, #10
    ram.extend_from_slice(&[0xE3, 0xA0, 0x10, 0x0A]);
    // ADDS R3, R0, R1
    ram.extend_from_slice(&[0xE0, 0x90, 0x30, 0x01]);

    let memory_bus = Bus::with_ram(ram, vec![], vec![]);
    let mut emulator = Emulator::new(Cpu::new(), memory_bus, Endian::Big);

    for register in 0..16 {
        assert_eq!(emulator.cpu[register as _], 0);
    }

    // Execute all 3 instructions
    for _ in 0..3 {
        emulator.step().unwrap();
    }

    assert!(!emulator.cpu.n());
    assert!(!emulator.cpu.z());
    assert!(!emulator.cpu.c());
    assert!(!emulator.cpu.v());

    assert_eq!(emulator.cpu[Register::R0 as _], 32);
    assert_eq!(emulator.cpu[Register::R1 as _], 10);
    assert_eq!(emulator.cpu[Register::R3 as _], 42);
}

/// Helper to create an emulator with executable RAM
/// Must provide valid code slice as little-endian
fn setup_integration_test(code: &[u8]) -> Emulator {
    // 1KiB SRAM
    let mut emulator =
        Emulator::new(Cpu::default(), Bus::default(), Endian::Little);
    emulator.load_program_with_sram_size(code, None, None, 1024);
    emulator
}

#[test]
fn test_arithmetic_sequence() {
    // 1. MOV R0, #2
    // 2. MOV R1, #4
    // 3. ADD R0, R0, R1
    let code: [u8; 12] = [
        0x02, 0x00, 0xA0, 0xE3, // MOV R0, #2
        0x04, 0x10, 0xA0, 0xE3, // MOV R1, #4
        0x01, 0x00, 0x80, 0xE0, // ADD R0, R0, R1
    ];

    let mut emulator = setup_integration_test(&code);

    // Execute 3 steps
    emulator.step().expect("Step 1 failed");
    emulator.step().expect("Step 2 failed");
    emulator.step().expect("Step 3 failed");

    // Expect R0 = 2 + 4 = 6
    assert_eq!(emulator.cpu.register(Register::R0 as _), 6);
    // Expect R1 = 4
    assert_eq!(emulator.cpu.register(Register::R1 as _), 4);
    // Expect PC = 12
    assert_eq!(emulator.cpu.pc(), 12);
}

#[test]
fn test_branching_loop() {
    // Simple loop:
    // 0x00: MOV R0, #0
    // 0x04: ADD R0, R0, #1
    // 0x08: CMP R0, #3
    // 0x0C: BNE 0x04 (Jump back to ADD if not equal to 3)
    // 0x10: MOV R1, #0xFF (marker)
    let code: [u8; 20] = [
        0x00, 0x00, 0xA0, 0xE3, 0x01, 0x00, 0x80, 0xE2, 0x03, 0x00, 0x50,
        0xE3, 0xFC, 0xFF, 0xFF, 0x1A, // BNE -4
        0xFF, 0x10, 0xA0, 0xE3,
    ];

    let mut emulator = setup_integration_test(&code);

    // Max steps to prevent infinite loop if broken
    let mut steps = 0;
    while emulator.cpu.pc() < 0x10 && steps < 20 {
        emulator.step().unwrap();
        steps += 1;
    }

    // Should finish loop when R0 == 3
    assert_eq!(emulator.cpu.register(Register::R0 as _), 3);

    // execute final MOV
    emulator.step().unwrap();
    assert_eq!(emulator.cpu.register(Register::R1 as _), 0xFF);
}

#[test]
fn test_stack_operations() {
    // 1. MOV R0, #0xAA
    // 2. PUSH {R0}
    // 3. MOV R0, #0
    // 4. POP {R0}
    let code: [u8; 16] = [
        0xAA, 0x00, 0xA0, 0xE3, // MOV R0, #0xAA
        0x01, 0x00, 0x2D, 0xE9, // PUSH {R0}
        0x00, 0x00, 0xA0, 0xE3, // MOV R0, #0
        0x01, 0x00, 0xBD, 0xE8, // POP {R0}
    ];

    let mut emulator = setup_integration_test(&code);
    let initial_sp = emulator.cpu.sp();

    // Run MOV + PUSH
    emulator.step().unwrap();
    emulator.step().unwrap();

    assert_eq!(emulator.cpu.sp(), initial_sp.overflowing_sub(4).0);
    // Verify memory write manually
    assert_eq!(
        emulator.read32(initial_sp.overflowing_sub(4).0).unwrap(),
        0xAA
    );

    // Run MOV (clear R0)
    emulator.step().unwrap();
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0);

    // Run POP
    emulator.step().unwrap();

    // R0 should be restored
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0xAA);
    // SP should be restored
    assert_eq!(emulator.cpu.sp(), initial_sp);
}
