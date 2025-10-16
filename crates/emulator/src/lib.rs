use thiserror::Error;

use crate::{
    cpu::{Cpu, registers},
    execution::ExecutableInstruction,
    instructions::{
        BlockDataTransferInstruction, BranchInstruction,
        DataProcessingInstruction, Instruction,
        InstructionConversionError, MemoryAccessInstruction,
        SupervisorCallInstruction,
    },
    memory::{
        Bus, Bytes, Endian, MemoryAccessError, MemoryAccessResult,
        MemoryMappedPeripheral, Word,
    },
};

pub mod cpu;
mod execution;
pub mod instructions;
pub mod memory;
pub mod peripherals;
pub mod system;

#[derive(Debug)]
pub struct Emulator {
    pub cpu: Cpu,
    pub memory_bus: Bus,
    pub endian: Endian,
}

#[derive(Debug, Error, Clone)]
pub enum ExecutionError {
    #[error(
        "breakpoint reached at instruction {instruction:?} at address {addr:#X}"
    )]
    Breakpoint { addr: u32, instruction: Instruction },

    #[error("memory access error: {0}")]
    MemoryAccessError(#[from] MemoryAccessError),

    #[error("illegal instruction, could not decode instruction: {0}")]
    InstructionConversionError(#[from] InstructionConversionError),
}

// Creation.
impl Emulator {
    /// Create an [Emulator] with the provided [Cpu] and [Bus] and [Endian]
    pub fn new(cpu: Cpu, memory_bus: Bus, endian: Endian) -> Self {
        Self {
            cpu,
            memory_bus,
            endian,
        }
    }
}

// Getters
impl Emulator {
    pub fn get_read_only_memory_view(&self) -> &Bytes {
        self.memory_bus.get_read_only_memory_view()
    }

    pub fn get_mapped_peripherals(&self) -> &[MemoryMappedPeripheral] {
        self.memory_bus.get_mapped_peripherals()
    }

    pub fn add_peripheral(
        &mut self,
        mapped_peripheral: MemoryMappedPeripheral,
    ) {
        self.memory_bus.add_peripheral(mapped_peripheral);
    }
}

// Execution of the code (asm).
impl Emulator {
    pub fn read32(&self, addr: Word) -> MemoryAccessResult<u32> {
        match self.endian {
            Endian::Big => self.memory_bus.read32_be(addr),
            Endian::Little => self.memory_bus.read32_le(addr),
        }
    }

    pub fn write32(
        &mut self,
        addr: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        match self.endian {
            Endian::Big => self.memory_bus.write32_be(addr, value),
            Endian::Little => self.memory_bus.write32_le(addr, value),
        }
    }

    /// Is the emulation finished execution?
    /// Has it returned from the main/_start function?
    pub fn is_done(&self) -> bool {
        true
    }

    /// Step over one ASM instruction, and then yield execution.
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        let instruction = self.read32(self.cpu[registers::PC])?;

        self.execute_single_instruction(instruction.try_into()?)
    }

    fn execute_single_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), ExecutionError> {
        match instruction {
            Instruction::DataProcessing(instr) => {
                self.execute_data_processing_instruction(instr)
            }
            Instruction::MemoryAccess(instr) => {
                self.execute_memory_access_instruction(instr)
            }
            Instruction::BlockDataTransfer(instr) => {
                self.execute_block_data_transfer_instruction(instr)
            }
            Instruction::Branch(instr) => {
                self.execute_branch_instruction(instr)
            }
            Instruction::SupervisorCall(instr) => {
                self.execute_supervisor_call_instruction(instr)
            }
        }
    }

    fn execute_data_processing_instruction(
        &mut self,
        instr: DataProcessingInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Data processing instruction: {instr:?}");
        instr.execute_with(self)
    }

    fn execute_memory_access_instruction(
        &mut self,
        instr: MemoryAccessInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Memory access instruction: {instr:?}");
        Ok(())
    }
    fn execute_block_data_transfer_instruction(
        &mut self,
        instr: BlockDataTransferInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Block data transfer instruction: {instr:?}");
        Ok(())
    }
    fn execute_branch_instruction(
        &mut self,
        instr: BranchInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Branch instruction: {instr:?}");
        Ok(())
    }
    fn execute_supervisor_call_instruction(
        &mut self,
        instr: SupervisorCallInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Supervisor call instruction: {instr:?}");
        Ok(())
    }

    /// Just execute without stopping.
    /// If there is an error (illegal instruction, trap, etc.) then stop.
    pub fn execute(&mut self) -> Result<(), ExecutionError> {
        Ok(())
    }
}
