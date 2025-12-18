from typing import Optional

from PyQt6.QtWidgets import QWidget


class DisassemblyScreen(QWidget):
    def __init__(self, parent: Optional[QWidget] = None) -> None:
        super().__init__(parent)
        self.setupUI()

    def setupUI(self) -> None:
        self.setWindowTitle("Disassembly")
