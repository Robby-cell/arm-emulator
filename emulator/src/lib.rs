use thiserror::Error;

use crate::{
    cpu::{
        Cpu, ExecutionState,
        registers::{self, PC},
    },
    execution::{ExecutableInstruction, ExecutionError},
    instructions::{
        BlockDataTransferInstruction, BranchInstruction,
        DataProcessingInstruction, Instruction,
        InstructionConversionError, MemoryAccessInstruction,
        SupervisorCallInstruction,
    },
    memory::{
        Bus, Bytes, Endian, MemoryAccessResult, MemoryMappedPeripheral,
        Word,
    },
};

pub mod cpu;
mod execution;
pub mod instructions;
pub mod memory;
pub mod peripherals;
pub mod system;
#[cfg(test)]
mod tests;

const BREAKPOINT_BE_BYTES: [u8; 4] = [0xE1, 0x20, 0x00, 0x70];

#[derive(Debug, Error, Clone)]
pub struct Breakpoint {
    pub addr: Word,
    pub instruction: Instruction,
}

impl std::fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct Emulator {
    pub cpu: Cpu,
    pub memory_bus: Bus,
    pub endian: Endian,
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

// Breakpoints
impl Emulator {
    /// Add a breakpoint to the address supplied.
    pub fn patch_breakpoint_at(
        &mut self,
        addr: Word,
    ) -> MemoryAccessResult<u32> {
        let instr = self.read32(addr)?;
        self.write32(addr, u32::from_be_bytes(BREAKPOINT_BE_BYTES))?;
        Ok(instr)
    }

    pub fn patch_instruction_at(
        &mut self,
        addr: Word,
        instr: u32,
    ) -> MemoryAccessResult<()> {
        self.write32(addr, instr)?;
        Ok(())
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

    pub fn read_byte(&self, addr: Word) -> MemoryAccessResult<u8> {
        match self.endian {
            Endian::Big => self.memory_bus.read_byte_be(addr),
            Endian::Little => self.memory_bus.read_byte_le(addr),
        }
    }

    pub fn write_byte(
        &mut self,
        addr: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        match self.endian {
            Endian::Big => self.memory_bus.write_byte_be(addr, value),
            Endian::Little => self.memory_bus.write_byte_le(addr, value),
        }
    }

    /// Is the emulation finished execution?
    /// Has it returned from the main/_start function?
    pub fn is_done(&self) -> bool {
        true
    }

    /// Fetch the instruction at the address of the current PC value
    #[inline]
    pub fn fetch(&self) -> MemoryAccessResult<u32> {
        self.read32(self.cpu.register(registers::PC))
    }

    /// Decode the instruction representation given.
    #[inline]
    pub fn decode(
        &self,
        instr: u32,
    ) -> Result<Instruction, InstructionConversionError> {
        instr.try_into()
    }

    fn post_execution_update(&mut self) -> Result<(), ExecutionError> {
        self.cpu[PC] += size_of::<Word>() as u32;
        Ok(())
    }

    /// Step over one ASM instruction, and then yield execution.
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        match &self.cpu.state {
            ExecutionState::Running => {}
            ExecutionState::Breakpoint(breakpoint) => {
                return Err(ExecutionError::Breakpoint(
                    breakpoint.clone(),
                ));
            }
            ExecutionState::Exception(exception) => {
                return Err(ExecutionError::Exception(exception.clone()));
            }
            ExecutionState::FinishedExecution(_) => {
                return Ok(());
            }
            ExecutionState::SupervisorCall(_) => {}
        }

        // Fetch
        let fetch = self.fetch()?;

        // Decode
        let decode = self.decode(fetch)?;

        // Execute
        self.execute_single_instruction(decode)?;

        Ok(())
    }

    fn execute_single_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), ExecutionError> {
        match instruction {
            Instruction::DataProcessing(instr) => {
                self.execute_data_processing_instruction(instr)?;
            }
            Instruction::MemoryAccess(instr) => {
                self.execute_memory_access_instruction(instr)?;
            }
            Instruction::BlockDataTransfer(instr) => {
                self.execute_block_data_transfer_instruction(instr)?;
            }
            Instruction::Branch(instr) => {
                self.execute_branch_instruction(instr)?;
            }
            Instruction::SupervisorCall(instr) => {
                self.execute_supervisor_call_instruction(instr)?;
            }
        }
        self.post_execution_update()?;
        Ok(())
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
        instr.execute_with(self)
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
        instr.execute_with(self)
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

impl Emulator {
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    pub fn use_little_endian(&mut self) {
        self.set_endian(Endian::Little);
    }

    pub fn use_big_endian(&mut self) {
        self.set_endian(Endian::Big);
    }

    pub fn max_address(&self) -> u32 {
        u32::MAX
    }
}

pub mod prelude {
    pub use crate::Emulator;
    pub use crate::cpu::{Cpu, CpuError, registers};
    pub use crate::execution::ExecutionError;
    pub use crate::instructions::{
        BlockDataTransferInstruction, BranchInstruction,
        DataProcessingInstruction, Instruction, MemoryAccessInstruction,
        SupervisorCallInstruction,
    };
    pub use crate::memory::{Bus, Endian, Peripheral};
}
