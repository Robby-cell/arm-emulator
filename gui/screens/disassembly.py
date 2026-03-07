from typing import Optional

from arm_emulator_rs import Emulator, ExecutionError  # type: ignore
from capstone import (
    Cs,
    CS_ARCH_ARM,
    CS_MODE_ARM,
    CS_MODE_BIG_ENDIAN,
)
from PyQt6.QtCore import QRegularExpression
from PyQt6.QtGui import QColor, QFont, QRegularExpressionValidator
from PyQt6.QtWidgets import (
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget,
    QCheckBox,
)
from PyQt6.QtCore import Qt


class DisassemblyScreen(QWidget):
    ROWS_TO_DISPLAY = 25

    def __init__(
        self,
        emulator: Emulator,
        parent: Optional[QWidget] = None,
    ) -> None:
        super().__init__(parent)

        if emulator is None:
            raise ValueError("DisassemblyScreen requires a valid Emulator instance.")

        self.emulator = emulator
        self._current_start_address = 0
        self._is_following_pc = True

        # Initialize Capstone Disassembler
        try:
            self.md = Cs(CS_ARCH_ARM, CS_MODE_ARM + CS_MODE_BIG_ENDIAN)
        except Exception as e:
            print(f"Failed to initialize Capstone: {e}")
            self.md = None

        # UI Widgets
        self._layout = QVBoxLayout(self)
        self._controls_widget = QWidget()
        self._controls_layout = QHBoxLayout(self._controls_widget)

        self._address_input = QLineEdit()
        self._go_button = QPushButton(self.tr("Go"))
        self._follow_pc_check = QCheckBox(self.tr("Follow PC"))
        self._follow_pc_check.setChecked(True)

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
        self._controls_layout.addWidget(self._follow_pc_check)

        self._address_input.setPlaceholderText("e.g., 0x0000")
        hex_regex = QRegularExpression("^(0x)?[0-9a-fA-F]+$")
        self._address_input.setValidator(QRegularExpressionValidator(hex_regex))

        # Setup Table
        # Columns: Address | Hex | Instruction | Operands
        self._table.setColumnCount(4)
        headers = ["Address", "Bytes", "Opcode", "Operands"]
        self._table.setHorizontalHeaderLabels(headers)

        self._table.setLayoutDirection(Qt.LayoutDirection.LeftToRight)

        self._table.setFont(QFont("monospace", 10))
        self._table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self._table.setShowGrid(False)
        self._table.verticalHeader().setVisible(False)  # type: ignore : not None
        self._table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)

        header_view: QHeaderView = self._table.horizontalHeader()  # type: ignore : not None
        header_view.setSectionResizeMode(
            0, QHeaderView.ResizeMode.ResizeToContents
        )  # Address
        header_view.setSectionResizeMode(
            1, QHeaderView.ResizeMode.ResizeToContents
        )  # Bytes
        header_view.setSectionResizeMode(
            2, QHeaderView.ResizeMode.ResizeToContents
        )  # Opcode
        header_view.setSectionResizeMode(3, QHeaderView.ResizeMode.Stretch)  # Operands

        self._layout.addWidget(self._controls_widget)
        self._layout.addWidget(self._table)

    def _init_connections(self) -> None:
        self._go_button.clicked.connect(self._on_go_to_address)
        self._address_input.returnPressed.connect(self._on_go_to_address)
        self._follow_pc_check.toggled.connect(self._on_follow_pc_toggled)

    def _on_follow_pc_toggled(self, checked: bool) -> None:
        self._is_following_pc = checked
        if checked:
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
            return

        self._is_following_pc = False
        self._follow_pc_check.setChecked(False)
        self._current_start_address = address
        self.update_view()

    def update_view(self) -> None:
        if not self.md:
            return

        # Determine start address
        # Get Current PC from Emulator registers (R15 is index 15)
        regs = self.emulator.registers
        current_pc = regs[15]

        if self._is_following_pc:
            # If following PC, start slightly before PC to show context
            start_addr = max(0, current_pc - 16)
            self._current_start_address = start_addr
        else:
            start_addr = self._current_start_address

        # Read memory chunk
        code_buffer = bytearray()

        # Read enough bytes to fill the view (4 bytes per instruction approx)
        bytes_to_read = self.ROWS_TO_DISPLAY * 4

        valid_read = False
        for i in range(bytes_to_read):
            try:
                byte = self.emulator.read_byte(start_addr + i)
                code_buffer.append(byte)
                valid_read = True
            except (ValueError, ExecutionError):
                # Stop reading if we hit unmapped memory
                break

        if not valid_read:
            self._table.setRowCount(0)
            return

        # Disassemble using Capstone
        try:
            instructions = list(self.md.disasm(bytes(code_buffer), start_addr))
        except Exception as e:
            print(f"Disassembly error: {e}")
            instructions = []

        # Update Table
        self._table.setRowCount(len(instructions))

        font_mono = QFont("monospace", 10)
        font_bold = QFont("monospace", 10, QFont.Weight.Bold)

        highlight_color = QColor("#3A3d41")  # Dark grey for highlighting PC
        default_bg = QColor("#1e1e1e")  # Or transparent/default

        for row, insn in enumerate(instructions):
            # PC Highlighting Logic
            is_current_pc = insn.address == current_pc
            bg_color = highlight_color if is_current_pc else default_bg

            # Address
            addr_item = QTableWidgetItem(f"0x{insn.address:08X}")
            addr_item.setFont(font_mono)
            addr_item.setBackground(bg_color)
            addr_item.setForeground(QColor("#aaaaaa"))
            self._table.setItem(row, 0, addr_item)

            # Hex Bytes
            hex_bytes = " ".join([f"{b:02X}" for b in insn.bytes])
            bytes_item = QTableWidgetItem(hex_bytes)
            bytes_item.setFont(font_mono)
            bytes_item.setBackground(bg_color)
            bytes_item.setForeground(QColor("#569CD6"))  # Blue-ish
            self._table.setItem(row, 1, bytes_item)

            # Mnemonic (Opcode)
            mnem_item = QTableWidgetItem(insn.mnemonic.upper())
            mnem_item.setFont(font_bold)
            mnem_item.setBackground(bg_color)
            mnem_item.setForeground(QColor("#C586C0"))  # Purple-ish
            self._table.setItem(row, 2, mnem_item)

            # Operands
            op_item = QTableWidgetItem(insn.op_str.upper())
            op_item.setFont(font_mono)
            op_item.setBackground(bg_color)
            op_item.setForeground(QColor("#dddddd"))
            self._table.setItem(row, 3, op_item)

            # Auto-scroll to PC if enabled
            if is_current_pc and self._is_following_pc:
                self._table.scrollToItem(addr_item)

    def set_endianness(self, is_little_endian: bool) -> None:
        """Re-initializes Capstone with the correct endianness."""
        # mode = CS_MODE_LITTLE_ENDIAN if is_little_endian else CS_MODE_BIG_ENDIAN

        # Because we fetch the number and rust automatically converts it to integer for us.
        # We need to use big endian, unless fetching bytes individually, which won't happen
        mode = CS_MODE_BIG_ENDIAN
        try:
            self.md = Cs(CS_ARCH_ARM, CS_MODE_ARM + mode)
            # Refresh the view immediately if visible
            if self.isVisible():
                self.update_view()
        except Exception as e:
            print(f"Failed to switch Capstone endianness: {e}")

    def retranslateUi(self) -> None:
        if hasattr(self, "_go_label"):
            self._go_label.setText(self.tr("Go to Address:"))
        self._go_button.setText(self.tr("Go"))
        self._follow_pc_check.setText(self.tr("Follow PC"))

        headers = ["Address", "Bytes", "Opcode", "Operands"]
        self._table.setHorizontalHeaderLabels(headers)
