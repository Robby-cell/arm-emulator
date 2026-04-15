from arm_emulator_rs import Emulator  # type: ignore : import exists
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QFont, QCursor
from PyQt6.QtWidgets import (
    QGridLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QVBoxLayout,
    QWidget,
    QLineEdit,
    QPushButton,
)


class CpuPanel(QWidget):
    user_state_changed = pyqtSignal()

    def __init__(self, emulator: Emulator, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._emulator = emulator

        # State tracking for highlighting changes
        self._last_registers: list[int] = []
        self._last_flags: dict[str, bool] = {}

        # UI Components storage
        self._reg_inputs: list[QLineEdit] = []
        self._flag_buttons: dict[str, QPushButton] = {}

        self._layout = QVBoxLayout(self)
        self.setLayout(self._layout)

        self.setupUI()
        self.update_view()

    def setupUI(self) -> None:
        # Registers Group
        self._register_group = QGroupBox(self.tr("Registers"))
        reg_layout = QGridLayout()
        self._register_group.setLayout(reg_layout)
        self._register_group.setLayoutDirection(Qt.LayoutDirection.LeftToRight)

        # Monospace font for values
        mono_font = QFont("monospace", 10)

        # Create labels for R0-R15
        reg_names = [f"R{i}" for i in range(13)] + ["SP", "LR", "PC"]

        input_style = """
            QLineEdit { 
                background: transparent; 
                color: #dddddd; 
                border: 1px solid transparent; 
                padding: 1px;
            }
            QLineEdit:focus { 
                border: 1px solid #569CD6; 
                background: #1e1e1e; 
            }
        """

        for i, name in enumerate(reg_names):
            # Name Label (e.g., "R0:")
            name_lbl = QLabel(f"{name}:")
            name_lbl.setStyleSheet("color: #888888; font-weight: bold;")

            # Value Label (e.g., "0x00000000")
            val_input = QLineEdit("0x00000000")
            val_input.setFont(mono_font)
            val_input.setStyleSheet(input_style)

            # Connect the Enter key to our save slot
            val_input.editingFinished.connect(
                lambda idx=i, inp=val_input: self._on_register_edited(idx, inp)
            )

            self._reg_inputs.append(val_input)

            # Add to grid (2 columns of registers)
            row = i // 2
            col = (i % 2) * 2  # 0 or 2
            reg_layout.addWidget(name_lbl, row, col)
            reg_layout.addWidget(val_input, row, col + 1)

        # Flags Group (CPSR)
        self._flag_group = QGroupBox(self.tr("CPSR Flags"))
        self._flag_group.setLayoutDirection(Qt.LayoutDirection.LeftToRight)
        flag_layout = QHBoxLayout()
        self._flag_group.setLayout(flag_layout)

        for flag in ["N", "Z", "C", "V"]:
            btn = QPushButton(flag)
            btn.setFont(QFont("sans-serif", 12, QFont.Weight.Bold))
            btn.setCursor(
                QCursor(Qt.CursorShape.PointingHandCursor)
            )  # Show it's clickable

            btn.clicked.connect(lambda checked, f=flag: self._on_flag_toggled(f))

            self._flag_buttons[flag] = btn
            flag_layout.addWidget(btn)

        # Add groups to main layout
        self._layout.addWidget(self._register_group)
        self._layout.addWidget(self._flag_group)
        self._layout.addStretch()  # Push everything up

    def _on_register_edited(self, index: int, line_edit: QLineEdit) -> None:
        """Called when user presses Enter after editing a register."""
        text = line_edit.text().strip()
        try:
            # Parse hex or decimal
            value = int(text, 16) if text.lower().startswith("0x") else int(text, 0)
            if value < 0 or value > 0xFFFFFFFF:
                raise ValueError

            # Write to emulator
            if self._emulator.get_register(index) != value:
                self._emulator.set_register(index, value)
                print(f"User manually set R{index} to {hex(value)}")

                #  Update the "previous" state so it doesn't highlight red
                if self._last_registers:
                    self._last_registers[index] = value

                self.user_state_changed.emit()
            else:
                # User typed the same number, or just reformatted it (e.g. typed '10' over '0x0000000A')
                # Silently snap it back to standard formatting without triggering a state change
                line_edit.setText(f"0x{value:08X}")

            # Notify MainWindow to update Disassembly and Memory View
            self.user_state_changed.emit()

        except ValueError:
            # If input is invalid, immediately revert to the actual CPU state
            self.update_view()

    def _on_flag_toggled(self, flag: str) -> None:
        """Called when user clicks a flag button."""
        current_flags = self._emulator.flags
        new_val = not current_flags.get(flag, False)

        # Write to Rust emulator
        self._emulator.set_flag(flag, new_val)
        print(f"User manually toggled flag {flag} to {new_val}")

        if self._last_flags:
            self._last_flags[flag] = new_val

        # Notify MainWindow
        self.user_state_changed.emit()

    def update_view(self) -> None:
        """Fetches state from emulator and updates UI with highlighting."""

        current_regs = self._emulator.registers

        for i, val in enumerate(current_regs):
            inp = self._reg_inputs[i]

            # Prevent overwriting text if the user is currently typing in that box
            if not inp.hasFocus():
                inp.setText(f"0x{val:08X}")

                # Check for changes (Highlight Red)
                if self._last_registers and self._last_registers[i] != val:
                    inp.setStyleSheet("""
                        QLineEdit { color: #ff5555; font-weight: bold; background: transparent; border: 1px solid transparent; padding: 1px; }
                        QLineEdit:focus { border: 1px solid #569CD6; background: #1e1e1e; }
                    """)
                else:
                    inp.setStyleSheet("""
                        QLineEdit { color: #dddddd; background: transparent; border: 1px solid transparent; padding: 1px; }
                        QLineEdit:focus { border: 1px solid #569CD6; background: #1e1e1e; }
                    """)

        self._last_registers = list(current_regs)

        current_flags = self._emulator.flags

        for flag_name, is_set in current_flags.items():
            btn = self._flag_buttons.get(flag_name)
            if not btn:
                continue

            if is_set:
                base_style = "color: #2b2b2b; background-color: #ffff00; border: none; border-radius: 4px; padding: 2px;"
            else:
                base_style = "color: #676767; background-color: #2b2b2b; border: 1px solid #444; border-radius: 4px; padding: 2px;"

            if self._last_flags and self._last_flags.get(flag_name) != is_set:
                base_style += "border: 2px solid #ff5555;"

            btn.setStyleSheet("QPushButton { " + base_style + " }")

        self._last_flags = dict(current_flags)

    def retranslateUi(self) -> None:
        self._register_group.setTitle(self.tr("Registers"))
        self._flag_group.setTitle(self.tr("CPSR Flags"))
