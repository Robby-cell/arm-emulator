use crate::{
    Emulator,
    cpu::Cpu,
    memory::{
        Bus, Endian, MemoryAccessResult, MemoryMappedPeripheral,
        Peripheral,
    },
};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

// Define a Mock Peripheral for testing
#[derive(Default)]
struct MockPeripheral {
    data: AtomicU32,
}

impl Peripheral for MockPeripheral {
    fn read32(&self, _offset: u32) -> MemoryAccessResult<u32> {
        Ok(self.data.load(Ordering::Relaxed))
    }

    fn write32(&self, _offset: u32, value: u32) -> MemoryAccessResult<()> {
        self.data.store(value, Ordering::Relaxed);
        Ok(())
    }

    fn read_byte(&self, _offset: u32) -> MemoryAccessResult<u8> {
        Ok(0)
    }

    fn write_byte(
        &self,
        _offset: u32,
        _value: u8,
    ) -> MemoryAccessResult<()> {
        Ok(())
    }

    fn reset(&self) {
        self.data.store(0, Ordering::Release);
    }
}

#[test]
fn test_peripheral_integration_write() {
    // Setup the mock peripheral and map it to 0x40000000
    let mock_periph = Arc::new(MockPeripheral::default());
    let mapped_periph = MemoryMappedPeripheral::new(
        0x40000000..=0x40000FFF,
        mock_periph.clone(),
    );

    let mut bus = Bus::new(1024, 1024, 0);
    bus.add_peripheral(mapped_periph);

    // Load Code:
    // MOV R1, #0x42
    // STR R1, [R0]
    let code: [u8; 8] = [
        0x42, 0x10, 0xA0, 0xE3, // MOV R1, #0x42
        0x00, 0x10, 0x80, 0xE5, // STR R1, [R0]
    ];
    bus.load_code(&code);

    let mut cpu = Cpu::new();
    cpu.set_register(0, 0x40000000); // Pre-load R0 with peripheral address

    let mut emulator = Emulator::new(cpu, bus, Endian::Little);

    // Execute the two instructions
    emulator.step().unwrap(); // Executes MOV
    emulator.step().unwrap(); // Executes STR

    // Assert that the Mock Peripheral caught the hardware write
    assert_eq!(mock_periph.data.load(Ordering::Relaxed), 0x42);
}
