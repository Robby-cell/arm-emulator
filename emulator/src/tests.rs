use crate::{
    Emulator,
    cpu::Cpu,
    instructions::fields::Register,
    memory::{Bus, Endian, Ram, little_endian_to_native},
};

#[test]
fn test_basic_execution() {
    let mut ram = Ram::default();

    // MOV R0, #32
    ram.extend_from_slice(
        &little_endian_to_native(0xE3A00020).to_le_bytes(),
    );
    // MOV R1, #10
    ram.extend_from_slice(
        &little_endian_to_native(0xE3A0100A).to_le_bytes(),
    );
    // ADDS R3, R0, R1
    ram.extend_from_slice(
        &little_endian_to_native(0xE0903001).to_le_bytes(),
    );

    let memory_bus = Bus::with_ram(ram);
    let mut emulator =
        Emulator::new(Cpu::new(), memory_bus, Endian::Little);

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
