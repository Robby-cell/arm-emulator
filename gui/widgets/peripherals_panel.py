from PyQt6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QComboBox,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QLineEdit,
    QFormLayout,
    QHeaderView,
    QMessageBox,
)
from PyQt6.QtGui import QRegularExpressionValidator
from PyQt6.QtCore import QRegularExpression
from typing import Optional, Dict, Type, List, Tuple

from arm_emulator_rs import memory  # type: ignore


# --- The Peripheral "Factory" section ---
class LEDBank:
    pass


class SevenSegmentDisplay:
    pass


class Timer:
    pass


class PushButtons:
    pass


PERIPHERAL_REGISTRY: Dict[str, Type] = {
    "LED Bank": LEDBank,
    "7-Segment Display": SevenSegmentDisplay,
    "Timer": Timer,
    "Push Buttons": PushButtons,
}

VALID_MEMORY_BEGIN: int = int(memory.MemoryRegion.PERIPHERAL_BEGIN)
VALID_MEMORY_END: int = int(memory.MemoryRegion.PERIPHERAL_END)


class PeripheralsPanel(QWidget):
    """
    A widget panel for creating and configuring simulated peripherals
    with memory validation.
    """

    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)

        # --- Data model to track configured memory ranges ---
        self._configured_ranges: List[Tuple[int, int]] = []

        # --- Widget initialization ---
        self._layout = QVBoxLayout(self)
        self.setLayout(self._layout)
        self._form_widget = QWidget()
        self._form_layout = QFormLayout(self._form_widget)
        self._type_combo = QComboBox()
        self._name_input = QLineEdit()
        self._begin_addr_input = QLineEdit()
        self._end_addr_input = QLineEdit()
        self._add_button = QPushButton("Add Peripheral")
        self._delete_button = QPushButton("Delete Selected")
        self._peripheral_table = QTableWidget()

        self.setupUI()

        # --- Connections ---
        self._add_button.clicked.connect(self._on_add_peripheral)
        self._delete_button.clicked.connect(self._on_delete_peripheral)
        self._peripheral_table.itemSelectionChanged.connect(
            self._update_delete_button_state
        )

    def setupUI(self):
        self._form_layout.addRow("Type:", self._type_combo)
        self._form_layout.addRow("Instance Name:", self._name_input)
        self._form_layout.addRow("Begin Address:", self._begin_addr_input)
        self._form_layout.addRow("End Address:", self._end_addr_input)
        self._type_combo.addItems(PERIPHERAL_REGISTRY.keys())

        addr_tooltip_text = f"Valid address between {hex(VALID_MEMORY_BEGIN)} ({VALID_MEMORY_BEGIN}) and {hex(VALID_MEMORY_END)} ({VALID_MEMORY_END})"
        hex_regex = QRegularExpression("^(0x)?[0-9a-fA-F]+$")
        hex_validator = QRegularExpressionValidator(hex_regex)
        self._begin_addr_input.setValidator(hex_validator)
        self._end_addr_input.setValidator(hex_validator)
        self._begin_addr_input.setPlaceholderText("Hex or Decimal")
        self._end_addr_input.setPlaceholderText("Hex or Decimal")

        self._begin_addr_input.setToolTip(addr_tooltip_text)
        self._end_addr_input.setToolTip(addr_tooltip_text)

        self._peripheral_table.setColumnCount(3)
        self._peripheral_table.setHorizontalHeaderLabels(
            ["Type", "Name", "Memory Range"]
        )
        header = self._peripheral_table.horizontalHeader()
        header.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)  # type: ignore
        header.setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)  # type: ignore
        header.setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)  # type: ignore
        self._peripheral_table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self._peripheral_table.setSelectionBehavior(
            QTableWidget.SelectionBehavior.SelectRows
        )
        self._peripheral_table.setSelectionMode(
            QTableWidget.SelectionMode.SingleSelection
        )
        self._layout.addWidget(self._form_widget)
        self._layout.addWidget(self._add_button)
        self._layout.addWidget(self._delete_button)
        self._layout.addWidget(self._peripheral_table, stretch=1)
        self._delete_button.setEnabled(False)

    def _parse_address(self, addr_str: str) -> Optional[int]:
        addr_str = addr_str.strip().lower()
        if not addr_str:
            return None
        try:
            if addr_str.startswith("0x"):
                return int(addr_str, 16)
            if any(c in "abcdef" for c in addr_str):
                return int(addr_str, 16)
            return int(addr_str, 10)
        except ValueError:
            return None

    def _on_add_peripheral(self):
        """Validates input against global and existing ranges, then adds."""
        p_type = self._type_combo.currentText()
        p_name = self._name_input.text().strip()
        start_addr = self._parse_address(self._begin_addr_input.text())
        end_addr = self._parse_address(self._end_addr_input.text())

        # --- Basic Validation ---
        if not p_name or start_addr is None or end_addr is None:
            QMessageBox.warning(
                self, "Input Error", "All fields must be filled with valid values."
            )
            return
        if start_addr > end_addr:
            QMessageBox.warning(
                self,
                "Input Error",
                "Start address must not be greater than end address.",
            )
            return

        # --- Enforce Global Memory Range ---
        if not (
            VALID_MEMORY_BEGIN <= start_addr <= VALID_MEMORY_END
            and VALID_MEMORY_BEGIN <= end_addr <= VALID_MEMORY_END
        ):
            msg = (
                f"Memory addresses must be within the valid range:\n"
                f"{hex(VALID_MEMORY_BEGIN)} - {hex(VALID_MEMORY_END)}"
            )
            QMessageBox.warning(self, "Address Out of Range", msg)
            return

        for exist_start, exist_end in self._configured_ranges:
            # Classic interval overlap check
            if start_addr <= exist_end and end_addr >= exist_start:
                msg = (
                    f"The proposed memory range ({hex(start_addr)} - {hex(end_addr)}) "
                    f"overlaps with an existing peripheral's range "
                    f"({hex(exist_start)} - {hex(exist_end)})."
                )
                QMessageBox.warning(self, "Memory Overlap", msg)
                return

        # --- All checks passed, add to table AND data model ---
        row_count = self._peripheral_table.rowCount()
        self._peripheral_table.insertRow(row_count)

        memory_range_str = f"{hex(start_addr)} - {hex(end_addr)}"

        type_item = QTableWidgetItem(p_type)
        type_item.setToolTip(p_type)
        name_item = QTableWidgetItem(p_name)
        name_item.setToolTip(p_name)
        range_item = QTableWidgetItem(memory_range_str)
        range_item.setToolTip(memory_range_str)

        self._peripheral_table.setItem(row_count, 0, type_item)
        self._peripheral_table.setItem(row_count, 1, name_item)
        self._peripheral_table.setItem(row_count, 2, range_item)

        self._configured_ranges.append((start_addr, end_addr))

        self._name_input.clear()
        self._begin_addr_input.clear()
        self._end_addr_input.clear()

    def _on_delete_peripheral(self):
        """Deletes the selected row from the table and the data model."""
        current_row = self._peripheral_table.currentRow()
        if current_row > -1:
            # --- Remove the range from the data model first ---
            del self._configured_ranges[current_row]
            # Then remove the row from the view
            self._peripheral_table.removeRow(current_row)

    def _update_delete_button_state(self):
        self._delete_button.setEnabled(len(self._peripheral_table.selectedItems()) > 0)
