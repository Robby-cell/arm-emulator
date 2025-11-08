from PyQt6.QtWidgets import QWidget, QVBoxLayout
from typing import Optional
from widgets.code_editor import CodeEditor

class EditorScreen(QWidget):
    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        
        self._layout = QVBoxLayout(self)
        self._layout.setContentsMargins(0, 0, 0, 0) # Use the full space
        self.setLayout(self._layout)

        self._editor = CodeEditor()
        self.setupUI()

    def setupUI(self):
        self._layout.addWidget(self._editor)
        self._editor.setPlainText("# Write your ARM assembly code here")

    def get_code(self) -> str:
        """A method to allow the MainWindow to retrieve the code."""
        return self._editor.toPlainText()
