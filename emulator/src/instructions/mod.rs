#![allow(unused_parens, dead_code)]

mod display;
pub mod fields;
#[cfg(test)]
mod tests;

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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[must_use]
pub enum Instruction {
    DataProcessing(DataProcessingInstruction),
    MemoryAccess(MemoryAccessInstruction),
    BlockDataTransfer(BlockDataTransferInstruction),
    Branch(BranchInstruction),
    SupervisorCall(SupervisorCallInstruction),
    Breakpoint(BreakpointInstruction),
}

/// Errors that can occur when converting a raw [u32] into an [Instruction].
#[derive(Error, Debug, Clone)]
pub enum InstructionConversionError {
    #[error("Invalid instruction class")]
    InvalidInstructionClass,
}

impl TryFrom<u32> for Instruction {
    type Error = InstructionConversionError;

    fn try_from(raw_instruction: u32) -> Result<Self, Self::Error> {
        if (raw_instruction & 0x0FF000F0) == 0x01200070 {
            return Ok(Instruction::Breakpoint(raw_instruction.into()));
        }

        // Check bits [27:25] to identify the instruction class. This is how the
        // ARM processor itself differentiates these top-level instruction types.
        let op_class = (raw_instruction >> 25) & 0b111;

        match op_class {
            // Data Processing (000 or 001)
            0b000 | 0b001 => {
                Ok(Instruction::DataProcessing(raw_instruction.into()))
            }
            // Memory Access (010 or 011)
            0b010 | 0b011 => {
                Ok(Instruction::MemoryAccess(raw_instruction.into()))
            }
            // Block Data Transfer (100)
            0b100 => {
                Ok(Instruction::BlockDataTransfer(raw_instruction.into()))
            }
            // Branch (101)
            0b101 => Ok(Instruction::Branch(raw_instruction.into())),

            // Class for miscellaneous instructions, including SVC (111)
            0b111 => {
                // This class is broad. We must check bits [27:24] to be specific.
                // An SVC instruction is identified by the pattern `1111`.
                if (raw_instruction >> 24) & 0b1111 == 0b1111 {
                    Ok(Instruction::SupervisorCall(raw_instruction.into()))
                } else {
                    // This is where you would decode other instructions from this class,
                    // like coprocessor instructions. For now, we consider them invalid.
                    Err(Self::Error::InvalidInstructionClass)
                }
            }

            // All other patterns are undefined.
            _ => Err(Self::Error::InvalidInstructionClass),
        }
    }
}

impl From<Instruction> for u32 {
    fn from(value: Instruction) -> Self {
        match value {
            Instruction::DataProcessing(instr) => instr.into(),
            Instruction::MemoryAccess(instr) => instr.into(),
            Instruction::BlockDataTransfer(instr) => instr.into(),
            Instruction::Branch(instr) => instr.into(),
            Instruction::SupervisorCall(instr) => instr.into(),
            Instruction::Breakpoint(instr) => instr.into(),
        }
    }
}
