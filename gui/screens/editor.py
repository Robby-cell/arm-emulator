"""
Editor Screen for the ARM Emulator GUI.

This module provides the main editing interface where users can write
ARM assembly code. It combines a code editor widget with a peripherals
panel for visualizing GPIO states.
"""

from typing import Optional

from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import QHBoxLayout, QSplitter, QWidget

from ..widgets.code_editor import CodeEditor
from ..widgets.peripherals_panel import PeripheralsPanel
from ..sample.starter_code import EXAMPLE_BLINK as DEFAULT_ASM


class EditorScreen(QWidget):
    """Main editing interface for writing ARM assembly code."""

    def __init__(self, parent: Optional[QWidget] = None) -> None:
        super().__init__(parent)

        self._layout = QHBoxLayout(self)
        self.setLayout(self._layout)
        self._layout.setContentsMargins(0, 0, 0, 0)

        self._splitter = QSplitter(Qt.Orientation.Horizontal)
        self._peripherals = PeripheralsPanel()
        self._editor = CodeEditor()

        self.setupUI()

    def setupUI(self) -> None:
        self._splitter.addWidget(self._peripherals)
        self._splitter.addWidget(self._editor)

        # This gives the peripherals panel 40% and the editor 70% of the space initially.
        self._splitter.setSizes([300, 700])
        self._splitter.setStretchFactor(1, 1)  # Allows the editor to expand more

        # Add the fully configured splitter to the main layout
        self._layout.addWidget(self._splitter)

        self._editor.setPlainText(DEFAULT_ASM)

    def get_code(self) -> str:
        """A method to allow the MainWindow to retrieve the code."""
        return self._editor.toPlainText()

    def retranslateUi(self) -> None:
        self._peripherals.retranslateUi()
