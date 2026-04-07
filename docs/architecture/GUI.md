# GUI Architecture

This document describes the high-level architecture of the ARM Emulator GUI.

For detailed API documentation, see [CODEBASE.md](../CODEBASE.md).

## Overview

The GUI is a PyQt6-based desktop application that provides:
- ARM assembly code editing with syntax highlighting
- Real-time CPU state visualization (registers, flags)
- Memory inspection
- GPIO/LED visualization
- Debugging controls (run, step, breakpoints)

## Component Structure

```
gui/
├── main_window.py          # Main application window
├── controllers/
│   └── debugger_controller.py  # Emulator execution control
├── screens/
│   ├── editor.py           # Code editor screen
│   ├── disassembly.py      # Instruction disassembly view
│   ├── memory_view.py      # Memory inspection
│   └── tutorial_dialog.py  # Help dialogs
└── widgets/
    ├── code_editor.py      # Syntax-highlighted editor
    ├── cpu_panel.py        # Register/flag display
    ├── peripherals_panel.py # GPIO visualization
    └── tab.py              # Tab widgets
```

## Key Components

### MainWindow

The main window provides:
- Menu bar (File, Edit, View, Run, Help)
- Toolbar with execution controls
- Tab-based screen navigation
- Status bar showing emulator state

### DebuggerController

The `DebuggerController` class acts as the bridge between the Rust emulator and the PyQt6 UI:

- **Signals**: `execution_started`, `execution_stopped`, `state_changed`, `breakpoint_hit`, `error_occurred`, `highlight_line`
- **Execution**: Uses QTimer for non-blocking execution loop
- **Source Mapping**: Maps source code lines to memory addresses for breakpoint management

## Build Dependencies

- PyQt6 (UI framework)
- keystone (ARM assembler)
- arm_emulator_rs (Rust bindings)
