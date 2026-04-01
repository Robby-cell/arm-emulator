use crate::{
    Emulator,
    cpu::Cpu,
    memory::{Bus, Endian, MemoryAccessError},
};

fn ramless_emulator(endian: Endian) -> Emulator {
    Emulator::new(Cpu::new(), Bus::new(0, 0, 0), endian)
}

#[test]
fn test_endianness_read_write() {
    let mut bus = Bus::new(1024, 1024, 0);

    // Write 0xAABBCCDD in Little Endian to SRAM (0x20000000)
    bus.write32_le(0x20000000, 0xAABBCCDD).unwrap();

    // Read back in Little Endian should be exactly the same
    assert_eq!(bus.read32_le(0x20000000).unwrap(), 0xAABBCCDD);

    // Read back in Big Endian should be reversed: 0xDDCCBBAA
    assert_eq!(bus.read32_be(0x20000000).unwrap(), 0xDDCCBBAA);
}

#[test]
fn test_memory_out_of_bounds_protection() {
    let bus = Bus::new(1024, 1024, 0); // SRAM ends at 0x200003FF

    // Attempt to read just past the allocated SRAM
    let result = bus.read32_le(0x20000400);

    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(MemoryAccessError::InvalidReadPermission { .. })
    ));
}

#[test]
fn test_unaligned_memory_write_throws_error() {
    let mut emulator = ramless_emulator(Endian::Little);

    // Attempt to write a 32-bit word (4 bytes) to an unaligned address (0x03)
    let unaligned_address = 0x03;
    let result = emulator.write32(unaligned_address, 0xDEADBEEF);

    // Assert that the emulator DOES NOT panic, but returns the correct Error type
    assert!(result.is_err());
    if let Err(MemoryAccessError::UnalignedAccess) = result {
        // Test passes
    } else {
        panic!("Expected UnalignedAccess error, got {:?}", result);
    }
}

#[test]
fn test_read_from_unmapped_memory_throws_error() {
    // Setup emulator with ONLY 1024 bytes of SRAM
    let emulator =
        Emulator::new(Cpu::new(), Bus::new(0, 1024, 0), Endian::Little);

    // Attempt to read from an address way outside the mapped SRAM (e.g., 0x2000_1000)
    let out_of_bounds_address = Bus::SRAM_BEGIN + 4096;
    let result = emulator.read32(out_of_bounds_address);

    assert!(result.is_err());
    if let Err(MemoryAccessError::InvalidReadPermission { addr }) = result
    {
        assert_eq!(addr, out_of_bounds_address);
    } else {
        panic!("Expected InvalidReadPermission, got {:?}", result);
    }
}
