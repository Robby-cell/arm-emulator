from typing import Optional

from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import QHBoxLayout, QSplitter, QWidget

from ..widgets.code_editor import CodeEditor
from ..widgets.peripherals_panel import PeripheralsPanel

DEFAULT_ASM = r"""_start:
    B main

turn_on:
    @ Save return address
    PUSH {LR}

    @ 1. Configure PA5 as Output
    @ We need bits 11:10 of MODER (Offset 0x00) to be '01'.
    @ Binary: ... 0000 0100 0000 0000
    @ Hex:    0x400
    MOV R1, #0x400
    STR R1, [R0]        @ Write to MODER (Offset 0)

    @ 2. Set PA5 High
    @ We need bit 5 of ODR (Offset 0x14) to be '1'.
    @ Binary: ... 0010 0000
    @ Hex:    0x20
    MOV R1, #0x20
    STR R1, [R0, #0x14] @ Write to ODR (Offset 20)

    @ Restore return address
    POP {PC}

turn_off:
    PUSH {LR}

    @ 1. Configure PA5 as Output
    MOV R1, #0x400
    STR R1, [R0]        @ Write to MODER (Offset 0)

    @ 2. Set PA5 Low
    MOV R1, #0x00
    STR R1, [R0, #0x14] @ Write to ODR (Offset 20)

    POP {PC}

main:
    MOV R2, #0

loop:
    LDR R0, =led0
    BL turn_on

    LDR R0, =led0
    BL turn_off

    ADD R2, R2, #1
    CMP R2, #0x3
    BNE loop

    MOV R7, #1 @ Exit syscall
    MOV R0, #0 @ Exit code 0
    SVC 0      @ Supervisor call

"""


class EditorScreen(QWidget):
    def __init__(self, parent: Optional[QWidget] = None) -> None:
        super().__init__(parent)

        self._layout = QHBoxLayout(self)
        self.setLayout(self._layout)
        self._layout.setContentsMargins(0, 0, 0, 0)

        self._splitter = QSplitter(Qt.Orientation.Horizontal)
        self._peripherals = PeripheralsPanel()
        self._editor = CodeEditor()

        self.setupUI()

    def setupUI(self) -> None:
        self._splitter.addWidget(self._peripherals)
        self._splitter.addWidget(self._editor)

        # This gives the peripherals panel 40% and the editor 70% of the space initially.
        self._splitter.setSizes([300, 700])
        self._splitter.setStretchFactor(1, 1)  # Allows the editor to expand more

        # Add the fully configured splitter to the main layout
        self._layout.addWidget(self._splitter)

        self._editor.setPlainText(DEFAULT_ASM)

    def get_code(self) -> str:
        """A method to allow the MainWindow to retrieve the code."""
        return self._editor.toPlainText()

    def retranslateUi(self) -> None:
        self._peripherals.retranslateUi()
