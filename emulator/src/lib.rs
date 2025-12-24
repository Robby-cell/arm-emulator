#![warn(clippy::pedantic)]

use std::collections::HashMap;

use thiserror::Error;

use crate::{
    cpu::{
        Cpu, ExecutionState,
        registers::{self, PC},
    },
    execution::{ExecutableInstruction, ExecutionError},
    instructions::{
        BlockDataTransferInstruction, BranchExchangeInstruction,
        BranchInstruction, BreakpointInstruction,
        DataProcessingInstruction, Instruction,
        InstructionConversionError, MemoryAccessInstruction,
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

#[derive(Debug, Error, Clone)]
#[must_use]
pub struct Breakpoint {
    pub addr: Word,
    pub instruction: Instruction,
}

impl std::fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type BreakpointMappings = HashMap<Word, u32>;

#[derive(Debug)]
#[must_use]
pub struct Emulator {
    pub cpu: Cpu,
    pub memory_bus: Bus,
    pub endian: Endian,
    pub breakpoint_destructive: BreakpointMappings,
}

impl Emulator {
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

        let sram_end = Bus::SRAM_BEGIN
            + (self.memory_bus.get_read_write_memory_view().len() as u32);
        self.cpu.set_sp(sram_end);
        tracing::trace!(
            "SRAM size: {}",
            self.memory_bus.get_read_write_memory_view().len()
        );

        tracing::info!("Loaded program. SP reset to {:#X}", self.cpu.sp());
    }

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
            Bus::SRAM_SIZE,
        );
    }

    pub fn load_code(&mut self, code: &[u8]) {
        self.memory_bus.load_code(code);
    }

    pub fn load_sram(&mut self, sram: &[u8]) {
        self.memory_bus.load_sram(sram);
    }

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

    pub fn patch_instruction_at(
        &mut self,
        addr: Word,
        instr: u32,
    ) -> MemoryAccessResult<()> {
        self.write32(addr, instr)?;
        Ok(())
    }

    pub fn save_breakpoint_at(&mut self, addr: Word, instr: u32) {
        self.breakpoint_destructive.insert(addr, instr);
    }

    pub fn add_breakpoint_at(
        &mut self,
        addr: Word,
    ) -> MemoryAccessResult<()> {
        self.save_breakpoint_at(addr, self.read32(addr)?);
        self.patch_breakpoint_at(addr)?;

        Ok(())
    }

    pub fn restore_instruction_at(
        &mut self,
        addr: Word,
    ) -> MemoryAccessResult<()> {
        match self.breakpoint_destructive.remove(&addr) {
            Some(instr) => {
                self.patch_instruction_at(addr, instr)?;
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
    #[must_use]
    pub fn get_read_only_memory_view(&self) -> &Bytes {
        self.memory_bus.get_read_only_memory_view()
    }

    #[must_use]
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
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory_bus.reset();
        self.breakpoint_destructive = BreakpointMappings::new();
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
        true
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
            }
            Instruction::BranchExchange(instr) => {
                self.execute_branch_exchange_instruction(instr)?;
            }
            Instruction::SupervisorCall(instr) => {
                self.execute_supervisor_call_instruction(instr)?;
            }
            Instruction::Breakpoint(instr) => {
                self.execute_breakpoint_instruction(instr)?;
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
        Ok(())
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
