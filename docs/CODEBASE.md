# ARM Emulator Codebase Documentation

## Table of Contents

- [Overview](#overview)
- [Project Structure](#project-structure)
- [Rust Emulator Core](#rust-emulator-core)
  - [Architecture](#architecture)
  - [CPU Module](#cpu-module)
  - [Memory Module](#memory-module)
  - [Instructions Module](#instructions-module)
  - [Execution Module](#execution-module)
  - [Peripherals Module](#peripherals-module)
- [Python GUI](#python-gui)
  - [Main Window](#main-window)
  - [Debugger Controller](#debugger-controller)
  - [Screens](#screens)
  - [Widgets](#widgets)
- [API Reference](#api-reference)
- [Building and Running](#building-and-running)

---

## Overview

This is an educational ARM emulator with a graphical user interface. The project consists of:

- **Rust Emulator Core**: A complete ARM instruction set emulator written in Rust
- **Python GUI**: A PyQt6-based desktop application for visualizing CPU operations

The emulator supports the ARM instruction set including:
- Data Processing (ADD, SUB, MOV, CMP, etc.)
- Memory Access (LDR, STR, LDM, STM)
- Branch instructions (B, BL, BX)
- Multiply instructions (MUL, MLA, UMULL, etc.)
- Supervisor Calls (SVC)

---

## Project Structure

```
arm-emulator/
├── crates/
│   └── emulator/           # Rust emulator core
│       └── src/
│           ├── lib.rs      # Main entry point
│           ├── cpu/        # CPU state and registers
│           ├── memory/     # Memory management and bus
│           ├── instructions/ # Instruction decoding
│           ├── execution/  # Instruction execution
│           ├── peripherals/ # GPIO and I/O
│           └── system/     # System utilities
├── gui/                    # Python GUI application
│   ├── main_window.py     # Main window
│   ├── controllers/       # Debugger controller
│   ├── screens/          # Application screens
│   └── widgets/          # UI widgets
├── docs/                  # Documentation
└── examples/              # Example ARM programs
```

---

## Rust Emulator Core

### Architecture

The emulator follows a fetch-decode-execute cycle:

```
┌─────────┐    ┌──────────┐    ┌───────────┐    ┌────────────┐
│  Fetch  │ -> │  Decode  │ -> │  Execute  │ -> │ Update PC  │
└─────────┘    └──────────┘    └───────────┘    └────────────┘
     │              │                │                  │
     v              v                v                  v
  read32()    try_into()      execute_with()   pc += 4
```

### CPU Module

**File**: `crates/emulator/src/cpu/mod.rs`

The CPU module provides the core processor state representation.

#### Key Types

```rust
// Main CPU state
pub struct Cpu {
    pub registers: [u32; 16],     // R0-R15
    pub cpsr: CpuFlags,           // Current Program Status Register
    pub state: ExecutionState,    // Current execution state
    // ... Other fields
}
```

#### Register Constants

```rust
pub mod registers {
    pub const R0: u8 = 0;
    pub const R1: u8 = 1;
    // ... R0-R15
    // And the aliases for SP, LR, PC
    pub const SP: u8 = 13;  // Stack Pointer
    pub const LR: u8 = 14;  // Link Register
    pub const PC: u8 = 15;  // Program Counter
}
```

#### Condition Flags

```rust
pub struct CpuFlags(u32);

// Flags are accessed via methods:
cpu.n()   // Negative flag (bit 31)
cpu.z()   // Zero flag (bit 30)
cpu.c()   // Carry flag (bit 29)
cpu.v()   // Overflow flag (bit 28)
```

#### Processor Modes

```rust
pub enum Mode {
    User = 0b10000,      // Unprivileged mode
    Fiq = 0b10001,       // Fast Interrupt Request
    Irq = 0b10010,       // Interrupt Request
    Supervisor = 0b10011, // Supervisor mode
    Abort = 0b10111,     // Abort mode
    Undefined = 0b11011, // Undefined mode
    System = 0b11111,    // Privileged User mode
}
```

#### Execution State

```rust
pub enum ExecutionState {
    Halted,              // Initial state
    Running,             // Actively executing
    Breakpoint(Breakpoint), // Debug breakpoint hit
    Exception(Exception), // Error occurred
    FinishedExecution(ExitStatus), // Program completed
    SupervisorCall(SupervisorCall), // SVC instruction
}
```

#### Key Methods

```rust
impl Cpu {
    pub fn new() -> Self;
    
    // Following methods take self as a parameter, but we will omit it for brevity
    // Register access
    pub fn pc() -> u32;           // Get program counter
    pub fn lr() -> u32;           // Get link register
    pub fn sp() -> u32;           // Get stack pointer
    pub fn register(u8) -> u32;   // Get any register
    pub fn set_pc(u32);
    pub fn set_sp(u32);
    pub fn set_lr(u32);
    pub fn set_register(u8, u32);
    
    // Flag manipulation
    pub fn set_n(bool); pub fn n() -> bool;
    pub fn set_z(bool); pub fn z() -> bool;
    pub fn set_c(bool); pub fn c() -> bool;
    pub fn set_v(bool); pub fn v() -> bool;
    
    // Mode control
    pub fn mode() -> Mode;
    pub fn set_mode(Mode);
    
    // Condition codes
    pub fn eq() -> bool;  // Equal (Z=1)
    pub fn ne() -> bool;  // Not Equal (Z=0)
    pub fn hs() -> bool;  // Higher or Same (C=1)
    pub fn lo() -> bool;  // Lower (C=0)
    pub fn ge() -> bool;  // Greater or Equal (N=V)
    pub fn lt() -> bool;  // Less Than (N!=V)
    pub fn gt() -> bool;  // Greater Than (!Z && N=V)
    pub fn le() -> bool;  // Less or Equal (Z || N!=V)
    
    // State control
    pub fn is_halted() -> bool;
    pub fn is_finished() -> bool;
    pub fn set_halted();
    pub fn set_running();
    pub fn reset();
}
```

---

### Memory Module

**File**: `crates/emulator/src/memory/mod.rs`

The memory module handles all memory operations including the memory bus and peripheral mapping.

#### Memory Map

| Region | Address Range | Size | Description |
|--------|---------------|------|-------------|
| Code | 0x00000000 - 0x1FFFFFFF | 512 MiB | Program code |
| SRAM | 0x20000000 - 0x3FFFFFFF | 512 MiB | Data/Stack/Heap |
| Peripherals | 0x40000000 - 0x5FFFFFFF | 512 MiB | Memory-mapped I/O |
| External | 0x60000000 - 0xFFFFFFFF | ~1 TiB (with  LPAE) | External devices |

#### Key Types

```rust
// Word type (32-bit on ARM)
pub type Word = u32;

// Error types
pub enum MemoryAccessError {
    InvalidReadPermission { addr: Word },
    InvalidWritePermission { addr: Word },
    UnalignedAccess,
    InvalidOffset { offset: Word },
    InvalidPeripheralRead { offset: Word },
    InvalidPeripheralWrite { offset: Word },
}

pub type MemoryAccessResult<T> = Result<T, MemoryAccessError>;

// Endianness
pub enum Endian {
    Little,
    Big,
}
```

#### Peripheral Trait

```rust
pub trait Peripheral {
    fn read32(&self, offset: u32) -> MemoryAccessResult<u32>;
    fn write32(&self, offset: u32, value: u32) -> MemoryAccessResult<()>;
    fn read_byte(&self, offset: u32) -> MemoryAccessResult<u8>;
    fn write_byte(&self, offset: u8, value: u8) -> MemoryAccessResult<()>;
    fn reset(&self);
}

pub struct MemoryMappedPeripheral {
    pub range: RangeInclusive<u32>,
    pub peripheral: Arc<dyn Peripheral + Send + Sync>,
}
```

#### Bus Structure

```rust
pub struct Bus {
    code: Vec<u8>,         // Code memory region
    sram: Vec<u8>,         // SRAM region (data/stack)
    peripherals: Vec<MemoryMappedPeripheral>, // Mapped peripherals
    external: Vec<u8>,     // External memory
}
```

#### Key Methods

```rust
impl Bus {
    pub fn new(code_size: Word, sram_size: Word, external_size: Word) -> Self;
    
    // Loading data
    pub fn load_code(&mut self, code: &[u8]);
    pub fn load_sram(&mut self, sram: &[u8]);
    pub fn load_external(&mut self, external: &[u8]);
    
    // Word access (endian-aware)
    pub fn read32_le(&self, addr: Word) -> MemoryAccessResult<u32>;
    pub fn read32_be(&self, addr: Word) -> MemoryAccessResult<u32>;
    pub fn write32_le(&mut self, addr: Word, value: u32) -> MemoryAccessResult<()>;
    pub fn write32_be(&mut self, addr: Word, value: u32) -> MemoryAccessResult<()>;
    
    // Byte access
    pub fn read_byte_le(&self, addr: Word) -> MemoryAccessResult<u8>;
    pub fn write_byte_le(&mut self, addr: Word, value: u8) -> MemoryAccessResult<()>;
    
    // Peripherals
    pub fn add_peripheral(&mut self, mapped_peripheral: MemoryMappedPeripheral);
    pub fn get_mapped_peripherals(&self) -> &[MemoryMappedPeripheral];
}
```

---

### Instructions Module

**File**: `crates/emulator/src/instructions/mod.rs`

The instructions module provides ARM instruction decoding.

#### Instruction Types

```rust
pub enum Instruction {
    DataProcessing(DataProcessingInstruction),  // ALU operations
    MemoryAccess(MemoryAccessInstruction),      // LDR/STR
    BlockDataTransfer(BlockDataTransferInstruction), // LDM/STM
    Branch(BranchInstruction),                  // B/BL
    BranchExchange(BranchExchangeInstruction), // BX/BLX
    SupervisorCall(SupervisorCallInstruction),  // SVC/SWI
    Multiply(MultiplyInstruction),              // MUL/MLA
    MultiplyLong(MultiplyLongInstruction),      // UMULL/SMULL
    Breakpoint(BreakpointInstruction),          // BKPT
}
```

#### Decoding

```rust
// Decode a raw 32-bit value into an Instruction
let raw: u32 = 0xE3A01001;  // MOV R1, #1
let instruction: Instruction = raw.try_into().unwrap();
```

#### Instruction Fields

**File**: `crates/emulator/src/instructions/fields.rs`

```rust
// Condition codes
pub enum Condition {
    EQ, // Equal
    NE, // Not Equal
    HS, // Higher or Same
    LO, // Lower
    MI, // Minus/Negative
    PL, // Plus/Positive
    VS, // Overflow
    VC, // No Overflow
    HI, // Higher
    LS, // Lower or Same
    GE, // Greater or Equal
    LT, // Less Than
    GT, // Greater Than
    LE, // Less or Equal
    AL, // Always
    NV, // Never
}

// Opcodes for Data Processing
pub enum Opcode {
    AND, // Logical AND
    EOR, // Logical XOR
    SUB, // Subtract
    RSB, // Reverse Subtract
    ADD, // Add
    ADC, // Add with Carry
    SBC, // Subtract with Carry
    RSC, // Reverse Subtract with Carry
    TST, // Test (set flags only)
    TEQ, // Test Equivalence (set flags only)
    CMP, // Compare (set flags only)
    CMN, // Compare Negative (set flags only)
    ORR, // Logical OR
    MOV, // Move
    BIC, // Bit Clear
    MVN, // Move Not
}

// Registers
pub enum Register {
    R0, R1, R2, R3, R4, R5, R6, R7,
    R8, R9, R10, R11, R12,
    SP, LR, PC,
}

// Shift types
pub enum ShiftType {
    LSL, // Logical Shift Left
    LSR, // Logical Shift Right
    ASR, // Arithmetic Shift Right
    ROR, // Rotate Right
}
```

---

### Execution Module

**File**: `crates/emulator/src/execution/mod.rs`

The execution module provides the instruction execution logic.

#### Execution Error

```rust
pub enum ExecutionError {
    Breakpoint(Breakpoint),
    MemoryAccessError(MemoryAccessError),
    InstructionConversionError(InstructionConversionError),
    Exception(Exception),
}
```

#### ExecutableInstruction Trait

```rust
pub trait ExecutableInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError>;
}
```

---

### Peripherals Module

**File**: `crates/emulator/src/peripherals/gpio.rs`

GPIO peripheral for LED simulation (STM32-style).

```rust
pub struct GpioPort {
    state: GpioState,
}

impl GpioPort {
    pub fn new() -> Self;
    pub fn is_led_on(&self) -> bool;
}

impl Peripheral for GpioPort {
    fn read32(&self, offset: u32) -> MemoryAccessResult<u32>;
    fn write32(&self, offset: u32, value: u32) -> MemoryAccessResult<()>;
    fn read_byte(&self, offset: u32) -> MemoryAccessResult<u8>;
    fn write_byte(&self, offset: u8, value: u8) -> MemoryAccessResult<()>;
    fn reset(&self);
}
```

---

## Python GUI

### Main Window

**File**: `gui/main_window.py`

The main application window providing:
- Menu bar with File, Edit, View, Run, Help menus
- Toolbar with run, step, stop, reset buttons
- Tabbed interface for different screens
- Status bar

### Debugger Controller

**File**: `gui/controllers/debugger_controller.py`

Manages emulator execution and UI synchronization.

```python
class DebuggerController(QObject):
    # Signals
    execution_started = pyqtSignal()
    execution_stopped = pyqtSignal()
    state_changed = pyqtSignal()
    breakpoint_hit = pyqtSignal(int)
    error_occurred = pyqtSignal(str)
    highlight_line = pyqtSignal(int)
    
    # Methods
    def load_program(self, program: AssembledOutput) -> None:
        """Load assembled program into emulator"""
    
    def run(self) -> None:
        """Start continuous execution"""
    
    def step(self) -> None:
        """Execute single instruction"""
    
    def stop(self) -> None:
        """Stop execution"""
    
    def reset_emulator(self) -> None:
        """Reset CPU to initial state"""
    
    def add_breakpoint_at_line(self, line: int) -> None:
        """Add breakpoint at source line"""
```

### Screens

| Screen | File | Description |
|--------|------|-------------|
| Editor | `gui/screens/editor.py` | Code editor with peripherals panel |
| Disassembly | `gui/screens/disassembly.py` | View disassembled instructions |
| Memory View | `gui/screens/memory_view.py` | Inspect memory contents |
| Tutorial | `gui/screens/tutorial_dialog.py` | Help/dialog windows |

### Widgets

| Widget | File | Description |
|--------|------|-------------|
| Code Editor | `gui/widgets/code_editor.py` | Syntax-highlighted ARM editor |
| CPU Panel | `gui/widgets/cpu_panel.py` | Register/flag display |
| Peripherals Panel | `gui/widgets/peripherals_panel.py` | GPIO/LED visualization |
| Tab | `gui/widgets/tab.py` | Custom tab widget |

---

## API Reference

### Creating an Emulator

```rust
use emulator::prelude::*;

let cpu = Cpu::new();
let bus = Bus::new(1024 * 1024, 1024 * 1024, 0);  // 1MB code, 1MB SRAM
let mut emulator = Emulator::new(cpu, bus, Endian::Little);
```

### Loading and Running

```rust
// Load ARM bytecode
emulator.load_program(&code, None, None);

// Step through instructions
while !emulator.is_done() {
    emulator.step()?;
}

// Check exit status
if let Some(status) = emulator.get_exit_status() {
    println!("Program exited with code: {}", status.exit_code);
}
```

### Breakpoints

```rust
// Add breakpoint at address
emulator.add_breakpoint_at(0x1000)?;

// Remove breakpoint
emulator.remove_breakpoint_at(0x1000)?;

// Step over breakpoint (when stopped at one)
emulator.step_over_breakpoint()?;
```

### Memory Access

```rust
// Read word (4 bytes)
let value: u32 = emulator.read32(0x1000)?;

// Write word
emulator.write32(0x1000, 0x12345678)?;

// Read byte
let byte: u8 = emulator.read_byte(0x1000)?;
```

---

## Building and Running

### Prerequisites

- Rust (cargo)
- Python 3.10+

### Building the Rust Emulator

Note: This is just a rust crate, there is no binary to build, no C bindings. These need to be created seperately.

```bash
# Debug build
cargo build --package emulator

# Release build
cargo build --package emulator --release
```

### Building Python Bindings

```bash
# Using maturin
maturin develop
# or
maturin develop --release
```

### Running the GUI

```bash
python gui_main.py
# Or
uv run gui_main.py
```

### Running Tests

```bash
cargo test --all
```
