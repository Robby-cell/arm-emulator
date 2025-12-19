from typing import Optional

from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import QHBoxLayout, QSplitter, QWidget

from ..widgets.code_editor import CodeEditor
from ..widgets.peripherals_panel import PeripheralsPanel

DEFAULT_ASM = r""".global _start
_start:
    mov r0, #23
    mov r1, #17

    @ Add r0 to r1, and store the result in r0. Set flags
    adds r0, r0, r1
    @ Branch, if not equal, i.e. Z flag is not set
    bne label0
label0:
    @ Setup system call to exit
    mov r7, #1
    @ 0 = no error
    mov r0, #0
    svc 0

"""


class EditorScreen(QWidget):
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
