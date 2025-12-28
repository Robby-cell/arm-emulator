from typing import Optional

from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QLabel,
    QPushButton,
    QWidget,
)


class TitleBar(QWidget):
    def __init__(self, title: str, parent: Optional[QWidget] = None) -> None:
        super().__init__(parent)
        self.initial_pos = None
        self.setFixedHeight(35)
        self.setStyleSheet("""
            QWidget { background-color: #333333; color: white; }
            QLabel { font-weight: bold; margin-left: 10px; }
        """)

        layout = QHBoxLayout()
        layout.setContentsMargins(0, 0, 0, 0)

        # Title Label
        self._title = QLabel(title)
        layout.addWidget(self._title)

        layout.addStretch()  # Push buttons to the right

        # Close Button
        self.btn_close = QPushButton("✕")
        self.btn_close.setFixedSize(35, 35)
        self.btn_close.setStyleSheet("""
            QPushButton { background-color: transparent; border: none; color: white; }
            QPushButton:hover { background-color: #d32f2f; }
        """)
        self.btn_close.clicked.connect(self.window().close)  # type: ignore : not None
        layout.addWidget(self.btn_close)

        self.setLayout(layout)

    def mousePressEvent(self, a0) -> None:
        if a0 is None:
            return
        if a0.button() == Qt.MouseButton.LeftButton:
            self.initial_pos = a0.globalPosition().toPoint()

    def mouseMoveEvent(self, a0) -> None:
        if a0 is None:
            return

        if self.initial_pos is not None:
            delta = a0.globalPosition().toPoint() - self.initial_pos
            self.window().move(self.window().pos() + delta)  # type: ignore : not None
            self.initial_pos = a0.globalPosition().toPoint()

    def mouseReleaseEvent(self, a0) -> None:
        self.initial_pos = None
