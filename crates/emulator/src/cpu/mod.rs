//! CPU state management module.
//!
//! This module provides the core CPU state representation for the ARM emulator,
//! handling:
//! - General-purpose registers (R0-R15)
//! - Program status registers (CPSR and SPSR)
//! - Processor modes (User, FIQ, IRQ, Supervisor, Abort, Undefined, System)
//! - Condition flags (N, Z, C, V, I, F, T)
//! - Execution state tracking (running, halted, breakpoints, exceptions)
//!
//! The [`Cpu`] struct is the central type, providing mutable access to CPU state
//! while maintaining ARM architecture semantics.

use std::ops::{Index, IndexMut};

use thiserror::Error;

use crate::{Breakpoint, memory::MemoryAccessError};

mod display;
#[cfg(test)]
mod tests;

impl Cpu {
    /// Default CPSR value after reset (0x000000D3).
    ///
    /// This value sets:
    /// - Mode bits to Supervisor (0b10011)
    /// - I flag set (IRQs disabled)
    /// - F flag set (FIQs disabled)
    /// - T flag clear (ARM state)
    pub const DEFAULT_CPSR: CpuFlags = CpuFlags(0x000000D3);

    /// Bitmask for the N (Negative) flag (bit 31).
    pub const N_FLAG: CpuFlags = CpuFlags(1 << 31);
    /// Bitmask for the Z (Zero) flag (bit 30).
    pub const Z_FLAG: CpuFlags = CpuFlags(1 << 30);
    /// Bitmask for the C (Carry) flag (bit 29).
    pub const C_FLAG: CpuFlags = CpuFlags(1 << 29);
    /// Bitmask for the V (Overflow) flag (bit 28).
    pub const V_FLAG: CpuFlags = CpuFlags(1 << 28);

    /// Bitmask for the I (IRQ Disable) flag (bit 7).
    pub const I_FLAG: CpuFlags = CpuFlags(1 << 7);
    /// Bitmask for the F (FIQ Disable) flag (bit 6).
    pub const F_FLAG: CpuFlags = CpuFlags(1 << 6);
    /// Bitmask for the T (Thumb State) flag (bit 5).
    pub const T_FLAG: CpuFlags = CpuFlags(1 << 5);
    /// Bitmask for the mode bits (bits 0-4).
    pub const MODE_MASK: CpuFlags = CpuFlags(0x1F);
}

/// Processor execution mode.
///
/// ARM processors have several privilege levels called modes. These modes
/// determine which registers are accessible and what operations are allowed.
/// Each privileged mode (except System) has its own Stack Pointer (SP) and
/// Link Register (LR), and some modes have their own Saved Program Status
/// Register (SPSR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[must_use]
pub enum Mode {
    /// User mode - unprivileged mode for applications
    User = 0b10000,
    /// FIQ (Fast Interrupt Request) mode - high priority interrupts
    Fiq = 0b10001,
    /// IRQ (Interrupt Request) mode - standard interrupt handling
    Irq = 0b10010,
    /// Supervisor mode - software interrupt or reset
    Supervisor = 0b10011,
    /// Abort mode - memory access violation handling
    Abort = 0b10111,
    /// Undefined mode - undefined instruction handling
    Undefined = 0b11011,
    /// System mode - privileged user mode
    System = 0b11111,
}

impl From<Mode> for CpuFlags {
    fn from(value: Mode) -> Self {
        Self(value as u32)
    }
}

/// The program is ready to exit, with the given `exit_code`.
///
/// Returned by the CPU when a program completes execution normally.
/// The exit code can be retrieved from the contained [`exit_code`](ExitStatus::exit_code) field.
#[derive(Debug, Clone)]
#[repr(transparent)]
#[must_use]
pub struct ExitStatus {
    /// The exit code of the program.
    /// i.e. the equival of `return 0`
    pub exit_code: i32,
}

/// A supervisor call (SVC) is currently active.
///
/// Also known as a software interrupt (SWI), this is triggered when the CPU
/// executes an SVC instruction. The supervisor call contains a `code` that
/// identifies the specific service being requested.
#[derive(Debug, Clone)]
#[repr(transparent)]
#[must_use]
pub struct SupervisorCall {
    /// The code of the active supervisor call.
    pub code: u32,
}

/// An exception that occurred during execution.
///
/// Exceptions include memory access violations, CPU errors, and other
/// error conditions. Unlike [`SupervisorCall`], this represents error
/// conditions that cause the normal execution flow to be interrupted.
///
/// # Variants
///
/// - [`MemoryAccess`](Exception::MemoryAccess): Memory access violation
/// - [`CpuError`](Exception::CpuError): CPU-level error condition
#[derive(Debug, Error, Clone)]
pub enum Exception {
    #[error("memory access violation: {0}")]
    MemoryAccess(#[from] MemoryAccessError),

    #[error("cpu error: {0}")]
    CpuError(#[from] CpuError),
}

/// Execution state of the CPU.
///
/// Represents what the CPU is currently doing or what has caused it to stop.
/// This state is used to control the emulation loop and determine whether
/// execution should continue, pause, or terminate.
///
/// # Variants
///
/// - [`Halted`](ExecutionState::Halted): Initial/default state, CPU is stopped
/// - [`Running`](ExecutionState::Running): CPU is actively executing instructions
/// - [`Breakpoint`](ExecutionState::Breakpoint): A breakpoint was hit
/// - [`Exception`](ExecutionState::Exception): An exception occurred
/// - [`FinishedExecution`](ExecutionState::FinishedExecution): Program completed
/// - [`SupervisorCall`](ExecutionState::SupervisorCall): SVC instruction being handled
#[derive(Debug, Default, Clone, derive_more::From)]
#[must_use]
pub enum ExecutionState {
    /// The program is currently executing
    #[default]
    Halted,

    Running,

    /// This is an active [`Breakpoint`].
    Breakpoint(Breakpoint),

    /// There is an active [`Exception`].
    Exception(Exception),

    /// Finished executing, and returned the exit code.
    FinishedExecution(ExitStatus),

    /// Supervisor call handler (SVC)
    SupervisorCall(SupervisorCall),
}

/// Error when converting CPSR bits to a valid [`Mode`].
///
/// This error should never occur in practice because the CPU struct ensures
/// mode bits are always set to valid values. This is primarily a safety check
/// for the [`TryFrom`] implementation.
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum ModeError {
    /// The mode bits in CPSR did not match any known mode
    #[error("invalid mode bits")]
    InvalidModeBits,
}

impl TryFrom<CpuFlags> for Mode {
    type Error = ModeError;

    fn try_from(value: CpuFlags) -> Result<Self, Self::Error> {
        let value = value.0;
        match value as u8 {
            0b10000 => Ok(Mode::User),
            0b10001 => Ok(Mode::Fiq),
            0b10010 => Ok(Mode::Irq),
            0b10011 => Ok(Mode::Supervisor),
            0b10111 => Ok(Mode::Abort),
            0b11011 => Ok(Mode::Undefined),
            0b11111 => Ok(Mode::System),
            _ => Err(Self::Error::InvalidModeBits),
        }
    }
}

/// CPU-specific errors that can occur during emulation.
///
/// These errors represent runtime conditions that cause the CPU to halt
/// or transition to an exception state.
#[derive(Debug, Error, Clone)]
pub enum CpuError {
    /// Attempted to access SPSR in an unprivileged mode (User or System)
    #[error("unprivileged access")]
    UnprivilegedAccess,

    /// Attempted to access invalid or protected memory address
    #[error("segmentation fault ({addr:#X})")]
    Segfault { addr: u32 },

    /// Attempted to access memory with invalid alignment
    #[error("unaligned memory access")]
    UnalignedAccess,
}

/// CPU flags stored in the Program Status Register (PSR).
///
/// This type wraps a u32 and represents the condition flags and control bits
/// in the CPSR (Current Program Status Register) or SPSR (Saved Program Status Register).
///
/// # Flags
///
/// - N (bit 31): Negative/Less than flag
/// - Z (bit 30): Zero flag
/// - C (bit 29): Carry flag
/// - V (bit 28): Overflow flag
/// - I (bit 7): IRQ disable
/// - F (bit 6): FIQ disable
/// - T (bit 5): Thumb state
/// - Mode bits (bits 0-4): Processor mode
///
/// Implements various bitwise operations for flag manipulation.
#[derive(
    Default,
    Copy,
    Clone,
    Eq,
    PartialEq,
    derive_more::From,
    derive_more::Into,
    derive_more::BitAnd,
    derive_more::BitOr,
    derive_more::BitAndAssign,
    derive_more::BitOrAssign,
    derive_more::Display,
    derive_more::Binary,
    derive_more::Not,
)]
#[repr(transparent)]
pub struct CpuFlags(u32);

/// Holds the state of the CPU, including the registers and CPSR.
///
/// This struct represents the complete state of an ARM processor, including:
/// - 16 general-purpose registers (R0-R15)
/// - Current Program Status Register (CPSR)
/// - Saved Program Status Registers (SPSR) for each privileged mode
/// - Current execution state
///
/// This is a low-level representation of the ARM CPU state.
/// Other crates will build on top of this to provide a full CPU emulator.
///
/// # Registers
///
/// - R0-R12: General-purpose registers
/// - R13 (SP): Stack Pointer
/// - R14 (LR): Link Register
/// - R15 (PC): Program Counter
///
/// # Example
///
/// ```
/// use emulator::cpu::Cpu;
///
/// let mut cpu = Cpu::new();
/// cpu.set_pc(0x1000);  // Set program counter
/// cpu.set_sp(0x8000);  // Set stack pointer
/// ```
#[derive(Clone)]
#[must_use]
pub struct Cpu {
    /// General-purpose registers R0-R15
    pub registers: [u32; 16],

    /// Current Program Status Register, as
    /// [specified][https://documentation-service.arm.com/static/5f8db1f7f86e16515cdba175] in the ARM architecture
    pub cpsr: CpuFlags,

    /// SPSR for Supervisor mode
    pub spsr_svc: CpuFlags,
    /// SPSR for Abort mode
    pub spsr_abt: CpuFlags,
    /// SPSR for Undefined mode
    pub spsr_und: CpuFlags,
    /// SPSR for IRQ mode
    pub spsr_irq: CpuFlags,
    /// SPSR for FIQ mode
    pub spsr_fiq: CpuFlags,

    /// Current execution state
    pub state: ExecutionState,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: [0; _],
            cpsr: Cpu::DEFAULT_CPSR,
            spsr_svc: Default::default(),
            spsr_abt: Default::default(),
            spsr_und: Default::default(),
            spsr_irq: Default::default(),
            spsr_fiq: Default::default(),
            state: Default::default(),
        }
    }
}

impl Index<u8> for Cpu {
    type Output = u32;

    fn index(&self, index: u8) -> &Self::Output {
        &self.registers[index as usize]
    }
}

impl IndexMut<u8> for Cpu {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.registers[index as usize]
    }
}

/// Register indices for the CPU.
pub mod registers {
    macro_rules! create_index {
        [$($name:ident = $index:expr),* $(,)?] => {
            $(pub const $name : u8 = $index);*;
        }
    }

    create_index![
        R0 = 0,
        R1 = 1,
        R2 = 2,
        R3 = 3,
        R4 = 4,
        R5 = 5,
        R6 = 6,
        R7 = 7,
        R8 = 8,
        R9 = 9,
        R10 = 10,
        R11 = 11,
        R12 = 12,
        R13 = 13,
        SP = R13,
        R14 = 14,
        LR = R14,
        R15 = 15,
        PC = R15,
    ];
}

impl Cpu {
    pub fn new() -> Self {
        let cpu = Default::default();
        tracing::trace!("Created new CPU: {cpu:?}");
        cpu
    }

    pub fn is_halted(&self) -> bool {
        matches!(
            self.state,
            ExecutionState::Halted | ExecutionState::FinishedExecution(_)
        )
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, ExecutionState::FinishedExecution(_))
    }

    pub fn set_halted(&mut self) {
        self.state = ExecutionState::Halted;
    }

    pub fn set_running(&mut self) {
        self.state = ExecutionState::Running;
    }

    pub fn set_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.state = breakpoint.into();
    }

    pub fn set_exception(&mut self, error: Exception) {
        self.state = error.into();
    }

    pub fn set_exit(&mut self, status: ExitStatus) {
        self.state = status.into();
    }

    pub fn set_supervisor_call(&mut self, svc: SupervisorCall) {
        self.state = svc.into();
    }

    /// Gets the current processor mode.
    pub fn mode(&self) -> Mode {
        match Mode::try_from(self.cpsr & Self::MODE_MASK) {
            Ok(mode) => {
                tracing::trace!("Current CPU mode: {mode:?}");
                mode
            }
            Err(_) => {
                tracing::error!(
                    "Invalid mode bits in CPSR: {:#b}",
                    self.cpsr & Self::MODE_MASK
                );
                unreachable!(
                    "Invalid mode bits in CPSR: {:#b}",
                    self.cpsr & Self::MODE_MASK
                )
            }
        }
    }

    /// Sets the processor mode.
    pub fn set_mode(&mut self, mode: Mode) {
        // Clear the current mode bits
        self.cpsr &= !Self::MODE_MASK;
        // Set the new mode bits
        self.cpsr |= Into::<CpuFlags>::into(mode);
    }

    /// Sets the I (IRQ Disable) flag.
    pub fn set_i(&mut self, value: bool) {
        self.set_flag(Self::I_FLAG, value);
    }

    /// Reads the I (IRQ Disable) flag.
    #[must_use]
    pub fn i(&self) -> bool {
        self.flag(Self::I_FLAG)
    }

    /// Sets the F (FIQ Disable) flag.
    pub fn set_f(&mut self, value: bool) {
        self.set_flag(Self::F_FLAG, value);
    }

    /// Reads the F (FIQ Disable) flag.
    #[must_use]
    pub fn f(&self) -> bool {
        self.flag(Self::F_FLAG)
    }

    /// Sets the T (Thumb State) bit.
    pub fn set_t(&mut self, value: bool) {
        self.set_flag(Self::T_FLAG, value);
    }

    /// Reads the T (Thumb State) bit.
    #[must_use]
    pub fn t(&self) -> bool {
        self.flag(Self::T_FLAG)
    }

    /// Returns a mutable reference to the SPSR for the current mode.
    /// Panics if the current mode is User or System, as they don't have an SPSR.
    #[must_use]
    pub fn spsr_mut(&mut self) -> Result<&mut CpuFlags, CpuError> {
        match self.mode() {
            Mode::Supervisor => Ok(&mut self.spsr_svc),
            Mode::Abort => Ok(&mut self.spsr_abt),
            Mode::Undefined => Ok(&mut self.spsr_und),
            Mode::Irq => Ok(&mut self.spsr_irq),
            Mode::Fiq => Ok(&mut self.spsr_fiq),
            Mode::User | Mode::System => {
                tracing::error!(
                    "Unprivileged access attempt. In user/system mode."
                );
                Err(CpuError::UnprivilegedAccess)
            }
        }
    }

    /// Returns a read-only reference to the SPSR for the current mode.
    #[must_use]
    pub fn spsr(&self) -> Result<CpuFlags, CpuError> {
        match self.mode() {
            Mode::Supervisor => Ok(self.spsr_svc),
            Mode::Abort => Ok(self.spsr_abt),
            Mode::Undefined => Ok(self.spsr_und),
            Mode::Irq => Ok(self.spsr_irq),
            Mode::Fiq => Ok(self.spsr_fiq),
            Mode::User | Mode::System => {
                tracing::error!(
                    "Unprivileged access attempt. In user/system mode."
                );
                Err(CpuError::UnprivilegedAccess)
            }
        }
    }

    /// Resets the CPU to its default user mode state.
    pub fn reset(&mut self) {
        self.reset_registers();
        self.reset_cpsr();
        self.state = ExecutionState::Halted;
    }

    /// Zeroes all registers.
    pub fn reset_registers(&mut self) {
        self.registers = [0; _];
    }

    /// Resets the CPSR to its default value [Self::DEFAULT_CPSR].
    pub fn reset_cpsr(&mut self) {
        self.set_cpsr(Self::DEFAULT_CPSR);
    }

    /// Assigns a new value to the CPSR.
    pub fn set_cpsr(&mut self, cpsr: CpuFlags) {
        self.cpsr = cpsr;
    }

    /// Sets a flag in the CPSR to a given value.
    pub fn set_flag(&mut self, flag: CpuFlags, value: bool) {
        if value {
            self.cpsr |= flag;
        } else {
            self.cpsr &= !flag;
        }
    }

    /// Reads a flag from the CPSR.
    #[must_use]
    pub fn flag(&self, flag: CpuFlags) -> bool {
        (self.cpsr & flag) == flag
    }

    /// Sets the N flag (Negative).
    /// After an operation, if the result is negative (i.e., the most significant bit is set),
    /// The N flag is set to 1; otherwise, it is cleared to 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use emulator::cpu::Cpu;
    /// let mut cpu = Cpu::new();
    /// assert!(!cpu.n());
    /// cpu.set_n(true);
    /// assert!(cpu.n());
    /// ```
    pub fn set_n(&mut self, value: bool) {
        self.set_flag(Self::N_FLAG, value);
    }

    /// Reads the N flag (Negative).
    #[must_use]
    pub fn n(&self) -> bool {
        self.flag(Self::N_FLAG)
    }

    /// Sets the Z flag (Zero).
    /// After an operation, if the result is zero (i.e., all bits are 0),
    /// The Z flag is set to 1; otherwise, it is cleared to 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use emulator::cpu::Cpu;
    /// let mut cpu = Cpu::new();
    /// assert!(!cpu.z());
    /// cpu.set_z(true);
    /// assert!(cpu.z());
    /// ```
    pub fn set_z(&mut self, value: bool) {
        self.set_flag(Self::Z_FLAG, value);
    }

    /// Reads the Z flag (Zero).
    #[must_use]
    pub fn z(&self) -> bool {
        self.flag(Self::Z_FLAG)
    }

    /// Sets the
    /// [C flag](https://developer.arm.com/documentation/100076/0200/a32-t32-instruction-set-reference/condition-codes/carry-flag)
    /// (Carry).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use emulator::cpu::Cpu;
    /// let mut cpu = Cpu::new();
    /// assert!(!cpu.n());
    /// cpu.set_n(true);
    /// assert!(cpu.n());
    /// ```
    pub fn set_c(&mut self, value: bool) {
        self.set_flag(Self::C_FLAG, value);
    }

    /// Reads the
    /// [C flag](https://developer.arm.com/documentation/100076/0200/a32-t32-instruction-set-reference/condition-codes/carry-flag)
    /// (Carry).
    #[must_use]
    pub fn c(&self) -> bool {
        self.flag(Self::C_FLAG)
    }

    /// Sets the
    /// [V flag](https://developer.arm.com/documentation/100076/0200/a32-t32-instruction-set-reference/condition-codes/overflow-flag)
    /// (Overflow).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use emulator::cpu::Cpu;
    /// let mut cpu = Cpu::new();
    /// assert!(!cpu.v());
    /// cpu.set_v(true);
    /// assert!(cpu.v());
    /// ```
    pub fn set_v(&mut self, value: bool) {
        self.set_flag(Self::V_FLAG, value);
    }

    /// Reads the
    /// [V flag](https://developer.arm.com/documentation/100076/0200/a32-t32-instruction-set-reference/condition-codes/overflow-flag)
    /// (Overflow).
    #[must_use]
    pub fn v(&self) -> bool {
        self.flag(Self::V_FLAG)
    }
}

/// Condition tests for the cpu.
///
/// [Explanation from the ARM documentation](https://developer.arm.com/documentation/dui0473/m/condition-codes/condition-code-suffixes-and-related-flags)
impl Cpu {
    /// Equal.
    #[must_use]
    pub fn eq(&self) -> bool {
        self.z()
    }

    /// Not equal.
    #[must_use]
    pub fn ne(&self) -> bool {
        !self.z()
    }

    /// Unsigned higher or same (or carry set).
    #[must_use]
    pub fn hs(&self) -> bool {
        self.cs()
    }

    #[must_use]
    pub fn cs(&self) -> bool {
        self.c()
    }

    /// Unsigned lower (or carry clear).
    #[must_use]
    pub fn lo(&self) -> bool {
        !self.c()
    }

    #[must_use]
    pub fn cc(&self) -> bool {
        self.lo()
    }

    /// Negative. The mnemonic stands for "minus".
    #[must_use]
    pub fn mi(&self) -> bool {
        self.n()
    }

    /// Positive or zero. The mnemonic stands for "plus".
    #[must_use]
    pub fn pl(&self) -> bool {
        !self.n()
    }

    /// Signed overflow. The mnemonic stands for "V set".
    #[must_use]
    pub fn vs(&self) -> bool {
        self.v()
    }

    /// No signed overflow. The mnemonic stands for "V clear".
    #[must_use]
    pub fn vc(&self) -> bool {
        !self.v()
    }

    /// Unsigned higher.
    #[must_use]
    pub fn hi(&self) -> bool {
        self.c() && !self.z()
    }

    /// Unsigned lower or same.
    #[must_use]
    pub fn ls(&self) -> bool {
        !self.c() || self.z()
    }

    /// Signed greater than or equal.
    #[must_use]
    pub fn ge(&self) -> bool {
        self.n() == self.v()
    }

    /// Signed less than.
    #[must_use]
    pub fn lt(&self) -> bool {
        self.n() != self.v()
    }

    /// Signed greater than.
    #[must_use]
    pub fn gt(&self) -> bool {
        !self.z() && self.ge()
    }

    /// Signed less than or equal.
    #[must_use]
    pub fn le(&self) -> bool {
        self.z() || self.lt()
    }

    /// Always executed.
    #[must_use]
    pub fn al(&self) -> bool {
        true
    }

    #[must_use]
    pub fn nv(&self) -> bool {
        false
    }

    #[inline]
    #[must_use]
    pub fn pc(&self) -> u32 {
        self.register(registers::PC)
    }

    #[inline]
    #[must_use]
    pub fn lr(&self) -> u32 {
        self.register(registers::LR)
    }

    #[inline]
    #[must_use]
    pub fn sp(&self) -> u32 {
        self.register(registers::SP)
    }

    #[inline]
    #[must_use]
    pub fn register(&self, register: u8) -> u32 {
        self.registers[register as usize]
    }

    #[inline]
    pub fn set_pc(&mut self, value: u32) {
        self.set_register(registers::PC, value);
    }

    #[inline]
    pub fn set_lr(&mut self, value: u32) {
        self.set_register(registers::LR, value);
    }

    #[inline]
    pub fn set_sp(&mut self, value: u32) {
        self.set_register(registers::SP, value);
    }

    #[inline]
    pub fn set_register(&mut self, register: u8, value: u32) {
        self.registers[register as usize] = value;
    }
}
