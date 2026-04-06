use crate::{
    instructions::{BlockDataTransferInstruction, fields::Register},
    memory::{Bus, Endian},
};

const SRAM_BEGIN: u32 = Bus::SRAM_BEGIN;

#[test]
fn test_stm_increment_after_no_writeback() {
    // STMIA R0, {R1, R2}  (Store Multiple Increment After)
    // R0 is base. Store R1 at [R0], R2 at [R0+4]. R0 is NOT updated.
    // Enc: Cond=AL(E), 100, P=0(Post/IA), U=1(Up), S=0, W=0, L=0(Store), Rn=0, Regs=00...0110
    let instr = BlockDataTransferInstruction::from(u32::from_be_bytes([
        0xE8, 0x80, 0x00, 0x06,
    ]));

    let mut emulator = super::emulator(Endian::Little, 8);

    // Setup registers
    emulator.cpu.set_register(Register::R0 as _, SRAM_BEGIN); // Base Address
    emulator.cpu.set_register(Register::R1 as _, 0xAABBCCDD);
    emulator.cpu.set_register(Register::R2 as _, 0x11223344);

    emulator
        .execute_block_data_transfer_instruction(instr)
        .unwrap();

    // Check Memory
    // R1 should be at SRAM_BEGIN
    assert_eq!(emulator.read32(SRAM_BEGIN).unwrap(), 0xAABBCCDD);
    // R2 should be at SRAM_BEGIN + 4
    assert_eq!(emulator.read32(SRAM_BEGIN + 4).unwrap(), 0x11223344);

    // Check Writeback (Should NOT have happened)
    assert_eq!(emulator.cpu.register(Register::R0 as _), SRAM_BEGIN);
}

#[test]
fn test_ldm_increment_after_with_writeback() {
    // LDMIA R0!, {R1, R2} (Load Multiple Increment After, Writeback)
    // Load [R0] into R1, [R0+4] into R2. Update R0 to R0+8.
    // Enc: Cond=AL, 100, P=0, U=1, S=0, W=1, L=1(Load), Rn=0, Regs=6
    let instr = BlockDataTransferInstruction::from(u32::from_be_bytes([
        0xE8, 0xB0, 0x00, 0x06,
    ]));

    let mut emulator = super::emulator(Endian::Little, 8);

    // Setup Memory
    emulator.write32(SRAM_BEGIN, 0xDEADBEEF).unwrap();
    emulator.write32(SRAM_BEGIN + 4, 0xCAFEBABE).unwrap();

    // Setup Base Register
    emulator.cpu.set_register(Register::R0 as _, SRAM_BEGIN);

    emulator
        .execute_block_data_transfer_instruction(instr)
        .unwrap();

    // Check Registers
    assert_eq!(emulator.cpu.register(Register::R1 as _), 0xDEADBEEF);
    assert_eq!(emulator.cpu.register(Register::R2 as _), 0xCAFEBABE);

    // Check Writeback (0x200 + 4*2 = 0x208)
    assert_eq!(emulator.cpu.register(Register::R0 as _), SRAM_BEGIN + 8);
}

#[test]
fn test_push_multiple() {
    // PUSH {R0, R1}  ->  STMDB SP!, {R0, R1}
    // Decrement Before: SP -= 8. Store Low Reg (R0) at Low Addr (SP). Store High Reg at High.
    // Enc: Cond=AL, 100, P=1(Pre), U=0(Down), S=0, W=1, L=0, Rn=SP(13), Regs=3
    let instr = BlockDataTransferInstruction::from(u32::from_be_bytes([
        0xE9, 0x2D, 0x00, 0x03,
    ]));

    let mut emulator = super::emulator(Endian::Little, 20);

    // Initialize SP to top of a memory region
    let start_sp = SRAM_BEGIN + 8;
    emulator.cpu.set_sp(start_sp);
    emulator.cpu.set_register(Register::R0 as _, 0x10);
    emulator.cpu.set_register(Register::R1 as _, 0x20);

    emulator
        .execute_block_data_transfer_instruction(instr)
        .unwrap();

    // Expect SP to decrease by 8 bytes
    assert_eq!(emulator.cpu.sp(), SRAM_BEGIN);

    // Check Memory:
    // Lowest register (R0) goes to lowest address (new SP)
    assert_eq!(emulator.read32(SRAM_BEGIN).unwrap(), 0x10);
    // Next register (R1) goes to next address
    assert_eq!(emulator.read32(SRAM_BEGIN + 4).unwrap(), 0x20);
}

#[test]
fn test_pop_multiple() {
    // POP {R0, R1} -> LDMIA SP!, {R0, R1}
    // Load R0 from SP, R1 from SP+4. SP += 8.
    // Enc: Cond=AL, 100, P=0(Post), U=1(Up), S=0, W=1, L=1, Rn=SP, Regs=3
    let instr = BlockDataTransferInstruction::from(u32::from_be_bytes([
        0xE8, 0xBD, 0x00, 0x03,
    ]));

    let mut emulator = super::emulator(Endian::Little, 8);

    // Setup Stack content
    emulator.write32(SRAM_BEGIN, 0x99).unwrap();
    emulator.write32(SRAM_BEGIN + 4, 0x88).unwrap();

    // Setup SP pointing to data
    emulator.cpu.set_sp(SRAM_BEGIN);

    emulator
        .execute_block_data_transfer_instruction(instr)
        .unwrap();

    // Verify Registers
    assert_eq!(emulator.cpu.register(Register::R0 as _), 0x99);
    assert_eq!(emulator.cpu.register(Register::R1 as _), 0x88);

    // Verify SP restored
    assert_eq!(emulator.cpu.sp(), SRAM_BEGIN + 8);
}
