//! Common types and re-exports for the emulator crate.
//!
//! This module re-exports the most commonly used types, providing a convenient
//! import path for users of the emulator library.
//!
//! # Usage
//!
//! ```ignore
//! use emulator::prelude::*;
//!
//! let cpu = Cpu::new();
//! let bus = Bus::new(1024, 1024, 0);
//! ```

pub use crate::Emulator;
pub use crate::cpu::{Cpu, CpuError, registers};
pub use crate::execution::ExecutionError;
pub use crate::instructions::{
    BlockDataTransferInstruction, BranchInstruction,
    DataProcessingInstruction, Instruction, MemoryAccessInstruction,
    SupervisorCallInstruction,
};
pub use crate::memory::{Bus, Endian, Peripheral};
