from PyQt6.QtCore import Qt, QRect
from PyQt6.QtGui import QPainter, QColor, QFont, QTextBlockUserData, QTextCursor
from PyQt6.QtWidgets import QWidget, QPlainTextEdit
from typing import Optional


class BreakpointUserData(QTextBlockUserData):
    """Holds the breakpoint state for a QTextBlock."""

    def __init__(self, is_breakpoint: bool = False):
        super().__init__()
        self.is_breakpoint = is_breakpoint


class LineNumberArea(QWidget):
    """The gutter widget, displaying line numbers and breakpoints."""

    _background_color: QColor
    _text_color: QColor
    _breakpoint_color: QColor

    def __init__(self, editor: "CodeEditor"):
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
    def background_color(self, color: QColor):
        if color is None:
            return
        self._background_color = color
        self.update()

    @property
    def text_color(self) -> QColor:
        return self._text_color

    @text_color.setter
    def text_color(self, color: QColor):
        self._text_color = color
        self.update()

    @property
    def breakpoint_color(self) -> QColor:
        return self._breakpoint_color

    @breakpoint_color.setter
    def breakpoint_color(self, color: QColor):
        self._breakpoint_color = color
        self.update()

    def paintEvent(self, a0):
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

    def mousePressEvent(self, a0):
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

    def __init__(
        self,
        line_number_area: Optional[LineNumberArea] = None,
        parent: Optional[QWidget] = None,
    ):
        super().__init__(parent)
        self._line_number_area = (
            line_number_area if line_number_area is not None else LineNumberArea(self)
        )

        self.setFont(QFont("monospace", 12))
        self.setLineWrapMode(QPlainTextEdit.LineWrapMode.NoWrap)

        self.setViewportMargins(self.GUTTER_FIXED_WIDTH, 0, 0, 0)

        # Connect signals
        self.updateRequest.connect(self._update_gutter)
        self.textChanged.connect(self._enforce_guard_line)
        self.cursorPositionChanged.connect(self._prevent_cursor_on_last_line)

        # Start with the guard line already present
        # If we don't do this, the breakpoint circle gets stretched weirdly when inserting a new line, when
        # it is set on the last line. Do this to avoid that.
        self.setPlainText("\n")

    def _update_gutter(self, rect: QRect, dy: int):
        if dy:
            self._line_number_area.scroll(0, dy)
        else:
            self._line_number_area.update(
                0, rect.y(), self._line_number_area.width(), rect.height()
            )

    def _enforce_guard_line(self):
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

    def _prevent_cursor_on_last_line(self):
        """Stops the user from selecting or writing on the guard line."""
        cursor = self.textCursor()
        if cursor.blockNumber() == self.blockCount() - 1:
            self.blockSignals(True)
            cursor.movePosition(QTextCursor.MoveOperation.PreviousBlock)
            cursor.movePosition(QTextCursor.MoveOperation.EndOfLine)
            self.setTextCursor(cursor)
            self.blockSignals(False)

    def resizeEvent(self, e):
        super().resizeEvent(e)
        cr = self.contentsRect()
        self._line_number_area.setGeometry(
            QRect(cr.left(), cr.top(), self.GUTTER_FIXED_WIDTH, cr.height())
        )

    def toggle_breakpoint(self, line_number: int):
        if line_number >= self.blockCount() - 1:
            return

        doc = self.document()
        if doc is None:
            return
        block = doc.findBlockByNumber(line_number)
        if not block.isValid():
            return

        data = block.userData()
        if not data:
            data = BreakpointUserData()

        data.is_breakpoint = not data.is_breakpoint  # type: ignore : This is our custom data type.
        block.setUserData(data)
        self._line_number_area.update()

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

    def get_breakpoints(self) -> list[int]:
        breakpoints = []
        for i in range(self.blockCount() - 1):
            if self.is_breakpoint(i):
                breakpoints.append(i)
        return breakpoints
