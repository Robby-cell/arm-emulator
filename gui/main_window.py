from PyQt6.QtWidgets import QMainWindow, QWidget
from PyQt6.QtCore import Qt
from typing import Optional

from widgets import Tab


class MainWindow(QMainWindow):
    def __init__(self, parent: Optional[QWidget]=None, flags: Qt.WindowType=Qt.WindowType.Window):
        super().__init__(parent=parent, flags=flags)
        self.setupUI()

    def setupUI(self):
        self.setWindowTitle("ARM Simulator")
