from PyQt6.QtWidgets import QWidget, QHBoxLayout
from typing import Optional
from widgets.code_editor import CodeEditor

class EditorScreen(QWidget):
    def __init__(self, parent: Optional[QWidget]=None):
        super().__init__(parent)
        self._layout = QHBoxLayout(self)
        self.setLayout(self._layout)

        self._editor = CodeEditor()
        self.setupUI()

    def setupUI(self):
        self.setWindowTitle("Editor")
        self.setStyleSheet("background-color: #444444;")

        self._layout.addWidget(self._editor)
        self._editor.setPlainText("# Write your code here")
        self._editor.show()
