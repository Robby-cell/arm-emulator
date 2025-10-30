from PyQt6.QtWidgets import QWidget, QLabel
from typing import Optional


class Tab(QWidget):
    label: QLabel

    def __init__(self, text: str, parent: Optional[QWidget]=None):
        super().__init__(parent=parent)
        self.label = QLabel(parent=self)
        self.setupUI(text=text)

    def setupUI(self, text: str) -> None:
        self.label.setText(text)

