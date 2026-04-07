//! ARM Emulator - An educational ARM emulator with GUI
//!
//! This crate provides a complete ARM emulation core with support for:
//! - ARM instruction set (Data Processing, Memory Access, Branch, Multiply, etc.)
//! - Memory management with bus system
//! - CPU state management with execution states
//! - Debugging support (breakpoints, stepping)
//! - Memory-mapped peripherals (GPIO)
//!
//! # Architecture
//!
//! The emulator consists of several core components:
//! - [`Emulator`] - Main entry point coordinating CPU and memory
//! - [`Cpu`] - CPU modeling with registers and execution states
//! - [`Bus`] - Memory bus with code, SRAM, and peripheral regions
//!
//! # Usage
//!
//! ```ignore
//! use arm_emulator::Emulator;
//! use arm_emulator::cpu::Cpu;
//! use arm_emulator::memory::Bus;
//!
//! let cpu = Cpu::new();
//! let bus = Bus::new();
//! let mut emulator = Emulator::new(cpu, bus, Endian::Little);
//!
//! // Load program and step
//! emulator.load_program(&code, None, None);
//! emulator.step()?;
//! ```

#![warn(clippy::pedantic)]
#![warn(clippy::missing_const_for_fn)]

use std::collections::HashMap;

use thiserror::Error;

use crate::{
    cpu::{
        Cpu, ExecutionState, ExitStatus,
        registers::{self, PC},
    },
    execution::{ExecutableInstruction, ExecutionError},
    instructions::{
        BlockDataTransferInstruction, BranchExchangeInstruction,
        BranchInstruction, BreakpointInstruction,
        DataProcessingInstruction, Instruction,
        InstructionConversionError, MemoryAccessInstruction,
        MultiplyInstruction, MultiplyLongInstruction,
        SupervisorCallInstruction, fields::Condition,
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
#[cfg(test)]
mod tests;

const BREAKPOINT_BE_BYTES: [u8; 4] = [0xE1, 0x20, 0x00, 0x70];

/// Represents a breakpoint in the emulator.
///
/// When a breakpoint is hit, the emulator halts execution and provides
/// information about the original instruction that was replaced.
#[derive(Debug, Error, Clone)]
#[must_use]
pub struct Breakpoint {
    /// The memory address where the breakpoint is set.
    pub addr: Word,
    /// The original instruction that was at this address before patching.
    pub instruction: Instruction,
}

impl std::fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Type alias for breakpoint address to original instruction mappings.
/// Used internally to track and restore original instructions when breakpoints are removed.
pub type BreakpointMappings = HashMap<Word, u32>;

/// Main emulator struct that coordinates the CPU, memory bus, and execution.
///
/// The emulator provides the primary interface for loading programs,
/// stepping through instructions, and managing breakpoints.
#[derive(Debug)]
#[must_use]
pub struct Emulator {
    /// The emulated CPU instance.
    pub cpu: Cpu,
    /// The memory bus handling reads and writes.
    pub memory_bus: Bus,
    /// Current endianness (little or big).
    pub endian: Endian,
    /// Internal mapping of breakpoint addresses to original instructions.
    /// This is used to restore instructions when breakpoints are removed.
    pub breakpoint_destructive: BreakpointMappings,
}

// Program loading.
impl Emulator {
    /// Loads a program with a specified SRAM size.
    ///
    /// This method resets the emulator, loads the code into memory,
    /// optionally loads SRAM and external memory, and sets up the stack pointer.
    ///
    /// # Arguments
    ///
    /// * `code` - The ARM bytecode to load
    /// * `sram` - Optional static RAM data to load
    /// * `external` - Optional external memory data to load
    /// * `sram_size` - The size of SRAM to reserve in bytes
    pub fn load_program_with_sram_size(
        &mut self,
        code: &[u8],
        sram: Option<&[u8]>,
        external: Option<&[u8]>,
        sram_size: u32,
    ) {
        self.reset();
        self.load_code(code);
        if let Some(sram) = sram {
            self.load_sram(sram);
        }
        self.memory_bus.reserve_exact_sram(sram_size);

        if let Some(external) = external {
            self.load_external(external);
        }

        self.set_sp_to_default();

        tracing::trace!(
            "SRAM size: {}",
            self.memory_bus.get_read_write_memory_view().len()
        );
    }

    /// Loads a program with default 64 KiB SRAM size.
    ///
    /// Convenience method that calls `load_program_with_sram_size` with 64 KiB.
    pub fn load_program(
        &mut self,
        code: &[u8],
        sram: Option<&[u8]>,
        external: Option<&[u8]>,
    ) {
        self.load_program_with_sram_size(
            code,
            sram,
            external,
            memory::KIBIBYTE * 64, // 64 KiB
        );
    }

    /// Loads ARM bytecode into the code memory region.
    pub fn load_code(&mut self, code: &[u8]) {
        self.memory_bus.load_code(code);
    }

    /// Loads data into the SRAM region.
    pub fn load_sram(&mut self, sram: &[u8]) {
        self.memory_bus.load_sram(sram);
    }

    /// Loads data into the external memory region.
    pub fn load_external(&mut self, external: &[u8]) {
        self.memory_bus.load_external(external);
    }
}

// Creation.
impl Emulator {
    /// Create an [Emulator] with the provided [Cpu] and [Bus] and [Endian]
    pub fn new(cpu: Cpu, memory_bus: Bus, endian: Endian) -> Self {
        Self {
            cpu,
            memory_bus,
            endian,
            breakpoint_destructive: Default::default(),
        }
    }
}

// Breakpoints
impl Emulator {
    /// The core debugger logic for stepping over a breakpoint.
    /// This should be called by the UI when the user wants to step
    /// while the emulator is paused at a breakpoint.
    /// This function handles un-patching, executing one instruction,
    /// and immediately re-patching.
    pub fn step_over_breakpoint(&mut self) -> Result<(), ExecutionError> {
        // Get the address of the current breakpoint from the CPU state.
        let breakpoint_addr =
            if let ExecutionState::Breakpoint(bp) = &self.cpu.state {
                bp.addr
            } else {
                // This function should only be called when at a breakpoint.
                // If not, it's a logic error in the calling code (the UI).
                // We can choose to either do a normal step or return an error.
                // Let's be strict and assume the UI knows the state.
                // For robustness, you could fall back to `return self.step();`
                tracing::warn!(
                    "step_over_breakpoint called when not at a breakpoint."
                );
                return Ok(());
            };

        // Temporarily restore the original instruction.
        let original_instruction_raw =
            match self.breakpoint_destructive.get(&breakpoint_addr) {
                Some(&instr) => instr,
                None => {
                    // This indicates a severe logic error, but we'll handle it gracefully.
                    return Err(ExecutionError::MemoryAccessError(
                        MemoryAccessError::InvalidReadPermission {
                            addr: breakpoint_addr,
                        },
                    ));
                }
            };
        self.patch_instruction_at(
            breakpoint_addr,
            original_instruction_raw,
        )?;

        // Decode and Execute that single, original instruction.
        let original_instruction_decoded =
            self.decode(original_instruction_raw)?;

        // Temporarily set state back to Running to allow the single execution.
        self.cpu.set_running();
        self.execute_single_instruction(original_instruction_decoded)?;

        // Immediately re-patch the breakpoint instruction. The emulator is now
        // safe again. The breakpoint is active for any subsequent "Run" or "Step" command.
        self.add_breakpoint_at(breakpoint_addr)?;

        // Set the state back to Running. The next `step()` call will proceed normally
        // from the new PC value.
        self.cpu.set_running();

        Ok(())
    }

    /// Add a breakpoint to the address supplied.
    #[must_use]
    pub fn patch_breakpoint_at(
        &mut self,
        addr: Word,
    ) -> MemoryAccessResult<u32> {
        let instr = self.read32(addr)?;
        self.write32(addr, u32::from_be_bytes(BREAKPOINT_BE_BYTES))?;
        Ok(instr)
    }

    /// Patch an instruction at the address supplied. Just delegates to `write32`.
    pub fn patch_instruction_at(
        &mut self,
        addr: Word,
        instr: u32,
    ) -> MemoryAccessResult<()> {
        self.write32(addr, instr)?;
        Ok(())
    }

    /// Save a breakpoint in the list of addresses -> instructions.
    /// Save the 'original' instruction (value) at the address (key) provided.
    pub fn save_breakpoint_at(&mut self, addr: Word, instr: u32) {
        self.breakpoint_destructive.insert(addr, instr);
    }

    /// Add a breakpoint to the address supplied.
    /// Saves the original instruction, and then patches a breakpoint to that address.
    /// This operation is distructive. It will overwrite the original instruction.
    pub fn add_breakpoint_at(
        &mut self,
        addr: Word,
    ) -> MemoryAccessResult<()> {
        self.save_breakpoint_at(addr, self.read32(addr)?);
        self.patch_breakpoint_at(addr)?;

        Ok(())
    }

    /// Remove breakpoint at the address supplied.
    /// Restores the original instruction at the address given, and removes it from the cache.
    pub fn remove_breakpoint_at(
        &mut self,
        addr: Word,
    ) -> MemoryAccessResult<()> {
        match self.breakpoint_destructive.remove(&addr) {
            Some(instr) => {
                self.patch_instruction_at(addr, instr)?;
                tracing::info!(
                    "Removed breakpoint at address {addr:#X} and patched instruction"
                );
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Restore the instruction at the address supplied.
    /// Restores the original instruction at the address given, removes it from the cache,
    /// and changes the CPU state back to running, if we are currently halted on this breakpoint.
    pub fn restore_instruction_at(
        &mut self,
        addr: Word,
    ) -> MemoryAccessResult<()> {
        match self.breakpoint_destructive.remove(&addr) {
            Some(instr) => {
                // 1. Put the original instruction back in memory
                self.patch_instruction_at(addr, instr)?;

                // 2. Check if we are currently halted on this breakpoint
                if let ExecutionState::Breakpoint(bp) = &self.cpu.state {
                    if bp.addr == addr {
                        // We are removing the breakpoint we are currently stuck on.
                        // Since the original instruction is now restored,
                        // we can treat the CPU as simply "Running" (ready to execute).
                        self.cpu.set_running();
                    }
                }

                Ok(())
            }
            None => {
                Err(memory::MemoryAccessError::InvalidReadPermission {
                    addr,
                })
            }
        }
    }
}

// Getters
impl Emulator {
    /// Returns a read-only view of the memory.
    #[must_use]
    pub fn get_read_only_memory_view(&self) -> &Bytes {
        self.memory_bus.get_read_only_memory_view()
    }

    /// Returns the list of memory-mapped peripherals.
    #[must_use]
    pub fn get_mapped_peripherals(&self) -> &[MemoryMappedPeripheral] {
        self.memory_bus.get_mapped_peripherals()
    }

    /// Adds a memory-mapped peripheral to the emulator.
    pub fn add_peripheral(
        &mut self,
        mapped_peripheral: MemoryMappedPeripheral,
    ) {
        self.memory_bus.add_peripheral(mapped_peripheral);
    }
}

// Execution of the code (asm).
impl Emulator {
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory_bus.reset();
        self.breakpoint_destructive = BreakpointMappings::new();
    }

    pub fn reset_cpu(&mut self) {
        self.cpu.reset();
        self.set_sp_to_default();
    }

    fn set_sp_to_default(&mut self) {
        self.cpu.set_sp(self.memory_bus.get_sp_default_addr());
    }

    pub fn is_halted(&self) -> bool {
        self.cpu.is_halted()
    }

    pub fn is_finished(&self) -> bool {
        self.cpu.is_finished()
    }

    #[must_use]
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

    #[must_use]
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
    #[must_use]
    pub fn is_done(&self) -> bool {
        matches!(self.cpu.state, ExecutionState::FinishedExecution(_))
    }

    #[must_use]
    pub fn get_exit_status(&self) -> Option<ExitStatus> {
        match &self.cpu.state {
            ExecutionState::FinishedExecution(status) => {
                Some(status.clone())
            }
            _ => None,
        }
    }

    /// Fetch the instruction at the address of the current PC value
    #[inline]
    #[must_use]
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

    fn post_execution_update(&mut self) {
        // The PC is advanced by default. Branch instructions will override this.
        self.cpu[PC] = self.cpu[PC].wrapping_add(size_of::<Word>() as u32);
    }

    /// Step over one ASM instruction, and then yield execution.
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        match &self.cpu.state {
            ExecutionState::Running => {
                // If we are running, proceed with fetch-decode-execute
            }
            ExecutionState::Breakpoint(breakpoint) => {
                // If we are at a breakpoint, stop and signal the UI by returning an error.
                // DO NOT execute anything. The UI must call `step_over_breakpoint`
                return Err(ExecutionError::Breakpoint(
                    breakpoint.clone(),
                ));
            }
            ExecutionState::FinishedExecution(_) => {
                // Program is done, nothing more to do.
                return Ok(());
            }
            // Handle other states like Exception, SupervisorCall
            _ => {}
        }

        // Fetch
        let fetch = self.fetch()?;

        // Decode
        let decode = self.decode(fetch)?;

        if self.cpu.should_execute(decode) {
            // Execute
            (|| {
                let r = self.execute_single_instruction(decode);
                tracing::info!("Step result: {r:?}");
                r
            })()?;
        } else {
            // Move the PC regardless.
            self.post_execution_update();
        }

        Ok(())
    }

    fn execute_single_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), ExecutionError> {
        let original_pc = self.cpu.pc();

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
                // We must NOT auto-increment, even if the branch target is the same as current PC.
                return Ok(());
            }
            Instruction::BranchExchange(instr) => {
                self.execute_branch_exchange_instruction(instr)?;
                // BX instructions determine the next PC explicitly.
                return Ok(());
            }
            Instruction::SupervisorCall(instr) => {
                self.execute_supervisor_call_instruction(instr)?;
            }
            Instruction::Multiply(instr) => {
                self.execute_multiply_instruction(instr)?;
            }
            Instruction::MultiplyLong(instr) => {
                self.execute_multiply_long_instruction(instr)?;
            }
            Instruction::Breakpoint(instr) => {
                self.execute_breakpoint_instruction(instr)?;
                // Breakpoints shouldn't advance PC either (they halt).
                return Ok(());
            }
        }
        if self.cpu.pc() == original_pc {
            self.post_execution_update();
        }

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
        instr.execute_with(self)
    }

    fn execute_branch_instruction(
        &mut self,
        instr: BranchInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Branch instruction: {instr:?}");
        instr.execute_with(self)
    }

    fn execute_branch_exchange_instruction(
        &mut self,
        instr: BranchExchangeInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Branch exchange instruction: {instr:?}");
        instr.execute_with(self)
    }

    fn execute_supervisor_call_instruction(
        &mut self,
        instr: SupervisorCallInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Supervisor call instruction: {instr:?}");
        instr.execute_with(self)
    }

    fn execute_multiply_instruction(
        &mut self,
        instr: MultiplyInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Multiply instruction: {instr:?}");
        instr.execute_with(self)
    }

    fn execute_multiply_long_instruction(
        &mut self,
        instr: MultiplyLongInstruction,
    ) -> Result<(), ExecutionError> {
        tracing::trace!("Multiply Long instruction: {instr:?}");
        instr.execute_with(self)
    }

    fn execute_breakpoint_instruction(
        &mut self,
        _instr: BreakpointInstruction, // Don't need the instruction here
    ) -> Result<(), ExecutionError> {
        let current_pc = self.cpu.pc();
        tracing::warn!(
            "Breakpoint instruction hit at PC: {:#X}",
            current_pc
        );

        // Look up the original instruction from our saved map.
        let original_instr_raw = self
            .breakpoint_destructive
            .get(&current_pc)
            .ok_or(MemoryAccessError::InvalidReadPermission {
                addr: current_pc,
            })?;

        let original_instr_decoded = self.decode(*original_instr_raw)?;

        // Change the CPU state to Breakpoint. This is the signal to the UI.
        self.cpu.set_breakpoint(Breakpoint {
            addr: current_pc,
            instruction: original_instr_decoded,
        });

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

    #[must_use]
    pub fn max_address(&self) -> u32 {
        u32::MAX
    }
}

impl Cpu {
    fn should_execute(&self, instr: Instruction) -> bool {
        match instr.cond() {
            Condition::AL => self.al(),
            Condition::EQ => self.eq(),
            Condition::NE => self.ne(),
            Condition::HS => self.hs(),
            Condition::LO => self.lo(),
            Condition::VS => self.vs(),
            Condition::VC => self.vc(),
            Condition::HI => self.hi(),
            Condition::LS => self.ls(),
            Condition::GE => self.ge(),
            Condition::LT => self.lt(),
            Condition::GT => self.gt(),
            Condition::LE => self.le(),
            Condition::MI => self.mi(),
            Condition::PL => self.pl(),
            Condition::NV => self.nv(),
        }
    }
}

pub mod prelude;
