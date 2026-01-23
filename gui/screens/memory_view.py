from typing import Optional

from arm_emulator_rs import Emulator, ExecutionError  # type: ignore
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
        emulator: Emulator,
        parent: Optional[QWidget] = None,
    ) -> None:
        super().__init__(parent)

        if emulator is None:
            raise ValueError("MemoryViewScreen requires a valid Emulator instance.")

        self.emulator = emulator
        self._current_address = 0

        # We need a flag to prevent infinite loops when the code updates the table programmatically
        self._is_updating = False

        self._layout = QVBoxLayout(self)
        self._controls_widget = QWidget()
        self._controls_layout = QHBoxLayout(self._controls_widget)
        self._address_input = QLineEdit()
        self._go_button = QPushButton(self.tr("Go"))
        self._table = QTableWidget()

        self.setupUI()
        self._init_connections()

        self.update_view()

    def setupUI(self) -> None:
        self.setLayout(self._layout)
        self._go_label = QLabel(self.tr("Go to Address:"))
        self._controls_layout.addWidget(self._go_label)
        self._controls_layout.addWidget(self._address_input)
        self._controls_layout.addWidget(self._go_button)

        self._address_input.setPlaceholderText("e.g., 0x20000000")
        hex_regex = QRegularExpression("^(0x)?[0-9a-fA-F]+$")
        self._address_input.setValidator(QRegularExpressionValidator(hex_regex))

        column_count = 1 + self.BYTES_PER_ROW + 1
        self._table.setColumnCount(column_count)
        self._table.setRowCount(self.ROWS_TO_DISPLAY)

        headers = (
            ["Address"] + [f"{i:02X}" for i in range(self.BYTES_PER_ROW)] + ["ASCII"]
        )
        self._table.setHorizontalHeaderLabels(headers)

        self._table.setFont(QFont("monospace", 10))

        # Allow clicking / typing to edit cells
        self._table.setEditTriggers(
            QTableWidget.EditTrigger.DoubleClicked
            | QTableWidget.EditTrigger.AnyKeyPressed
            | QTableWidget.EditTrigger.EditKeyPressed
        )

        self._table.setShowGrid(False)
        self._table.verticalHeader().setVisible(False)

        header_view = self._table.horizontalHeader()
        header_view.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        for i in range(1, self.BYTES_PER_ROW + 1):
            header_view.setSectionResizeMode(i, QHeaderView.ResizeMode.Fixed)
            self._table.setColumnWidth(i, 30)

        header_view.setSectionResizeMode(
            self.BYTES_PER_ROW + 1, QHeaderView.ResizeMode.ResizeToContents
        )

        self._layout.addWidget(self._controls_widget)
        self._layout.addWidget(self._table)

    def _init_connections(self) -> None:
        self._go_button.clicked.connect(self._on_go_to_address)
        self._address_input.returnPressed.connect(self._on_go_to_address)

        # Connect Cell Change
        self._table.cellChanged.connect(self._on_cell_changed)

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
                self.tr("Invalid Address"),
                self.tr("Please enter a valid decimal or hexadecimal address."),
            )
            return
        # Align to row boundary
        self._current_address = address - (address % self.BYTES_PER_ROW)
        self.update_view()

    def _on_cell_changed(self, row: int, column: int) -> None:
        """
        Called when a user finishes editing a cell.
        Parses the value and writes it to the emulator memory.
        """
        if self._is_updating:
            return

        # Ignore Address column (0) and ASCII column (last)
        if column == 0 or column == self.BYTES_PER_ROW + 1:
            return

        # Calculate Memory Address
        # We need to recalculate the address exactly as update_view does, including wrap-around
        row_offset = (self._current_address + (row * self.BYTES_PER_ROW)) & 0xFFFFFFFF
        byte_offset = column - 1
        target_address = (row_offset + byte_offset) & 0xFFFFFFFF

        # Get the item text
        item = self._table.item(row, column)
        if not item:
            return
        text = item.text().strip()

        # Parse Hex
        try:
            # Allow "FF", "0xFF", "10", etc.
            value = int(text, 16)
            if value < 0 or value > 255:
                raise ValueError("Byte out of range")
        except ValueError:
            # Invalid input: Revert the view to what memory actually has
            self.update_view()
            return

        # Write to Emulator
        try:
            self.emulator.write_byte(target_address, value)
            # Refresh view to normalize formatting (e.g. user typed "a", we want "0A")
            self.update_view()
        except Exception as e:
            # e.g. Writing to Read-Only memory or Unmapped regions
            QMessageBox.critical(
                self, self.tr("Write Error"), f"Failed to write to memory:\n{e}"
            )
            self.update_view()  # Revert to original value

    def update_view(self) -> None:
        """
        Populates the table by reading data directly from the Rust Emulator's bus.
        Handles wrapping at 0xFFFFFFFF.
        """
        self._is_updating = True  # Prevent _on_cell_changed from firing while we draw

        address_font = QFont("monospace", 10, QFont.Weight.Bold)
        address_color = QColor("#aaaaaa")
        invalid_color = QColor("#555555")
        valid_color = QColor("#dddddd")

        for row in range(self.ROWS_TO_DISPLAY):
            # Address Wrapping
            # Use bitwise AND to wrap around 32-bit boundary
            row_start_address = (
                self._current_address + (row * self.BYTES_PER_ROW)
            ) & 0xFFFFFFFF

            # Address Column (Read Only)
            addr_item = QTableWidgetItem(f"0x{row_start_address:08X}")
            addr_item.setFont(address_font)
            addr_item.setForeground(address_color)
            # Make Address Non-Editable
            addr_item.setFlags(addr_item.flags() ^ Qt.ItemFlag.ItemIsEditable)
            self._table.setItem(row, 0, addr_item)

            ascii_representation = ""

            # Byte Columns
            for i in range(self.BYTES_PER_ROW):
                # Wrap the individual byte address too (e.g. if row starts at 0xFFFFFFFF)
                address_to_read = (row_start_address + i) & 0xFFFFFFFF

                byte_str = "??"
                byte_color = invalid_color
                char_repr = "."
                is_valid_memory = False

                try:
                    if address_to_read < self.emulator.max_address():
                        byte_val = self.emulator.read_byte(address_to_read)

                        byte_str = f"{byte_val:02X}"
                        byte_color = valid_color
                        is_valid_memory = True

                        if 32 <= byte_val <= 126:
                            char_repr = chr(byte_val)
                        else:
                            char_repr = "."

                except (ValueError, ExecutionError):
                    pass

                byte_item = QTableWidgetItem(byte_str)
                byte_item.setTextAlignment(Qt.AlignmentFlag.AlignCenter)
                byte_item.setForeground(byte_color)

                # Set Read-Only if memory invalid
                if not is_valid_memory:
                    # If memory is unmapped (??), don't let them try to write to it
                    byte_item.setFlags(byte_item.flags() ^ Qt.ItemFlag.ItemIsEditable)

                self._table.setItem(row, i + 1, byte_item)

                ascii_representation += char_repr

            # ASCII Column (Read Only)
            ascii_item = QTableWidgetItem(ascii_representation)
            ascii_item.setForeground(valid_color)
            # Make ASCII Non-Editable
            ascii_item.setFlags(ascii_item.flags() ^ Qt.ItemFlag.ItemIsEditable)
            self._table.setItem(row, self.BYTES_PER_ROW + 1, ascii_item)

        self._is_updating = False  # Re-enable signal handling

    def retranslateUi(self) -> None:
        if hasattr(self, "_go_label"):
            self._go_label.setText(self.tr("Go to Address:"))

        self._go_button.setText(self.tr("Go"))

        headers = (
            ["Address"] + [f"{i:02X}" for i in range(self.BYTES_PER_ROW)] + ["ASCII"]
        )
        self._table.setHorizontalHeaderLabels(headers)
