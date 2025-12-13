from typing import Optional

from arm_emulator_rs import emulator  # type: ignore
from PyQt6.QtCore import QRegularExpression, Qt
from PyQt6.QtGui import QColor, QFont, QRegularExpressionValidator
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QMessageBox,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget,
)


class MemoryViewScreen(QWidget):
    # Constants for the view layout
    BYTES_PER_ROW = 16
    ROWS_TO_DISPLAY = 32

    def __init__(
        self,
        emulator: emulator.Emulator,
        parent: Optional[QWidget] = None,
    ) -> None:
        super().__init__(parent)

        if emulator is None:
            raise ValueError("MemoryViewScreen requires a valid Emulator instance.")

        # Data Model is the Rust Emulator's Bus
        self.emulator = emulator
        self._current_address = 0

        # UI Widgets
        self._layout = QVBoxLayout(self)
        self._controls_widget = QWidget()
        self._controls_layout = QHBoxLayout(self._controls_widget)
        self._address_input = QLineEdit()
        self._go_button = QPushButton("Go")
        self._table = QTableWidget()

        self._setup_ui()
        self._init_connections()

        # Initial population of the view
        self.update_view()

    def _setup_ui(self) -> None:
        self.setLayout(self._layout)
        self._controls_layout.addWidget(QLabel("Go to Address:"))
        self._controls_layout.addWidget(self._address_input)
        self._controls_layout.addWidget(self._go_button)
        self._address_input.setPlaceholderText("e.g., 0x20000000")
        hex_regex = QRegularExpression("^(0x)?[0-9a-fA-F]+$")
        self._address_input.setValidator(QRegularExpressionValidator(hex_regex))
        column_count = 1 + self.BYTES_PER_ROW + 1
        self._table.setColumnCount(column_count)
        headers = (
            ["Address"] + [f"{i:02X}" for i in range(self.BYTES_PER_ROW)] + ["ASCII"]
        )
        self._table.setHorizontalHeaderLabels(headers)
        self._table.setFont(QFont("monospace", 10))
        self._table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self._table.setShowGrid(False)
        self._table.verticalHeader().setVisible(False)  # type: ignore : not None
        header_view = self._table.horizontalHeader()
        header_view.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)  # type: ignore : not None
        for i in range(1, self.BYTES_PER_ROW + 1):
            header_view.setSectionResizeMode(i, QHeaderView.ResizeMode.Fixed)  # type: ignore : not None
            self._table.setColumnWidth(i, 30)
        header_view.setSectionResizeMode(  # type: ignore : not None
            self.BYTES_PER_ROW + 1, QHeaderView.ResizeMode.ResizeToContents
        )
        self._layout.addWidget(self._controls_widget)
        self._layout.addWidget(self._table)

    def _init_connections(self) -> None:
        self._go_button.clicked.connect(self._on_go_to_address)
        self._address_input.returnPressed.connect(self._on_go_to_address)

    def _parse_address(self, addr_str: str) -> Optional[int]:
        addr_str = addr_str.strip().lower()
        if not addr_str:
            return None
        try:
            return int(addr_str, 0)
        except ValueError:
            return None

    def _on_go_to_address(self) -> None:
        address = self._parse_address(self._address_input.text())
        if address is None:
            QMessageBox.warning(
                self,
                "Invalid Address",
                "Please enter a valid decimal or hexadecimal address.",
            )
            return
        self._current_address = address - (address % self.BYTES_PER_ROW)
        self.update_view()

    def update_view(self) -> None:
        """
        Populates the table by reading data directly from the Rust Emulator's bus.
        """
        # Can't call this yet. Currently broken. Working on it later
        # self._do_update_view()
        pass

    def _do_update_view(self):
        self._table.setRowCount(0)
        self._table.setRowCount(self.ROWS_TO_DISPLAY)

        address_font = QFont("monospace", 10, QFont.Weight.Bold)
        address_color = QColor("#aaaaaa")

        for row in range(self.ROWS_TO_DISPLAY):
            row_start_address = self._current_address + (row * self.BYTES_PER_ROW)

            if row_start_address >= self.emulator.max_address():
                self._table.setRowCount(row)
                break

            # Address Column
            addr_item = QTableWidgetItem(f"0x{row_start_address:08X}")
            addr_item.setFont(address_font)
            addr_item.setForeground(address_color)
            self._table.setItem(row, 0, addr_item)

            # --- Read byte-by-byte from the emulator ---
            ascii_representation = ""
            for i in range(self.BYTES_PER_ROW):
                address_to_read = row_start_address + i

                # Check bounds before reading
                if address_to_read < self.emulator.max_address():
                    byte_val = self.emulator.read_byte(address_to_read)
                    byte_str = f"{byte_val:02X}"
                    ascii_representation += (
                        chr(byte_val) if 32 <= byte_val <= 126 else "."
                    )
                else:
                    byte_str = ".."  # Visual indicator for out of bounds
                    ascii_representation += "."

                byte_item = QTableWidgetItem(byte_str)
                byte_item.setTextAlignment(Qt.AlignmentFlag.AlignCenter)
                self._table.setItem(row, i + 1, byte_item)

            # Set the final ASCII item
            ascii_item = QTableWidgetItem(ascii_representation)
            self._table.setItem(row, self.BYTES_PER_ROW + 1, ascii_item)
