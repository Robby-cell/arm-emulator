from PyQt6.QtWidgets import QWidget, QHBoxLayout, QSplitter
from PyQt6.QtCore import Qt

from widgets.code_editor import CodeEditor
from widgets.peripherals_panel import PeripheralsPanel

from typing import Optional


class EditorScreen(QWidget):
    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)

        self._layout = QHBoxLayout(self)
        self.setLayout(self._layout)
        self._layout.setContentsMargins(0, 0, 0, 0)

        self._splitter = QSplitter(Qt.Orientation.Horizontal)
        self._peripherals = PeripheralsPanel()
        self._editor = CodeEditor()

        self.setupUI()

    def setupUI(self):
        self._splitter.addWidget(self._peripherals)
        self._splitter.addWidget(self._editor)

        # This gives the peripherals panel 40% and the editor 70% of the space initially.
        self._splitter.setSizes([300, 700])
        self._splitter.setStretchFactor(1, 1)  # Allows the editor to expand more

        # Add the fully configured splitter to the main layout
        self._layout.addWidget(self._splitter)

        self._editor.setPlainText("# Write your code here")

    def get_code(self) -> str:
        """A method to allow the MainWindow to retrieve the code."""
        return self._editor.toPlainText()
