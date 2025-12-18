from typing import Optional

from PyQt6.QtWidgets import QLabel, QPushButton, QWidget


class Tab(QPushButton):
    label: QLabel

    def __init__(self, text: str, parent: Optional[QWidget] = None) -> None:
        super().__init__(text=text, parent=parent)
        self.setupUI()

    def setupUI(self) -> None: ...
