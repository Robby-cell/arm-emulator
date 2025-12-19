from typing import Optional

from arm_emulator_rs import Emulator, ExecutionError  # type: ignore : import exists
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

    def load_program(self, program: AssembledOutput) -> None:
        """Resets the emulator and loads the assembled program sections into memory."""

        self.reset_emulator()
        try:
            # Load the .text section into the 'code' memory region (starting at 0x0)
            if program.text is not None:
                print(f"Loading {len(program.text)} bytes into .text section (0x0)...")
                self._emulator.load_code(program.text)

            # Load the .sram section into the 'sram' memory region (starting at 0x20000000)
            if program.sram is not None:
                print(
                    f"Loading {len(program.sram)} bytes into .sram section (0x20000000)..."
                )
                self._emulator.load_sram(program.sram)

            # Load the .external section into the 'external' memory region (starting at 0x60000000)
            if program.external is not None:
                print(
                    f"Loading {len(program.external)} bytes into .external section (0x60000000)"
                )
                self._emulator.load_external(program.external)

            print("Load successful.")
            print(f"{self._emulator}")
        except Exception as e:
            self.error_occurred.emit(f"Failed to write program to emulator memory: {e}")
            return

        # Notify the UI that the memory state has changed and should be updated.
        self.state_changed.emit()

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

    def reset_emulator(self) -> None:
        """Resets the emulator to its initial state."""
        self.stop()
        self._emulator.reset()
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

    def _run_loop_step(self) -> None:
        """A single step within the continuous run loop."""
        if not self._is_running:
            return
        # The step method already contains the try/except block that will
        # automatically stop the timer on an error or breakpoint.
        self.step()
