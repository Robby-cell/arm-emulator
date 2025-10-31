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
