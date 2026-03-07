from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple, Type

from arm_emulator_rs import GpioPort, MemoryRegion  # type: ignore : import exists
from PyQt6.QtCore import QRegularExpression, Qt
from PyQt6.QtGui import QBrush, QColor, QPainter, QRegularExpressionValidator
from PyQt6.QtWidgets import (
    QComboBox,
    QFormLayout,
    QHeaderView,
    QLineEdit,
    QMessageBox,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget,
)


# The Peripheral "Factory" section
class PyGpioPort(GpioPort):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    def read32(self, addr: int) -> int:
        res: int = super().read32(addr)
        print("PyGpioPort read32")
        return res

    def write32(self, addr: int, data: int) -> None:
        res: None = super().write32(addr, data)
        print("PyGpioPort write32")
        return res

    def read_byte(self, addr: int) -> int:
        res: int = super().read_byte(addr)
        print("PyGpioPort read_byte")
        return res

    def write_byte(self, addr: int, data: int) -> None:
        res: None = super().write_byte(addr, data)
        print("PyGpioPort write_byte")
        return res

    def reset(self) -> None:
        res: None = super().reset()
        print("PyGpioPort reset")
        return res


PERIPHERAL_REGISTRY: Dict[str, Type] = {
    "LED": PyGpioPort,
}


# Custom LED Widget
class LedIndicator(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self._on = False
        self.setFixedSize(20, 20)

    def set_state(self, is_on: bool):
        if self._on != is_on:
            self._on = is_on
            self.update()  # Trigger paintEvent

    def paintEvent(self, a0) -> None:
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        color = QColor("#4CAF50") if self._on else QColor("#555555")
        painter.setBrush(QBrush(color))
        painter.setPen(Qt.PenStyle.NoPen)

        painter.drawEllipse(2, 2, 16, 16)


VALID_MEMORY_BEGIN: int = int(MemoryRegion.PERIPHERAL_BEGIN)
VALID_MEMORY_END: int = int(MemoryRegion.PERIPHERAL_END)


def parse_address(addr_str: str) -> Optional[int]:
    addr_str = addr_str.strip().lower()
    if not addr_str:
        return None
    try:
        return int(addr_str, 0)
    except ValueError:
        return None


@dataclass
class PeripheralData:
    type_name: str
    name: str
    start: int
    end: int
    instance: Any
    led_widget: "LedIndicator"


def get_default_peripheral():
    return PeripheralData(
        "LED", "led0", 0x40000000, 0x4000FFFF, PyGpioPort(), LedIndicator()
    )


class PeripheralsPanel(QWidget):
    """
    A widget panel for creating and configuring simulated peripherals
    with memory validation.
    """

    def __init__(self, parent: Optional[QWidget] = None) -> None:
        super().__init__(parent)

        self._peripherals_data: List[PeripheralData] = []

        # Data model to track configured memory ranges
        self._configured_ranges: List[Tuple[int, int]] = []

        # Widget initialization
        self._layout = QVBoxLayout(self)
        self.setLayout(self._layout)
        self._form_widget = QWidget()
        self._form_layout = QFormLayout(self._form_widget)
        self._type_combo = QComboBox()
        self._name_input = QLineEdit()
        self._begin_addr_input = QLineEdit()
        self._end_addr_input = QLineEdit()
        self._add_button = QPushButton()
        self._delete_button = QPushButton()
        self._peripheral_table = QTableWidget()

        self.setupUI()

        # Connections
        self._add_button.clicked.connect(self._on_add_peripheral)
        self._delete_button.clicked.connect(self._on_delete_peripheral)
        self._peripheral_table.itemSelectionChanged.connect(
            self._update_delete_button_state
        )

        self._add_peripheral_entry(get_default_peripheral())

        self.retranslateUi()

    def setupUI(self) -> None:
        self._form_layout.addRow("Type:", self._type_combo)
        self._form_layout.addRow("Instance Name:", self._name_input)
        self._form_layout.addRow("Begin Address:", self._begin_addr_input)
        self._form_layout.addRow("End Address:", self._end_addr_input)
        self._type_combo.addItems(PERIPHERAL_REGISTRY.keys())

        self._add_button.setText("")
        self._delete_button.setText("")

        name_regex = QRegularExpression("^[a-zA-Z_][a-zA-Z0-9_]*$")
        name_validator = QRegularExpressionValidator(name_regex)
        self._name_input.setValidator(name_validator)
        self._name_input.setPlaceholderText("e.g. LED_BANK (No spaces)")

        hex_regex = QRegularExpression("^(0x)?[0-9a-fA-F]+$")
        hex_validator = QRegularExpressionValidator(hex_regex)
        self._begin_addr_input.setValidator(hex_validator)
        self._end_addr_input.setValidator(hex_validator)

        self._peripheral_table.setColumnCount(4)

        header = self._peripheral_table.horizontalHeader()
        header.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)  # type: ignore
        header.setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)  # type: ignore
        header.setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)  # type: ignore
        header.setSectionResizeMode(3, QHeaderView.ResizeMode.Fixed)  # type: ignore
        self._peripheral_table.setColumnWidth(3, 50)

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

    def reset_peripherals(self) -> None:
        for p in self._peripherals_data:
            if hasattr(p.instance, "reset"):
                p.instance.reset()

            p.led_widget.set_state(False)

        print("Peripherals reset.")

    def _add_peripheral_entry(self, data: PeripheralData) -> None:
        """Adds a PeripheralData object to the table and internal lists."""
        row_count = self._peripheral_table.rowCount()
        self._peripheral_table.insertRow(row_count)

        memory_range_str = f"{hex(data.start)} - {hex(data.end)}"

        type_item = QTableWidgetItem(data.type_name)
        type_item.setToolTip(data.type_name)
        name_item = QTableWidgetItem(data.name)
        name_item.setToolTip(data.name)
        range_item = QTableWidgetItem(memory_range_str)
        range_item.setToolTip(memory_range_str)

        self._peripheral_table.setItem(row_count, 0, type_item)
        self._peripheral_table.setItem(row_count, 1, name_item)
        self._peripheral_table.setItem(row_count, 2, range_item)

        # Setup LED container
        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(data.led_widget)
        self._peripheral_table.setCellWidget(row_count, 3, container)

        self._peripherals_data.append(data)
        self._configured_ranges.append((data.start, data.end))

    def _on_add_peripheral(self) -> None:
        """Validates input against global and existing ranges, then adds."""
        p_type = self._type_combo.currentText()
        p_name = self._name_input.text().strip()
        start_addr = parse_address(self._begin_addr_input.text())
        end_addr = parse_address(self._end_addr_input.text())

        # Basic Validation
        if not p_name or start_addr is None or end_addr is None:
            QMessageBox.warning(
                self,
                self.tr("Input Error"),
                self.tr("All fields must be filled with valid values."),
            )
            return

        for p in self._peripherals_data:
            if p.name == p_name:
                QMessageBox.warning(
                    self,
                    self.tr("Input Error"),
                    self.tr("Peripheral name '{}' already exists.").format(p_name),
                )
                return

        if start_addr > end_addr:
            QMessageBox.warning(
                self,
                self.tr("Input Error"),
                self.tr("Start address must not be greater than end address."),
            )
            return

        # Enforce Global Memory Range
        if not (
            VALID_MEMORY_BEGIN <= start_addr <= VALID_MEMORY_END
            and VALID_MEMORY_BEGIN <= end_addr <= VALID_MEMORY_END
        ):
            msg = self.tr(
                "Memory addresses must be within the valid range:\n{} - {}"
            ).format(hex(VALID_MEMORY_BEGIN), hex(VALID_MEMORY_END))
            QMessageBox.warning(self, self.tr("Address Out of Range"), msg)
            return

        for exist_start, exist_end in self._configured_ranges:
            # Classic interval overlap check
            if start_addr <= exist_end and end_addr >= exist_start:
                msg = self.tr(
                    "The proposed memory range ({} - {}) "
                    "overlaps with an existing peripheral's range "
                    "({} - {})."
                ).format(
                    hex(start_addr), hex(end_addr), hex(exist_start), hex(exist_end)
                )
                QMessageBox.warning(self, self.tr("Memory Overlap"), msg)
                return

        # All checks passed, add to table AND data model
        peripheral_class = PERIPHERAL_REGISTRY[p_type]
        instance = peripheral_class()

        # Create Data Object
        data = PeripheralData(
            type_name=p_type,
            name=p_name,
            start=start_addr,
            end=end_addr,
            instance=instance,
            led_widget=LedIndicator(),
        )

        # Use the helper to add it
        self._add_peripheral_entry(data)

        self._name_input.clear()
        self._begin_addr_input.clear()
        self._end_addr_input.clear()

    def _on_delete_peripheral(self) -> None:
        """Deletes the selected row from the table and the data model."""
        current_row = self._peripheral_table.currentRow()
        if current_row > -1:
            # Remove the range from the data model first
            del self._configured_ranges[current_row]
            del self._peripherals_data[current_row]
            # Then remove the row from the view
            self._peripheral_table.removeRow(current_row)

    def _update_delete_button_state(self) -> None:
        self._delete_button.setEnabled(len(self._peripheral_table.selectedItems()) > 0)

    def get_defined_symbols(self) -> Dict[str, int]:
        """
        Returns a dictionary of symbol names to start addresses
        for all configured peripherals.
        """
        symbols = {}
        for p in self._peripherals_data:
            symbols[p.name] = p.start
        return symbols

    def get_peripherals(self) -> List[Tuple[int, int, Any]]:
        """
        Returns a list of (start_address, size_bytes, instance_object)
        compatible with the Rust emulator's add_python_peripheral.
        """
        results = []
        for p in self._peripherals_data:
            start = p.start
            end = p.end
            instance = p.instance
            results.append((start, end, instance))
        return results

    def update_view(self) -> None:
        """Called by timer/controller to update LEDs."""
        for p in self._peripherals_data:
            # instance = p.instance
            # led_widget = p.led_widget
            # if hasattr(instance, "is_led_on"):
            #     is_on = instance.is_led_on()
            #     led_widget.set_state(is_on)
            if hasattr(p.instance, "is_led_on"):
                is_on = p.instance.is_led_on()
                p.led_widget.set_state(is_on)

    def retranslateUi(self):
        """Refreshes all visible text."""
        # 1. Update Form Labels
        # QFormLayout.labelForField returns the QLabel associated with the input widget

        type_label = self._form_layout.labelForField(self._type_combo)
        if type_label is not None:
            type_label.setText(self.tr("Type:"))  # type: ignore : method exists

        name_label = self._form_layout.labelForField(self._name_input)
        if name_label:
            name_label.setText(self.tr("Instance Name:"))  # type: ignore : method exists

        start_label = self._form_layout.labelForField(self._begin_addr_input)
        if start_label:
            start_label.setText(self.tr("Begin Address:"))  # type: ignore : method exists

        end_label = self._form_layout.labelForField(self._end_addr_input)
        if end_label:
            end_label.setText(self.tr("End Address:"))  # type: ignore : method exists

        # 2. Update Buttons
        self._add_button.setText(self.tr("Add Peripheral"))
        self._delete_button.setText(self.tr("Delete Selected"))

        # 3. Update Table Headers
        self._peripheral_table.setHorizontalHeaderLabels(
            [
                self.tr("Type"),
                self.tr("Name"),
                self.tr("Memory Range"),
                self.tr("State"),
            ]
        )

        # 4. Update tooltips
        addr_tooltip = self.tr("Valid address between {} ({}) and {} ({})").format(
            hex(VALID_MEMORY_BEGIN),
            VALID_MEMORY_BEGIN,
            hex(VALID_MEMORY_END),
            VALID_MEMORY_END,
        )
        self._begin_addr_input.setToolTip(addr_tooltip)
        self._end_addr_input.setToolTip(addr_tooltip)

        # 5. Update placeholder text
        placeholder_text = self.tr("Hex or Decimal")
        self._begin_addr_input.setPlaceholderText(placeholder_text)
        self._end_addr_input.setPlaceholderText(placeholder_text)
