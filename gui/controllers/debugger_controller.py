from typing import Dict, Optional

from arm_emulator_rs import (
    Emulator,  # type: ignore : import exists
    ExecutionError,  # type: ignore : import exists
    RangeInclusiveU32,  # type: ignore : import exists
)
from PyQt6.QtCore import QObject, QTimer, pyqtSignal

from assembler import AssembledOutput


class DebuggerController(QObject):
    """
    Manages the state and execution of the Rust emulator.
    Communicates with the UI via signals.
    """

    execution_started = pyqtSignal()
    execution_stopped = pyqtSignal()
    state_changed = pyqtSignal()
    breakpoint_hit = pyqtSignal(int)  # Emits the address of the breakpoint
    error_occurred = pyqtSignal(str)  # Emits the error message
    _peripherals = []
    highlight_line = pyqtSignal(int)

    def __init__(self, emulator: Emulator, parent: Optional[QObject] = None) -> None:
        super().__init__(parent)

        if emulator is None:
            raise ValueError("DebuggerController requires a valid Emulator instance.")

        self._emulator = emulator
        self._is_running = False
        self._is_at_breakpoint = False
        self._breakpoint_addr = None

        # Use a QTimer for the non-blocking run loop
        self._run_timer = QTimer(self)
        self._run_timer.timeout.connect(self._run_loop_step)

        self._source_map: Dict[int, int] = {}  # Line -> Addr
        self._reverse_map: Dict[int, int] = {}  # Addr -> Line

    def load_program(self, program: AssembledOutput) -> None:
        """Resets the emulator and loads the assembled program sections into memory."""

        self.reset_emulator()
        self._source_map = program.source_map
        self._reverse_map = program.reverse_map

        try:
            self._emulator.load_program(program.text, program.sram, program.external)

            print("Load successful.")
            print(f"{self._emulator}")
        except Exception as e:
            self.error_occurred.emit(f"Failed to write program to emulator memory: {e}")
            return

        # Notify the UI that the memory state has changed and should be updated.
        self.state_changed.emit()
        self._update_highlight()

    def run(self) -> None:
        """Starts continuous execution of the emulator."""
        if self._is_running:
            return

        self._is_running = True
        self.execution_started.emit()
        self._run_timer.start(0)  # 0ms interval runs as fast as the event loop allows

    def stop(self) -> None:
        """Stops continuous execution."""
        if not self._is_running:
            return

        self._run_timer.stop()
        self._is_running = False
        self.execution_stopped.emit()

    def step(self) -> None:
        """Executes a single step, handling breakpoints correctly."""
        try:
            if self._is_at_breakpoint:
                # If we were paused at a breakpoint, we must "step over" it.
                # This un-patches, executes one, and re-patches atomically.
                self._emulator.step_over_breakpoint()
                self._is_at_breakpoint = False
            else:
                # Otherwise, just do a normal step.
                self._emulator.step()

            # After a successful step, tell the UI to update everything.
            self.state_changed.emit()
            self._update_highlight()
        except ExecutionError as e:
            self.stop()  # Always stop the run loop on any error

            # Check if the error is a breakpoint signal from Rust
            if e.is_breakpoint():
                # Extract address from error if possible
                # self._breakpoint_addr = self._emulator.cpu.pc()
                self._is_at_breakpoint = True
                self.breakpoint_hit.emit(self._breakpoint_addr)
            else:
                # It's a different error (e.g., memory access violation)
                self.error_occurred.emit(str(e))

            # In either error case, the state has changed.
            self.state_changed.emit()
            self._update_highlight()

    def _update_highlight(self):
        # Get current PC from Rust emulator
        # You need to expose pc via a getter or access registers[15]
        # Assuming registers getter returns list: [R0...R15]
        pc = self._emulator.registers[15]

        if pc in self._reverse_map:
            line_num = self._reverse_map[pc]
            self.highlight_line.emit(line_num)
        else:
            # PC is in unknown territory (e.g. OS code or unmapped), clear highlight
            self.highlight_line.emit(-1)

    def reset_emulator(self) -> None:
        """Resets the emulator to its initial state."""
        self.stop()
        self._emulator.reset()

        self.configure_peripherals(self._peripherals)

        self._is_at_breakpoint = False
        self.state_changed.emit()

    def toggle_breakpoint(self, address: int, is_set: bool) -> None:
        """Adds or removes a breakpoint in the emulator."""
        try:
            if is_set:
                self._emulator.add_breakpoint_at(address)
            else:
                self._emulator.restore_instruction_at(address)
        except Exception as e:
            self.error_occurred.emit(
                f"Failed to toggle breakpoint at {hex(address)}: {e}"
            )

    def on_breakpoint_toggled(self, line_number: int, is_set: bool):
        if line_number in self._source_map:
            addr = self._source_map[line_number]
            try:
                if is_set:
                    print(f"Setting breakpoint at line {line_number} (0x{addr:X})")
                    self._emulator.add_breakpoint_at(addr)
                else:
                    print(f"Removing breakpoint at line {line_number} (0x{addr:X})")
                    self._emulator.restore_instruction_at(addr)
            except Exception as e:
                self.error_occurred.emit(f"Breakpoint Error: {e}")
        else:
            print(f"Warning: Line {line_number} does not map to an instruction.")

    def _run_loop_step(self) -> None:
        """A single step within the continuous run loop."""
        if not self._is_running:
            return
        # The step method already contains the try/except block that will
        # automatically stop the timer on an error or breakpoint.
        self.step()

    def set_peripherals(self, peripherals_list) -> None:
        self._peripherals = peripherals_list

    def configure_peripherals(self, peripherals_list) -> None:
        for addr, addr_end, instance in peripherals_list:
            self._emulator.add_peripheral(RangeInclusiveU32(addr, addr_end), instance)
