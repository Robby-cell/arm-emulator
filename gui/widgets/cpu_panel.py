from typing import Dict, List, Optional

from arm_emulator_rs import Emulator  # type: ignore : import exists
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QFont
from PyQt6.QtWidgets import (
    QGridLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QVBoxLayout,
    QWidget,
)


class CpuPanel(QWidget):
    def __init__(self, emulator: Emulator, parent: Optional[QWidget] = None) -> None:
        super().__init__(parent)
        self._emulator = emulator

        # State tracking for highlighting changes
        self._last_registers: List[int] = []
        self._last_flags: Dict[str, bool] = {}

        # UI Components storage
        self._reg_labels: List[QLabel] = []
        self._flag_labels: Dict[str, QLabel] = {}

        self._layout = QVBoxLayout(self)
        self.setLayout(self._layout)

        self.setupUI()
        self.update_view()

    def setupUI(self) -> None:
        # 1. Registers Group
        self._register_group = QGroupBox(self.tr("Registers"))
        reg_layout = QGridLayout()
        self._register_group.setLayout(reg_layout)
        self._register_group.setLayoutDirection(Qt.LayoutDirection.LeftToRight)

        # Monospace font for values
        mono_font = QFont("monospace", 10)

        # Create labels for R0-R15
        reg_names = [f"R{i}" for i in range(13)] + ["SP", "LR", "PC"]

        for i, name in enumerate(reg_names):
            # Name Label (e.g., "R0:")
            name_lbl = QLabel(f"{name}:")
            name_lbl.setStyleSheet("color: #888888; font-weight: bold;")

            # Value Label (e.g., "0x00000000")
            val_lbl = QLabel("0x00000000")
            val_lbl.setFont(mono_font)

            self._reg_labels.append(val_lbl)

            # Add to grid (2 columns of registers)
            row = i // 2
            col = (i % 2) * 2  # 0 or 2
            reg_layout.addWidget(name_lbl, row, col)
            reg_layout.addWidget(val_lbl, row, col + 1)

        # 2. Flags Group (CPSR)
        self._flag_group = QGroupBox(self.tr("CPSR Flags"))
        self._flag_group.setLayoutDirection(Qt.LayoutDirection.LeftToRight)
        flag_layout = QHBoxLayout()
        self._flag_group.setLayout(flag_layout)

        for flag in ["N", "Z", "C", "V"]:
            lbl = QLabel(flag)
            lbl.setFont(QFont("sans-serif", 12, QFont.Weight.Bold))
            lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
            lbl.setStyleSheet(
                "color: #444444; background-color: #2b2b2b; border-radius: 4px; padding: 2px;"
            )

            self._flag_labels[flag] = lbl
            flag_layout.addWidget(lbl)

        # Add groups to main layout
        self._layout.addWidget(self._register_group)
        self._layout.addWidget(self._flag_group)
        self._layout.addStretch()  # Push everything up

    def update_view(self) -> None:
        """Fetches state from emulator and updates UI with highlighting."""

        # Update Registers
        current_regs = self._emulator.registers

        for i, val in enumerate(current_regs):
            label = self._reg_labels[i]
            text = f"0x{val:08X}"
            label.setText(text)

            # Check for changes
            if self._last_registers and self._last_registers[i] != val:
                # Highlight Color (Red/Orange)
                label.setStyleSheet("color: #ff5555; font-weight: bold;")
            else:
                # Standard Color
                label.setStyleSheet("color: #dddddd;")

        self._last_registers = list(current_regs)

        # Update Flags
        current_flags = self._emulator.flags

        for flag_name, is_set in current_flags.items():
            label = self._flag_labels.get(flag_name)
            if not label:
                continue

            # Determine style based on state
            if is_set:
                # Yellow background
                base_style = "color: #2b2b2b; background-color: #ffff00;"
            else:
                # Inactive styling (Dimmed)
                base_style = (
                    "color: #676767; background-color: #2b2b2b; border: 1px solid #444;"
                )

            # Check for changes to add a border or flash effect (Optional, but nice)
            # For flags, usually just showing active/inactive is enough,
            # but we can make the text bold if it JUST changed.
            if self._last_flags and self._last_flags.get(flag_name) != is_set:
                # Add a border to indicate it just flipped
                base_style += "border: 2px solid #ff5555;"

            label.setStyleSheet(base_style)

        self._last_flags = dict(current_flags)

    def retranslateUi(self) -> None:
        self._register_group.setTitle(self.tr("Registers"))
        self._flag_group.setTitle(self.tr("CPSR Flags"))
