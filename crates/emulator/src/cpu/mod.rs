use std::ops::{Index, IndexMut};

use thiserror::Error;

use crate::{Breakpoint, memory::MemoryAccessError};

mod display;
#[cfg(test)]
mod tests;

impl Cpu {
    pub const DEFAULT_CPSR: CpuFlags = CpuFlags(0x000000D3);

    pub const N_FLAG: CpuFlags = CpuFlags(1 << 31);
    pub const Z_FLAG: CpuFlags = CpuFlags(1 << 30);
    pub const C_FLAG: CpuFlags = CpuFlags(1 << 29);
    pub const V_FLAG: CpuFlags = CpuFlags(1 << 28);

    pub const I_FLAG: CpuFlags = CpuFlags(1 << 7);
    pub const F_FLAG: CpuFlags = CpuFlags(1 << 6);
    pub const T_FLAG: CpuFlags = CpuFlags(1 << 5);
    pub const MODE_MASK: CpuFlags = CpuFlags(0x1F); // Mask for the bottom 5 bits
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[must_use]
pub enum Mode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

impl From<Mode> for CpuFlags {
    fn from(value: Mode) -> Self {
        Self(value as u32)
    }
}

/// The program is ready to exit, with the given `exit_code`.
#[derive(Debug, Clone)]
#[repr(transparent)]
#[must_use]
pub struct ExitStatus {
    /// The exit code of the program.
    /// i.e. the equival of `return 0`
    pub exit_code: i32,
}

/// An interupt is currently active. The processor must handle it first.
#[derive(Debug, Clone)]
#[repr(transparent)]
#[must_use]
pub struct SupervisorCall {
    /// The code of the active supervisor call.
    pub code: u32,
}

/// An exception. Includes traps/breakpoints. Does not include [SupervisorCall].
#[derive(Debug, Error, Clone)]
pub enum Exception {
    #[error("memory access violation: {0}")]
    MemoryAccess(#[from] MemoryAccessError),

    #[error("cpu error: {0}")]
    CpuError(#[from] CpuError),
}

/// Execution state of the CPU in the current program. Keeps track of what is happening within the CPU.
/// Has a breakpoint been hit? Is it still running? Should it be exiting? Is it handling an interupt?
#[derive(Debug, Default, Clone, derive_more::From)]
#[must_use]
pub enum ExecutionState {
    /// The program is currently executing
    #[default]
    Halted,

    Running,

    /// This is an active [Breakpoint].
    Breakpoint(Breakpoint),

    /// There is an active [Exception].
    /// Does not include supervisor calls though, even though
    /// it probably should be considered as such.
    Exception(Exception),

    /// Finished executing, and returned the exit code.
    /// Wraps [ExitStatus], which captures the exit state of the CPU, when finished
    /// executing a program.
    FinishedExecution(ExitStatus),

    /// Interupt handler. Software interupts/[SupervisorCall]
    SupervisorCall(SupervisorCall),
}

/// Error retrieving a [Mode] from bits.
/// This should never happen in practice, the [CPU](Cpu) struct ensures
/// this is interacted with correctly.
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum ModeError {
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

#[derive(Debug, Error, Clone)]
pub enum CpuError {
    #[error("unprivileged access")]
    UnprivilegedAccess,

    #[error("segmentation fault ({addr:#X})")]
    Segfault { addr: u32 },

    #[error("unaligned memory access")]
    UnalignedAccess,
}

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
/// Does not include memory, instructions, actual execution logic, etc.
/// This is simply a low-level representation of the ARM CPU state.
/// Other crates will build on top of this to provide a full CPU emulator.
#[derive(Clone)]
#[must_use]
pub struct Cpu {
    pub registers: [u32; 16],

    /// Current Program Status Register, as
    /// [specified][https://documentation-service.arm.com/static/5f8db1f7f86e16515cdba175] in the ARM architecture
    pub cpsr: CpuFlags,

    // SPSR (Saved Program Status Register) for each privileged mode
    pub spsr_svc: CpuFlags,
    pub spsr_abt: CpuFlags,
    pub spsr_und: CpuFlags,
    pub spsr_irq: CpuFlags,
    pub spsr_fiq: CpuFlags,

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
        (self.cpsr & flag) != CpuFlags(0)
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
