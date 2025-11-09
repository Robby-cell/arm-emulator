from PyQt6.QtWidgets import QWidget, QPushButton, QLabel
from typing import Optional


class Tab(QPushButton):
    label: QLabel

    def __init__(self, text: str, parent: Optional[QWidget] = None):
        super().__init__(text=text, parent=parent)
        self.setupUI()

    def setupUI(self) -> None: ...
