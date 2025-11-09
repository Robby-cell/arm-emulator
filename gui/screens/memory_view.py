from PyQt6.QtWidgets import QWidget
from typing import Optional


class MemoryViewScreen(QWidget):
    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setupUI()

    def setupUI(self):
        self.setWindowTitle("Memory View")
