use super::*;

use crate::memory::little_endian_to_native;

#[test]
fn test_decode_data_processing_immediate() {
    // Assembly: MOV r1, #123   (Always condition)
    let raw_inst = little_endian_to_native(0xE3A0107B);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::DataProcessing(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.immediate(), ImmediateFlag::Imm);
        assert_eq!(inst.opcode(), Opcode::MOV);
        assert_eq!(inst.s(), SetFlags::No);
        assert_eq!(inst.rn(), Register::R0); // Rn is not used in this MOV variant
        assert_eq!(inst.rd(), Register::R1);
        assert_eq!(inst.operand2(), 123);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_data_processing_register() {
    // Assembly: ADD r3, r2, r1   (Always condition)
    let raw_inst = little_endian_to_native(0xE0823001);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::DataProcessing(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.immediate(), ImmediateFlag::Register);
        assert_eq!(inst.opcode(), Opcode::ADD);
        assert_eq!(inst.s(), SetFlags::No);
        assert_eq!(inst.rn(), Register::R2);
        assert_eq!(inst.rd(), Register::R3);

        // Operand2 for this format is just the register `rm`.
        let rm = inst.operand2() & 0b1111;
        assert_eq!(rm, 1); // r1
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_memory_access_immediate() {
    // Assembly: LDR r5, [r6, #-4]!   (pre-indexed, write-back)
    let raw_inst = little_endian_to_native(0xE5365004);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::MemoryAccess(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.i(), OffsetType::Immediate);
        assert_eq!(inst.p(), IndexFlag::Pre);
        assert_eq!(inst.u(), UpDownFlag::Sub);
        assert_eq!(inst.b(), ByteWordFlag::Word);
        assert_eq!(inst.w(), WriteBackFlag::Write);
        assert_eq!(inst.l(), LoadStoreFlag::Load);
        assert_eq!(inst.rn(), Register::R6);
        assert_eq!(inst.rd(), Register::R5);
        assert_eq!(inst.offset(), 4);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_branch() {
    // Assembly: BL 0x24   (Branch and Link 8 words/32 bytes forward from PC+8)
    let raw_inst = little_endian_to_native(0xEB000008);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::Branch(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.l(), LinkFlag::Yes);
        assert_eq!(inst.offset() as u32, 8);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_block_data_transfer() {
    // Assembly: STMDB sp!, {r4, r5, lr}  (Canonical PUSH)
    let raw_inst = little_endian_to_native(0xE92D4030);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::BlockDataTransfer(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.p(), IndexFlag::Pre); // D = Pre
        assert_eq!(inst.u(), UpDownFlag::Sub); // D = Decrement/Subtract
        assert_eq!(inst.s(), PrivilegeActionFlag::Normal);
        assert_eq!(inst.w(), WriteBackFlag::Write);
        assert_eq!(inst.l(), LoadStoreFlag::Store);
        assert_eq!(inst.rn(), Register::R13); // sp

        // Register list for {r4, r5, r14}
        use register_mask::*;
        let expected_list = R4 | R5 | R14;

        assert_eq!(inst.register_list() as u16, expected_list.into());
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_supervisor_call_conditional() {
    // Assembly: SVCEQ #0x1A   (Supervisor call, if Z flag is set)
    let raw_inst = little_endian_to_native(0x0F00001A);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::SupervisorCall(inst) = decoded {
        assert_eq!(inst.cond(), Condition::EQ);
        assert_eq!(inst.immediate(), 0x1A);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_data_processing_s_flag() {
    // Assembly: CMP r2, #100   (Always condition)
    // CMP is effectively SUBS with the result discarded. It always sets flags.
    let raw_inst = little_endian_to_native(0xE3520064);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::DataProcessing(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.immediate(), ImmediateFlag::Imm);
        assert_eq!(inst.opcode(), Opcode::CMP);
        assert_eq!(inst.s(), SetFlags::Yes);
        assert_eq!(inst.rn(), Register::R2);
        // rd is not relevant for CMP but is encoded as r0
        assert_eq!(inst.rd(), Register::R0);
        assert_eq!(inst.operand2(), 100);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_data_processing_register_shifted() {
    // Assembly: ADD r5, r4, r3, LSL #2
    let raw_inst = little_endian_to_native(0xE0845103);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::DataProcessing(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.immediate(), ImmediateFlag::Register);
        assert_eq!(inst.opcode(), Opcode::ADD);
        assert_eq!(inst.s(), SetFlags::No);
        assert_eq!(inst.rn(), Register::R4);
        assert_eq!(inst.rd(), Register::R5);

        // For `r3, LSL #2`, operand2 is encoded as:
        // shift_amount = 2 (in bits 11-7)
        // shift_type   = 0 (LSL, in bits 6-5)
        // rm           = 3 (in bits 3-0)
        // This results in the value 0b00010_00_0_0011 = 0x103
        assert_eq!(inst.operand2(), 0x103);

        // We can also decode it using the provided helper struct
        let shifted = ShiftedRegisterOffset::from(inst.operand2() as u16);
        assert_eq!(shifted.rm(), Register::R3);
        assert_eq!(shifted.shift_type(), ShiftType::LSL);
        assert_eq!(shifted.shift_amount(), 2);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_memory_access_post_indexed() {
    // Assembly: STR r8, [r1], #-16   (post-indexed, write-back)
    let raw_inst = little_endian_to_native(0xE4018010);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::MemoryAccess(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.i(), OffsetType::Immediate);
        assert_eq!(inst.p(), IndexFlag::Post);
        assert_eq!(inst.u(), UpDownFlag::Sub);
        assert_eq!(inst.w(), WriteBackFlag::NoModify); // W bit is used for other purposes in post-indexed mode
        assert_eq!(inst.l(), LoadStoreFlag::Store);
        assert_eq!(inst.rn(), Register::R1);
        assert_eq!(inst.rd(), Register::R8);
        assert_eq!(inst.offset(), 16);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_block_data_transfer_load() {
    // Assembly: LDM r8, {r0, r1, r2, r3}
    let raw_inst = little_endian_to_native(0xE898000F);
    let decoded: Instruction = raw_inst.try_into().unwrap();

    if let Instruction::BlockDataTransfer(inst) = decoded {
        assert_eq!(inst.cond(), Condition::AL);
        assert_eq!(inst.p(), IndexFlag::Post); // Corresponds to 'Increment After'
        assert_eq!(inst.u(), UpDownFlag::Add);
        assert_eq!(inst.w(), WriteBackFlag::NoModify); // NoModify '!' so no write-back
        assert_eq!(inst.l(), LoadStoreFlag::Load);
        assert_eq!(inst.rn(), Register::R8);

        // Register list for {r0, r1, r2, r3} is a bitmask of the first 4 bits
        let expected_list = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3);
        assert_eq!(inst.register_list(), expected_list);
    } else {
        panic!("Incorrect instruction type decoded: {:?}", decoded);
    }
}

#[test]
fn test_decode_invalid_instruction_class() {
    // An instruction with bits [27:25] as `110` corresponds to coprocessor
    // instructions, which are not handled by our decoder.
    let raw_inst = little_endian_to_native(0xEC000000);
    let result = Instruction::try_from(raw_inst);
    assert!(matches!(
        result,
        Err(InstructionConversionError::InvalidInstructionClass)
    ));
}
