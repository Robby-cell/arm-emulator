from PyQt6.QtCore import Qt, QRect, QRegularExpression, pyqtSignal
from PyQt6.QtGui import (
    QPainter,
    QColor,
    QFont,
    QTextBlockUserData,
    QTextCursor,
    QSyntaxHighlighter,
    QTextCharFormat,
    QTextDocument,
    QTextFormat,
)
from PyQt6.QtWidgets import QWidget, QPlainTextEdit, QToolTip, QTextEdit
from typing import Optional, List


CONDITIONAL_REGEX = "EQ|NE|CS|HS|CC|LO|MI|PL|VS|VC|HI|LS|GE|LT|GT|LE|AL|NV"

ARMITHMETIC_INSTRUCTIONS = [
    "ADD",
    "SUB",
    "RSB",
    "ADC",
    "SBC",
    "RSC",
    "AND",
    "ORR",
    "EOR",
    "BIC",
    "MOV",
    "MVN",
    "ORN",
    "ASR",
    "LSL",
    "LSR",
    "ROR",
    "RRX",
    "MUL",
    "MLA",
]

FLAG_INSTRUCTIONS = [
    "CMP",
    "CMN",
    "TST",
    "TEQ",
]

NON_UPDATING_INSTRUCTIONS = ["LDR", "STR", "B", "BL", "PUSH", "POP", "SVC"]

ARM_REGISTERS = [
    "R0",
    "R1",
    "R2",
    "R3",
    "R4",
    "R5",
    "R6",
    "R7",
    "R8",
    "R9",
    "R10",
    "R11",
    "R12",
    "SP",
    "LR",
    "PC",
    "R13",
    "R14",
    "R15",
]

# Simple tooltips for common instructions
INSTRUCTION_TOOLTIPS = {
    "ADD": "ADD <dest>, <op1>, <op2>\n\nAdds two operands and stores the result in a register.",
    "SUB": "SUB <dest>, <op1>, <op2>\n\nSubtracts two operands and stores the result.",
    "MOV": "MOV <dest>, <op>\n\nMoves a value into a register.",
    "LDR": "LDR <reg>, [<address>]\n\nLoads a value from memory into a register.",
    "STR": "STR <reg>, [<address>]\n\nStores a value from a register into memory.",
    "B": "B <label>\n\nUnconditional branch to a label.",
    "BL": "BL <label>\n\nBranch with Link. Branches to a label and stores the return address in LR.",
    "CMP": "CMP <op1>, <op2>\n\nCompares two operands and sets condition flags.",
    "BEQ": "BEQ <label>\n\nBranch if Equal (Z flag is set).",
    "BNE": "BNE <label>\n\nBranch if Not Equal (Z flag is clear).",
}


class ARMHighlighter(QSyntaxHighlighter):
    """
    Handles syntax highlighting for ARM assembly.
    """

    def __init__(self, parent) -> None:
        super().__init__(parent)
        self._highlighting_rules = []
        self._labels = set()

        # Arithmetic/Logic format
        arithmetic_format = QTextCharFormat()
        arithmetic_format.setForeground(QColor("#569CD6"))  # Blue
        arithmetic_format.setFontWeight(QFont.Weight.Bold)

        # Flag-setting instruction format (e.g., CMP, TST)
        flag_format = QTextCharFormat()
        flag_format.setForeground(QColor("#DCDCAA"))  # Gold/Yellow
        flag_format.setFontWeight(QFont.Weight.Bold)

        # Control flow / Memory instruction format (e.g., LDR, B, STR, BEQ)
        control_mem_format = QTextCharFormat()
        control_mem_format.setForeground(QColor("#C586C0"))  # Magenta/Light Purple
        control_mem_format.setFontWeight(QFont.Weight.Bold)

        conditional_part = f"({CONDITIONAL_REGEX})?"

        # Pattern for arithmetic instructions (handles optional 'S' and conditional)
        arithmetic_pattern = QRegularExpression(
            r"\b(("
            + "|".join(ARMITHMETIC_INSTRUCTIONS)
            + r")"
            + conditional_part
            + r"S?)\b",
            QRegularExpression.PatternOption.CaseInsensitiveOption,
        )

        # Pattern for flag-setting instructions (handles optional conditional)
        flag_pattern = QRegularExpression(
            r"\b((" + "|".join(FLAG_INSTRUCTIONS) + r")" + conditional_part + r")\b",
            QRegularExpression.PatternOption.CaseInsensitiveOption,
        )

        # Pattern for control/memory instructions (handles optional conditional)
        control_mem_pattern = QRegularExpression(
            r"\b(("
            + "|".join(NON_UPDATING_INSTRUCTIONS)
            + r")"
            + conditional_part
            + r")\b",
            QRegularExpression.PatternOption.CaseInsensitiveOption,
        )
        self._highlighting_rules.append((arithmetic_pattern, arithmetic_format))
        self._highlighting_rules.append((flag_pattern, flag_format))
        self._highlighting_rules.append((control_mem_pattern, control_mem_format))

        # Register format
        register_format = QTextCharFormat()
        register_format.setForeground(QColor("#4EC9B0"))  # Teal
        self._highlighting_rules.append(
            (
                QRegularExpression(
                    r"\b(" + "|".join(ARM_REGISTERS) + r")\b",
                    QRegularExpression.PatternOption.CaseInsensitiveOption,
                ),
                register_format,
            )
        )

        # Number format (hex, decimal)
        number_format = QTextCharFormat()
        number_format.setForeground(QColor("#B5CEA8"))  # Greenish
        self._highlighting_rules.append(
            (QRegularExpression(r"\b#?0x[0-9a-fA-F]+\b|\b#?[0-9]+\b"), number_format)
        )

        # Comment format
        comment_format = QTextCharFormat()
        comment_format.setForeground(QColor("#6A9955"))  # Dark Green
        comment_format.setFontItalic(True)
        self._highlighting_rules.append(
            (QRegularExpression(r";.*$|@.*$"), comment_format)
        )

        # String format
        string_format = QTextCharFormat()
        string_format.setForeground(QColor("#CE9178"))  # Orange
        self._highlighting_rules.append((QRegularExpression(r'"[^"]*"'), string_format))

        # Label definition format
        self.label_def_format = QTextCharFormat()
        self.label_def_format.setForeground(QColor("#C586C0"))  # Purple
        self.label_def_format.setFontWeight(QFont.Weight.Bold)

    def update_labels(self, labels: set) -> None:
        """Updates the set of known labels to highlight them as well."""
        self._labels = labels
        self.rehighlight()

    def highlightBlock(self, text: Optional[str]) -> None:
        match_iterator = QRegularExpression(
            r"^\s*([a-zA-Z_][a-zA-Z0-9_]*):"
        ).globalMatch(text)
        while match_iterator.hasNext():
            match = match_iterator.next()
            self.setFormat(
                match.capturedStart(1), match.capturedLength(1), self.label_def_format
            )

        for pattern, format in self._highlighting_rules:
            match_iterator = pattern.globalMatch(text)
            while match_iterator.hasNext():
                match = match_iterator.next()
                self.setFormat(match.capturedStart(), match.capturedLength(), format)

        if self._labels:
            label_usage_format = QTextCharFormat()
            label_usage_format.setForeground(QColor("#C586C0"))  # Purple
            label_pattern = QRegularExpression(r"\b(" + "|".join(self._labels) + r")\b")
            match_iterator = label_pattern.globalMatch(text)
            while match_iterator.hasNext():
                match = match_iterator.next()
                self.setFormat(
                    match.capturedStart(), match.capturedLength(), label_usage_format
                )


class BreakpointUserData(QTextBlockUserData):
    """Holds the breakpoint state for a QTextBlock."""

    def __init__(self, is_breakpoint: bool = False) -> None:
        super().__init__()
        self.is_breakpoint = is_breakpoint


class LineNumberArea(QWidget):
    """The gutter widget, displaying line numbers and breakpoints."""

    _background_color: QColor
    _text_color: QColor
    _breakpoint_color: QColor

    def __init__(self, editor: "CodeEditor") -> None:
        super().__init__(editor)
        self._editor = editor
        self._font = QFont("monospace", 10)

        self._background_color = QColor("#444")
        self._text_color = QColor("#FFFFFF")
        self._breakpoint_color = QColor("red")

    @property
    def background_color(self) -> QColor:
        return self._background_color

    @background_color.setter
    def background_color(self, color: QColor) -> None:
        if color is None:
            return
        self._background_color = color
        self.update()

    @property
    def text_color(self) -> QColor:
        return self._text_color

    @text_color.setter
    def text_color(self, color: QColor) -> None:
        self._text_color = color
        self.update()

    @property
    def breakpoint_color(self) -> QColor:
        return self._breakpoint_color

    @breakpoint_color.setter
    def breakpoint_color(self, color: QColor) -> None:
        self._breakpoint_color = color
        self.update()

    def paintEvent(self, a0) -> None:
        if a0 is None:
            return

        painter = QPainter(self)
        painter.fillRect(a0.rect(), self.background_color)

        block = self._editor.firstVisibleBlock()
        block_number = block.blockNumber()
        top_y = (
            self._editor.blockBoundingGeometry(block)
            .translated(self._editor.contentOffset())
            .top()
        )

        last_visible_line = self._editor.blockCount() - 1

        while block.isValid() and top_y <= a0.rect().bottom():
            if block.isVisible() and block_number < last_visible_line:
                # Draw line number (right-aligned)
                number = str(block_number + 1)
                painter.setPen(self.text_color)
                painter.setFont(self._font)
                painter.drawText(
                    0,
                    int(top_y),
                    self.width() - 5,
                    self._editor.fontMetrics().height(),
                    Qt.AlignmentFlag.AlignRight,
                    number,
                )

                # Draw breakpoint dot
                if self._editor.is_breakpoint(block_number):
                    block_height = self._editor.blockBoundingRect(block).height()
                    dot_y = int(top_y + (block_height / 2) - 7)
                    dot_x = 5
                    painter.setBrush(self.breakpoint_color)
                    painter.setPen(self.breakpoint_color)
                    painter.drawEllipse(dot_x, dot_y, 10, 10)

            block = block.next()
            block_number += 1
            if block.isValid():
                top_y = (
                    self._editor.blockBoundingGeometry(block)
                    .translated(self._editor.contentOffset())
                    .top()
                )

    def mousePressEvent(self, a0) -> None:
        if a0 is not None and a0.button() == Qt.MouseButton.LeftButton:
            clicked_y = a0.pos().y()
            block = self._editor.firstVisibleBlock()
            content_offset_y = self._editor.contentOffset().y()
            block_top = (
                self._editor.blockBoundingGeometry(block)
                .translated(0, content_offset_y)
                .top()
            )

            while block.isValid():
                block_height = self._editor.blockBoundingRect(block).height()
                if block_top <= clicked_y < block_top + block_height:
                    self._editor.toggle_breakpoint(block.blockNumber())
                    return
                block = block.next()
                block_top += block_height


class CodeEditor(QPlainTextEdit):
    """A code editor with a stable gutter, using a 'guard line' at the end."""

    GUTTER_FIXED_WIDTH = 60
    breakpoint_toggled = pyqtSignal(int, bool)

    def __init__(
        self,
        line_number_area: Optional[LineNumberArea] = None,
        parent: Optional[QWidget] = None,
    ) -> None:
        super().__init__(parent)
        self._line_number_area = (
            line_number_area if line_number_area is not None else LineNumberArea(self)
        )

        self.setFont(QFont("monospace", 12))
        self.setLineWrapMode(QPlainTextEdit.LineWrapMode.NoWrap)
        self._update_tab_stop_width()

        self.setViewportMargins(self.GUTTER_FIXED_WIDTH, 0, 0, 0)

        # Connect signals
        self.updateRequest.connect(self._update_gutter)
        self.textChanged.connect(self._enforce_guard_line)
        self.cursorPositionChanged.connect(self._prevent_cursor_on_last_line)

        # Start with the guard line already present
        # If we don't do this, the breakpoint circle gets stretched weirdly when inserting a new line, when
        # it is set on the last line. Do this to avoid that.
        self.setPlainText("\n")

        # Syntax Highlighting
        self._highlighter = ARMHighlighter(self.document())

        # Track Labels
        self._labels = set()
        self.textChanged.connect(self._update_labels)

        self.setFont(QFont("monospace", 12))
        self.setLineWrapMode(QPlainTextEdit.LineWrapMode.NoWrap)

        self._setup_shortcuts()

    def set_execution_line(self, line_number: int) -> None:
        """Highlights the background of the specified line number."""
        extra_selections = []

        if line_number >= 0:
            selection = QTextEdit.ExtraSelection()
            line_color = QColor("#3A3d41")  # Subtle highlight (VSCode style debug line)
            # Or brighter: QColor("#5c5c3d") (Yellowish tint)

            selection.format.setBackground(line_color)
            selection.format.setProperty(QTextFormat.Property.FullWidthSelection, True)

            # Find the block for the line
            doc: QTextDocument = self.document()  # type: ignore
            block = doc.findBlockByNumber(line_number)

            if block.isValid():
                cursor = self.textCursor()
                cursor.setPosition(block.position())
                selection.cursor = cursor
                extra_selections.append(selection)

        self.setExtraSelections(extra_selections)

    def _setup_shortcuts(self):
        """Initializes all editor-specific keyboard shortcuts."""
        ...

    def _update_tab_stop_width(self) -> None:
        """Sets the tab stop distance to be equivalent to 4 space characters."""
        font_metrics = self.fontMetrics()
        space_width = font_metrics.horizontalAdvance(" ")
        self.setTabStopDistance(4 * space_width)

    def setFont(self, a0: QFont) -> None:
        """Overrides the base setFont method to also update the tab width."""
        super().setFont(a0)
        self._update_tab_stop_width()

    def _update_labels(self) -> None:
        """Parse the entire document to find label definitions."""
        new_labels = set()
        block = self.document().firstBlock()  # type: ignore
        while block.isValid():
            text = block.text()
            match = QRegularExpression(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*):").match(text)
            if match.hasMatch():
                new_labels.add(match.captured(1))
            block = block.next()

        if self._labels != new_labels:
            self._labels = new_labels
            self._highlighter.update_labels(self._labels)

    def keyPressEvent(self, e) -> None:
        """Handle key presses for autocompletion and auto-indent."""
        if e is None:
            return

        # First, check for the Ctrl+Enter shortcut
        if (
            e.key() in (Qt.Key.Key_Return, Qt.Key.Key_Enter)
            and e.modifiers() == Qt.KeyboardModifier.ControlModifier
        ):
            cursor = self.textCursor()
            current_line_text = cursor.block().text()

            # Find the indentation of the current line
            indentation: str = str()
            match = QRegularExpression(r"^(\s*).*").match(current_line_text)
            if match.hasMatch():
                indentation = match.captured(1)

            if (
                QRegularExpression(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*):")
                .match(current_line_text)
                .hasMatch()
            ):
                indentation += "\t"

            # Manually insert a newline character at the end of the line, followed by the indent
            cursor.movePosition(QTextCursor.MoveOperation.EndOfLine)
            cursor.insertText("\n" + indentation)
            self.setTextCursor(cursor)

            # We have handled the event. Do not process it further.
            return

        # Second, check for a regular Enter press for auto-indentation
        if e.key() in (Qt.Key.Key_Return, Qt.Key.Key_Enter):
            cursor = self.textCursor()
            current_line_text = cursor.block().text()

            indentation = ""
            match = QRegularExpression(r"^(\s*).*").match(current_line_text)
            if match.hasMatch():
                indentation = match.captured(1)

            # If the current line was a label, add one more level of indent for the next line
            if (
                QRegularExpression(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*):")
                .match(current_line_text)
                .hasMatch()
            ):
                indentation += "\t"

            # Let the default Enter action happen first
            super().keyPressEvent(e)
            # Then insert our calculated indentation
            self.insertPlainText(indentation)
            return

        # For all other keys, use the default behavior
        super().keyPressEvent(e)

    # Tooltips on Hover
    def event(self, e) -> bool:
        """Show tooltips when hovering over instructions."""
        if e is None:
            return False

        if e.type() == e.Type.ToolTip:
            pos = e.pos()  # type: ignore
            cursor = self.cursorForPosition(pos)
            cursor.select(QTextCursor.SelectionType.WordUnderCursor)
            word = cursor.selectedText().upper()

            if word in INSTRUCTION_TOOLTIPS:
                tooltip_text = INSTRUCTION_TOOLTIPS[word]
                QToolTip.showText(self.mapToGlobal(pos), tooltip_text, self)
                return True
        return super().event(e)

    def _update_gutter(self, rect: QRect, dy: int):
        if dy:
            self._line_number_area.scroll(0, dy)
        else:
            self._line_number_area.update(
                0, rect.y(), self._line_number_area.width(), rect.height()
            )

    def _enforce_guard_line(self) -> None:
        """Ensures there is always one, and only one, empty line at the end."""
        self.blockSignals(True)
        doc = self.document()
        if doc is None:
            return

        last_block = doc.lastBlock()
        if last_block.text() != "":
            cursor = self.textCursor()
            cursor.movePosition(QTextCursor.MoveOperation.End)
            cursor.insertBlock()
        self.blockSignals(False)

    def _prevent_cursor_on_last_line(self) -> None:
        """Stops the user from selecting or writing on the guard line."""
        cursor = self.textCursor()
        if cursor.blockNumber() == self.blockCount() - 1:
            self.blockSignals(True)
            cursor.movePosition(QTextCursor.MoveOperation.PreviousBlock)
            cursor.movePosition(QTextCursor.MoveOperation.EndOfLine)
            self.setTextCursor(cursor)
            self.blockSignals(False)

    def resizeEvent(self, e) -> None:
        super().resizeEvent(e)
        cr = self.contentsRect()
        self._line_number_area.setGeometry(
            QRect(cr.left(), cr.top(), self.GUTTER_FIXED_WIDTH, cr.height())
        )

    def toggle_breakpoint(self, line_number: int) -> None:
        if line_number >= self.blockCount() - 1:
            return

        doc = self.document()
        block = doc.findBlockByNumber(line_number)
        if not block.isValid():
            return

        data = block.userData()
        if not data:
            data = BreakpointUserData()

        # Toggle state
        new_state = not data.is_breakpoint
        data.is_breakpoint = new_state
        block.setUserData(data)
        self._line_number_area.update()

        # EMIT SIGNAL
        self.breakpoint_toggled.emit(line_number, new_state)

    def is_breakpoint(self, line_number: int) -> bool:
        doc = self.document()
        if doc is None:
            return False
        block = doc.findBlockByNumber(line_number)
        if block.isValid():
            data = block.userData()
            if data and isinstance(data, BreakpointUserData):
                return data.is_breakpoint
        return False

    def get_breakpoints(self) -> List[int]:
        breakpoints = []
        for i in range(self.blockCount() - 1):
            if self.is_breakpoint(i):
                breakpoints.append(i)
        return breakpoints
