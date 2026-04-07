//! ARM instruction set decoding and representation.
//!
//! This module provides the core instruction types for the ARM emulator:
//! - [`DataProcessingInstruction`]: ALU operations (ADD, SUB, MOV, etc.)
//! - [`MemoryAccessInstruction`]: Load/Store operations (LDR, STR)
//! - [`BranchInstruction`]: Branch and Branch with Link (B, BL)
//! - [`BlockDataTransferInstruction`]: Load/Store Multiple (LDM, STM)
//! - [`BranchExchangeInstruction`]: Mode switching (BX, BLX)
//! - [`SupervisorCallInstruction`]: System calls (SVC/SWI)
//! - [`MultiplyInstruction`]: 32-bit multiplication (MUL, MLA)
//! - [`MultiplyLongInstruction`]: 64-bit multiplication (UMULL, SMULL, etc.)
//! - [`BreakpointInstruction`]: Debug breakpoint (BKPT)
//! - [`Instruction`]: Enum representing any ARM instruction
//!
//! Instructions are decoded from raw 32-bit values using the [`TryFrom<u32>`]
//! implementation on [`Instruction`]. The [`ArmDecoder`] and [`InstructionEnum`]
//! macros from [`emulator_macros`] provide the decoding logic.
//!
//! The [`fields`] submodule contains common instruction field types like
//! [`Condition`], [`Opcode`], [`Register`], and shift types.

#![allow(unused_parens, dead_code)]

mod display;
pub mod fields;
#[cfg(test)]
mod tests;

use emulator_macros::{ArmDecoder, InstructionEnum};
use fields::*;

use modular_bitfield::prelude::*;
use thiserror::Error;

macro_rules! assert_sized {
    ($t1:ty = $t2:ty) => {
        const _: () = assert!(size_of::<$t1>() == size_of::<$t2>());
    };
}
macro_rules! assert_u32_sized {
    ($t:ty) => {
        assert_sized!($t = u32);
    };
}

/// Represents a Data Processing instruction in ARM architecture.
/// Fields are defined according to the ARM instruction set encoding.
///
/// For more details, refer to
/// [Official documentation](https://developer.arm.com/documentation/ddi0406/b/Application-Level-Architecture/The-Instruction-Sets/Data-processing-instructions)
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
#[must_use]
pub struct DataProcessingInstruction {
    // Fields are defined from LSB (bit 0) to MSB (bit 31)
    pub operand2: B12,
    pub rd: Register,
    pub rn: Register,
    pub s: SetFlags,
    pub opcode: Opcode,
    pub immediate: ImmediateFlag,
    #[skip]
    _b2: B2,
    pub cond: Condition,
}

assert_u32_sized!(DataProcessingInstruction);

#[derive(Debug, Copy, Clone)]
#[must_use]
pub enum Operand2 {
    ShiftedRegisterOffset(ShiftedRegisterOffset),
    Immediate(u16),
}

impl DataProcessingInstruction {
    pub fn op2(&self) -> Operand2 {
        match self.immediate() {
            ImmediateFlag::Imm => Operand2::Immediate(self.operand2()),
            ImmediateFlag::Register => {
                Operand2::ShiftedRegisterOffset(self.operand2().into())
            }
        }
    }
}

/// Represents a Memory Access instruction in the ARM architecture.
/// Fields are defined according to the ARM instruction set encoding.
/// This covers Load and Store instructions (LDR, STR, etc.).
///
/// For more details, refer to
/// [Official documentation](https://developer.arm.com/documentation/dui0231/b/arm-instruction-reference/arm-memory-access-instructions)
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
#[must_use]
pub struct MemoryAccessInstruction {
    pub offset: B12,
    pub rd: Register,
    pub rn: Register,
    pub l: LoadStoreFlag,
    pub w: WriteBackFlag,
    pub b: ByteWordFlag,
    pub u: UpDownFlag,
    pub p: IndexFlag,
    pub i: OffsetType,
    #[skip]
    _b2: B2,
    pub cond: Condition,
}

assert_u32_sized!(MemoryAccessInstruction);

#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u16)]
#[must_use]
pub struct ShiftedRegisterOffset {
    pub rm: Register,
    #[skip]
    __: B1,
    pub shift_type: ShiftType,
    pub shift_amount: B5,
    #[skip]
    __: B4,
}

#[bitfield]
#[derive(Debug, Copy, Clone)]
#[must_use]
pub struct RotatedImmediate {
    pub immediate: B8,
    pub rotate: B4,
    #[skip]
    __: B4,
}

#[derive(Debug, Copy, Clone)]
#[must_use]
pub enum MemoryOffset {
    Immediate(u16),
    ShiftedRegister(ShiftedRegisterOffset),
}

impl MemoryAccessInstruction {
    pub fn memory_offset(&self) -> MemoryOffset {
        match self.i() {
            OffsetType::Immediate => {
                MemoryOffset::Immediate(self.offset())
            }
            OffsetType::Register => MemoryOffset::ShiftedRegister(
                ShiftedRegisterOffset::from(self.offset() as u16),
            ),
        }
    }
}

/// Represents a Branch instruction in the ARM architecture.
/// Fields are defined according to the ARM instruction set encoding.
/// This covers both Branch (B) and Branch with Link (BL) instructions.
///
/// For more details, refer to
/// [Official documentation](https://developer.arm.com/documentation/ddi0403/d/Application-Level-Architecture/The-ARMv7-M-Instruction-Set/Branch-instructions?lang=en)
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
#[must_use]
pub struct BranchInstruction {
    pub offset: B24,
    pub l: LinkFlag,
    #[skip]
    _b3: B3,
    pub cond: Condition,
}

assert_u32_sized!(BranchInstruction);

/// Represents a Block Data Transfer instruction in the ARM architecture.
/// Fields are defined according to the ARM instruction set encoding.
/// This covers instructions like LDM and STM. (Load/Store Multiple)
///
/// For more details, refer to
/// [Official documentation](https://developer.arm.com/documentation/ddi0597/2022-03/A32-Instructions-by-Encoding/Branch--branch-with-link--and-block-data-transfer)
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
#[must_use]
pub struct BlockDataTransferInstruction {
    pub register_list: B16,
    pub rn: Register,
    pub l: LoadStoreFlag,
    pub w: WriteBackFlag,
    pub s: PrivilegeActionFlag,
    pub u: UpDownFlag,
    pub p: IndexFlag,
    #[skip]
    _b3: B3,
    pub cond: Condition,
}

assert_u32_sized!(BlockDataTransferInstruction);

/// Represents a Supervisor Call (SVC) instruction in the ARM architecture.
/// Fields are defined according to the ARM instruction set encoding.
/// This instruction is used to invoke system calls/supervisor-level functions.
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
#[must_use]
pub struct SupervisorCallInstruction {
    /// A 24-bit immediate value passed to the supervisor.
    /// Modern OSes like Linux typically ignore this in favor of a register-based ABI.
    /// In our implementation, we will just use this.
    pub immediate: B24,
    /// The fixed bit pattern `1111` that identifies this as an SVC instruction.
    #[skip]
    _b4: B4,
    /// The condition under which this instruction will execute.
    pub cond: Condition,
}

assert_u32_sized!(SupervisorCallInstruction);

#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
pub struct BranchExchangeInstruction {
    pub rm: Register, // Bits 0-3: The register containing the target address
    #[skip]
    _should_be_1: B4, // Bits 4-7: Always 0001
    #[skip]
    _should_be_fff: B12, // Bits 8-19: Always 1111 1111 1111
    #[skip]
    _should_be_12: B8, // Bits 20-27: Always 0001 0010
    pub cond: Condition, // Bits 28-31
}

assert_u32_sized!(BranchExchangeInstruction);

/// Represents 32-bit Multiply instructions (MUL, MLA).
/// Format: cond 0000 00AS Rd Rn Rs 1001 Rm
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
pub struct MultiplyInstruction {
    pub rm: Register,
    #[skip]
    _signature: B4, // Must be 1001 (9)
    pub rs: Register,
    pub rn: Register,
    pub rd: Register,
    pub s: SetFlags,
    pub a: AccumulateFlag,
    #[skip]
    _zeros: B6, // Must be 000000
    pub cond: Condition,
}

assert_u32_sized!(MultiplyInstruction);

/// Represents 64-bit Multiply Long instructions (UMULL, UMLAL, SMULL, SMLAL).
/// Format: cond 0000 1UAS RdHi RdLo Rs 1001 Rm
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
pub struct MultiplyLongInstruction {
    pub rm: Register,
    #[skip]
    _signature: B4, // Must be 1001 (9)
    pub rs: Register,
    pub rd_lo: Register,
    pub rd_hi: Register,
    pub s: SetFlags,
    pub a: AccumulateFlag,
    pub u: SignedFlag,
    #[skip]
    _zeros: B5, // Must be 00001
    pub cond: Condition,
}

assert_u32_sized!(MultiplyLongInstruction);

/// Represents a Breakpoint instruction.
#[bitfield]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[repr(C)]
#[must_use]
pub struct BreakpointInstruction {
    pub imm4: B4,
    #[skip]
    _b8: B8,
    pub imm12: B12,
    #[skip]
    _b8: B8,
}

assert_u32_sized!(BreakpointInstruction);

/// Represents an ARM instruction, which can be one of several types:
/// [Data Processing](DataProcessingInstruction), [Memory Access](MemoryAccessInstruction),
/// [Branch](BranchInstruction), or [Block Data Transfer](BlockDataTransferInstruction).
///
/// This is a rust enum that allows for easy discrimination between the types.
/// Construction is done via [TryFrom<u32>], which analyzes the raw instruction bits
/// to determine the correct variant.
/// The size of this enum will be larget than 4 bytes, as it needs to store
/// the largest variant (they are all 4 bytes) plus a discriminant.
///
/// # Examples
///
/// ```rust
/// use emulator::instructions::*;
/// use emulator::instructions::fields::*;
///
/// // Assembly: MOV r1, #123   (Always condition)
/// let raw_inst = 0xE3A0107B;
/// let decoded: Instruction = raw_inst.try_into().unwrap();
///
/// if let Instruction::DataProcessing(inst) = decoded {
///     assert_eq!(inst.cond(), Condition::AL);
///     assert_eq!(inst.immediate(), ImmediateFlag::Imm);
///     assert_eq!(inst.opcode(), Opcode::MOV);
///     assert_eq!(inst.s(), SetFlags::No);
///     assert_eq!(inst.rn(), Register::R0); // Rn is not used in this MOV variant
///     assert_eq!(inst.rd(), Register::R1);
///     assert_eq!(inst.operand2(), 123);
/// } else {
///     panic!("Incorrect instruction type decoded: {:?}", decoded);
/// }
/// ```
#[derive(
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    derive_more::From,
    derive_more::Display,
    InstructionEnum,
    ArmDecoder,
)]
#[must_use]
pub enum Instruction {
    /// xxxx 0001 0010 xxxx xxxx xxxx 0111 xxxx
    #[decode("cond 0001 0010 xxxx xxxx xxxx 0111 xxxx")]
    Breakpoint(BreakpointInstruction),

    // NOTE: BranchExchange must be before DataProcessing, otherwise it will be decoded
    // as DataProcessing, because the macro will check in order.
    /// xxxx 0001 0010 1111 1111 1111 0001 xxxx
    #[decode("cond 0001 0010 1111 1111 1111 0001 mmmm")]
    BranchExchange(BranchExchangeInstruction),

    /// xxxx 1111 xxxx xxxx xxxx xxxx xxxx xxxx
    #[decode("cond 1111 xxxx xxxx xxxx xxxx xxxx xxxx")]
    SupervisorCall(SupervisorCallInstruction),

    /// xxxx 0000 00AS dddd nnnn ssss 1001 mmmm
    #[decode("cond 0000 00AS dddd nnnn ssss 1001 mmmm")]
    Multiply(MultiplyInstruction),

    /// xxxx 0000 1UAS hhhh llll ssss 1001 mmmm
    #[decode("cond 0000 1UAS hhhh llll ssss 1001 mmmm")]
    MultiplyLong(MultiplyLongInstruction),

    /// xxxx 000x xxxx xxxx xxxx xxxx xxxx xxxx (Data Processing - Reg/Imm Shift)
    /// xxxx 001x xxxx xxxx xxxx xxxx xxxx xxxx (Data Processing - Immediate)
    #[decode("cond 000x xxxx xxxx xxxx xxxx xxxx xxxx")]
    #[decode("cond 001x xxxx xxxx xxxx xxxx xxxx xxxx")]
    DataProcessing(DataProcessingInstruction),

    /// xxxx 010x xxxx xxxx xxxx xxxx xxxx xxxx
    /// xxxx 011x xxxx xxxx xxxx xxxx xxxx xxxx
    #[decode("cond 010x xxxx xxxx xxxx xxxx xxxx xxxx")]
    #[decode("cond 011x xxxx xxxx xxxx xxxx xxxx xxxx")]
    MemoryAccess(MemoryAccessInstruction),

    /// xxxx 100x xxxx xxxx xxxx xxxx xxxx xxxx
    #[decode("cond 100x xxxx xxxx xxxx xxxx xxxx xxxx")]
    BlockDataTransfer(BlockDataTransferInstruction),

    /// xxxx 101x xxxx xxxx xxxx xxxx xxxx xxxx
    #[decode("cond 101x xxxx xxxx xxxx xxxx xxxx xxxx")]
    Branch(BranchInstruction),
}

impl Instruction {
    pub fn cond(&self) -> Condition {
        match self {
            Instruction::DataProcessing(inst) => inst.cond(),
            Instruction::MemoryAccess(inst) => inst.cond(),
            Instruction::BlockDataTransfer(inst) => inst.cond(),
            Instruction::Branch(inst) => inst.cond(),
            Instruction::BranchExchange(inst) => inst.cond(),
            Instruction::SupervisorCall(inst) => inst.cond(),
            Instruction::Multiply(inst) => inst.cond(),
            Instruction::MultiplyLong(inst) => inst.cond(),
            Instruction::Breakpoint(_inst) => Condition::AL,
        }
    }
}

/// Errors that can occur when converting a raw [u32] into an [Instruction].
#[derive(Error, Debug, Clone)]
pub enum InstructionConversionError {
    #[error("Invalid instruction class")]
    InvalidInstructionClass,
}
