from typing import Optional
from PyQt6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QTextBrowser,
    QPushButton,
    QHBoxLayout,
    QWidget,
)


class TutorialDialog(QDialog):
    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setWindowTitle(self.tr("ARM Emulator - Quick Start Guide"))
        self.resize(700, 500)

        self._layout = QVBoxLayout(self)

        # QTextBrowser supports rich text / basic HTML
        self._text_browser = QTextBrowser()
        self._text_browser.setOpenExternalLinks(True)
        self._text_browser.setStyleSheet("""
            QTextBrowser {
                background-color: #1e1e1e;
                color: #dddddd;
                font-family: 'Segoe UI', sans-serif;
                font-size: 14px;
                border: none;
            }
            h1, h2, h3 { color: #569CD6; }
            code { background-color: #2b2b2b; color: #CE9178; padding: 2px 4px; border-radius: 3px; font-family: monospace; }
            pre { background-color: #2b2b2b; padding: 10px; border-radius: 5px; font-family: monospace; }
        """)

        self._set_tutorial_content()

        self._layout.addWidget(self._text_browser)

        # Close button
        btn_layout = QHBoxLayout()
        btn_layout.addStretch()
        close_btn = QPushButton(self.tr("Got it!"))
        close_btn.setFixedWidth(100)
        close_btn.clicked.connect(self.accept)
        btn_layout.addWidget(close_btn)

        self._layout.addLayout(btn_layout)

    def _set_tutorial_content(self):
        """Sets the HTML content of the tutorial."""

        # Using self.tr() so the whole manual can be translated later!
        html_content = self.tr("""
        <h1>Welcome to the ARM Simulator</h1>
        <p>This tool is designed to help you learn ARMv7 Assembly language and how software interacts with hardware peripherals.</p>
        
        <h2>1. The Interface</h2>
        <ul>
            <li><b>Editor:</b> Write your ARM assembly here. Click the gutter (left margin) to set <b>Breakpoints</b> (red dots).</li>
            <li><b>CPU Panel (Right):</b> Watch the Registers (R0-R15) and CPSR Flags (N, Z, C, V) change in real-time. Changed values highlight in red.</li>
            <li><b>Memory View:</b> Inspect the raw RAM byte-by-byte. Scroll infinitely through the 4GB address space.</li>
            <li><b>Disassembly:</b> See exactly how your assembly text compiled into machine code and how the CPU interprets it.</li>
        </ul>

        <h2>2. Controls & Shortcuts</h2>
        <ul>
            <li><code>F7</code> - <b>Load/Build:</b> Compiles your code and loads it into memory at <code>0x00000000</code> without starting execution.</li>
            <li><code>F5</code> - <b>Run:</b> Executes the code continuously until a breakpoint or the exit syscall.</li>
            <li><code>F10</code> - <b>Step:</b> Executes exactly one instruction. Great for watching registers change!</li>
            <li><code>Shift+F5</code> - <b>Stop:</b> Halts a running program.</li>
            <li><code>Ctrl+R</code> - <b>Reset:</b> Clears CPU state, rewinds to start, and resets all hardware peripherals.</li>
        </ul>

        <h2>3. Hardware Peripherals (The LED)</h2>
        <p>You can simulate real hardware. Add a <b>GPIO Port</b> peripheral mapped to an address (e.g., <code>0x40000000</code>).</p>
        <p>To turn the LED on, you must write specific bits to its memory-mapped registers:</p>
        <pre>
LDR R0, =led0       @ Load the base address of the peripheral
MOV R1, #0x400
STR R1, [R0]        @ Write to MODER (Offset 0x00) to set as Output
MOV R1, #0x20
STR R1, [R0, #0x14] @ Write to ODR (Offset 0x14) to turn LED High
        </pre>

        <h2>4. Exiting your program</h2>
        <p>To tell the simulator your program is done, use the Linux-style exit system call:</p>
        <pre>
MOV R7, #1      @ Syscall 1 is 'exit'
MOV R0, #0      @ Return code 0 (Success)
SVC 0           @ Trigger Supervisor Call
        </pre>
        """)
        self._text_browser.setHtml(html_content)
