"""
Memory View Screen for the ARM Emulator GUI.

This module provides a view for inspecting memory contents at runtime.
Users can view code, SRAM, and external memory regions in a tabular format
with addresses and values displayed in hexadecimal.
"""

from typing import Optional

from arm_emulator_rs import Emulator  # type: ignore
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
    QScrollBar,
)


class MemoryTableWidget(QTableWidget):
    """
    A custom table widget that intercepts scroll and key events
    to trigger infinite virtual scrolling.
    """

    def __init__(self, memory_screen: "MemoryViewScreen", *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.memory_screen = memory_screen
        self._wheel_accumulator = 0

    def wheelEvent(self, a0) -> None:
        if a0 is None:
            return

        delta = a0.angleDelta().y()
        # accumulate smooth scrolling
        self._wheel_accumulator += delta

        step_size = 120  # one notch
        rows_per_step = 3

        while abs(self._wheel_accumulator) >= step_size:
            if self._wheel_accumulator > 0:
                self.memory_screen.scroll_rows(-rows_per_step)
                self._wheel_accumulator -= step_size
            else:
                self.memory_screen.scroll_rows(rows_per_step)
                self._wheel_accumulator += step_size

        a0.accept()

    def keyPressEvent(self, e) -> None:
        if e is None:
            return

        row = self.currentRow()

        # Intercept arrows if we are at the top/bottom edge to trigger a scroll
        if e.key() == Qt.Key.Key_Down and row == self.memory_screen.ROWS_TO_DISPLAY - 1:
            self.memory_screen.scroll_rows(1)
            e.accept()
            return
        elif e.key() == Qt.Key.Key_Up and row == 0:
            self.memory_screen.scroll_rows(-1)
            e.accept()
            return
        # Handle Page Up / Page Down
        elif e.key() == Qt.Key.Key_PageDown:
            self.memory_screen.scroll_rows(self.memory_screen.ROWS_TO_DISPLAY)
            e.accept()
            return
        elif e.key() == Qt.Key.Key_PageUp:
            self.memory_screen.scroll_rows(-self.memory_screen.ROWS_TO_DISPLAY)
            e.accept()
            return

        super().keyPressEvent(e)


class MemoryViewScreen(QWidget):
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
        self._max_rows = (1 << 32) // self.BYTES_PER_ROW
        self._is_updating = False

        self._layout = QVBoxLayout(self)
        self._controls_widget = QWidget()
        self._controls_layout = QHBoxLayout(self._controls_widget)
        self._address_input = QLineEdit()
        self._go_button = QPushButton(self.tr("Go"))

        # Use our custom endless table
        self._table = MemoryTableWidget(self)

        # Add a custom global scrollbar
        self._scrollbar = QScrollBar(Qt.Orientation.Vertical)

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

        # Configure Table Columns
        column_count = 1 + self.BYTES_PER_ROW + 1
        self._table.setColumnCount(column_count)
        self._table.setRowCount(self.ROWS_TO_DISPLAY)  # Set rows ONCE
        self._table.setLayoutDirection(Qt.LayoutDirection.LeftToRight)

        headers = (
            ["Address"] + [f"{i:02X}" for i in range(self.BYTES_PER_ROW)] + ["ASCII"]
        )
        self._table.setHorizontalHeaderLabels(headers)
        self._table.setFont(QFont("monospace", 10))

        # Allow editing (for memory modification)
        self._table.setEditTriggers(
            QTableWidget.EditTrigger.DoubleClicked
            | QTableWidget.EditTrigger.AnyKeyPressed
            | QTableWidget.EditTrigger.EditKeyPressed
        )

        self._table.setShowGrid(False)
        self._table.verticalHeader().setVisible(False)  # type: ignore : not None

        # Disable native scrollbar so we can use ours
        self._table.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)

        header_view: QHeaderView = self._table.horizontalHeader()  # type: ignore : not None
        header_view.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        for i in range(1, self.BYTES_PER_ROW + 1):
            header_view.setSectionResizeMode(i, QHeaderView.ResizeMode.Fixed)
            self._table.setColumnWidth(i, 30)

        header_view.setSectionResizeMode(
            self.BYTES_PER_ROW + 1, QHeaderView.ResizeMode.ResizeToContents
        )

        # Configure custom scrollbar
        self._scrollbar.setRange(0, self._max_rows - 1)
        self._scrollbar.setPageStep(self.ROWS_TO_DISPLAY)

        # Put table and scrollbar side-by-side
        table_layout = QHBoxLayout()
        table_layout.setContentsMargins(0, 0, 0, 0)
        table_layout.setSpacing(0)
        table_layout.addWidget(self._table)
        table_layout.addWidget(self._scrollbar)

        self._layout.addWidget(self._controls_widget)
        self._layout.addLayout(table_layout)

    def _init_connections(self) -> None:
        self._go_button.clicked.connect(self._on_go_to_address)
        self._address_input.returnPressed.connect(self._on_go_to_address)
        self._table.cellChanged.connect(self._on_cell_changed)
        self._scrollbar.valueChanged.connect(self._on_scrollbar_moved)

    def scroll_rows(self, rows: int) -> None:
        """Called by the Table widget to virtual scroll up/down."""
        current = self._scrollbar.value()
        new_value = (current + rows) % self._max_rows
        self._scrollbar.setValue(new_value)

    def _on_scrollbar_moved(self, value: int) -> None:
        """Called when the user drags the custom scrollbar."""
        self._current_address = (value * self.BYTES_PER_ROW) & 0xFFFFFFFF
        self.update_view()

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
                self.tr("Please enter a valid address."),
            )
            return

        aligned = address - (address % self.BYTES_PER_ROW)
        self._scrollbar.setValue((aligned // self.BYTES_PER_ROW) % self._max_rows)

    def _on_cell_changed(self, row: int, column: int) -> None:
        """Handles user modifying memory inside the table."""
        if self._is_updating:
            return

        if column == 0 or column == self.BYTES_PER_ROW + 1:
            return

        # Recalculate address with wrap around
        row_offset = (self._current_address + (row * self.BYTES_PER_ROW)) & 0xFFFFFFFF
        byte_offset = column - 1
        target_address = (row_offset + byte_offset) & 0xFFFFFFFF

        item = self._table.item(row, column)
        if not item:
            return
        text = item.text().strip()

        try:
            value = int(text, 16)
            if value < 0 or value > 255:
                raise ValueError("Byte out of range")
        except ValueError:
            self.update_view()  # Invalid input, revert visually
            return

        try:
            self.emulator.write_byte(target_address, value)
            self.update_view()  # Refresh to normalize formatting
        except Exception as e:
            QMessageBox.critical(
                self, self.tr("Write Error"), f"Failed to write to memory:\n{e}"
            )
            self.update_view()

    def update_view(self) -> None:
        """Populates the table efficiently using a single bulk read."""
        self._is_updating = True

        value = self._current_address // self.BYTES_PER_ROW
        if self._scrollbar.value() != value:
            self._scrollbar.blockSignals(True)
            self._scrollbar.setValue(value)
            self._scrollbar.blockSignals(False)

        self._table.viewport().setUpdatesEnabled(False)  # type: ignore : not None

        try:
            # Styling constants
            address_font = QFont("monospace", 10, QFont.Weight.Bold)
            address_color = QColor("#aaaaaa")
            invalid_color = QColor("#555555")
            valid_color = QColor("#dddddd")

            # ONE SINGLE RUST CALL
            total_bytes = self.ROWS_TO_DISPLAY * self.BYTES_PER_ROW
            # Returns a flat list of 512 integers (-1 for invalid, 0-255 for valid)
            chunk_data = self.emulator.try_read_chunk(
                self._current_address, total_bytes
            )

            for row in range(self.ROWS_TO_DISPLAY):
                row_start_address = (
                    self._current_address + (row * self.BYTES_PER_ROW)
                ) & 0xFFFFFFFF

                # 1. Address Column
                addr_item = self._table.item(row, 0)
                if not addr_item:
                    addr_item = QTableWidgetItem()
                    # OPTIMIZATION 2: Set static properties ONLY ONCE upon creation
                    addr_item.setFont(address_font)
                    addr_item.setForeground(address_color)
                    addr_item.setFlags(
                        Qt.ItemFlag.ItemIsEnabled | Qt.ItemFlag.ItemIsSelectable
                    )
                    self._table.setItem(row, 0, addr_item)

                addr_item.setText(f"0x{row_start_address:08X}")

                # Build ASCII string efficiently
                ascii_chars = []

                # 2. Byte Columns
                for i in range(self.BYTES_PER_ROW):
                    col_index = i + 1

                    # Extract from our bulk-read flat list
                    data_index = (row * self.BYTES_PER_ROW) + i
                    val = chunk_data[data_index]

                    if val is not None:
                        byte_str = f"{val:02X}"
                        byte_color = valid_color
                        char_repr = chr(val) if 32 <= val <= 126 else "."
                        is_valid_memory = True
                    else:
                        byte_str = "??"
                        byte_color = invalid_color
                        char_repr = "."
                        is_valid_memory = False

                    # Reuse Item
                    byte_item = self._table.item(row, col_index)
                    if not byte_item:
                        byte_item = QTableWidgetItem()
                        # Static properties applied only once
                        byte_item.setTextAlignment(Qt.AlignmentFlag.AlignCenter)
                        self._table.setItem(row, col_index, byte_item)

                    # Update only what changes
                    if byte_item.text() != byte_str:
                        byte_item.setText(byte_str)

                    byte_item.setForeground(byte_color)

                    if is_valid_memory:
                        byte_item.setFlags(
                            Qt.ItemFlag.ItemIsEnabled
                            | Qt.ItemFlag.ItemIsSelectable
                            | Qt.ItemFlag.ItemIsEditable
                        )
                    else:
                        byte_item.setFlags(
                            Qt.ItemFlag.ItemIsEnabled | Qt.ItemFlag.ItemIsSelectable
                        )

                    ascii_chars.append(char_repr)

                # 3. ASCII Column
                ascii_col_index = self.BYTES_PER_ROW + 1
                ascii_item = self._table.item(row, ascii_col_index)
                if not ascii_item:
                    ascii_item = QTableWidgetItem()
                    # Static properties applied only once
                    ascii_item.setForeground(valid_color)
                    ascii_item.setFlags(
                        Qt.ItemFlag.ItemIsEnabled | Qt.ItemFlag.ItemIsSelectable
                    )
                    self._table.setItem(row, ascii_col_index, ascii_item)

                ascii_item.setText("".join(ascii_chars))

        finally:
            self._table.viewport().setUpdatesEnabled(True)  # type: ignore : not None
            self._is_updating = False

    def retranslateUi(self) -> None:
        if hasattr(self, "_go_label"):
            self._go_label.setText(self.tr("Go to Address:"))
        self._go_button.setText(self.tr("Go"))
        headers = (
            ["Address"] + [f"{i:02X}" for i in range(self.BYTES_PER_ROW)] + ["ASCII"]
        )
        self._table.setHorizontalHeaderLabels(headers)
