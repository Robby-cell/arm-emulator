from PyQt6.QtWidgets import (
    QMainWindow,
    QWidget,
    QMenu,
    QFileDialog,
    QToolBar,  # Import QToolBar
)
from PyQt6.QtGui import QAction, QIcon, QPixmap
from PyQt6.QtWidgets import QTabWidget
from PyQt6.QtCore import Qt, QSize, QByteArray

from typing import Optional

from screens.editor import EditorScreen
from screens.memory_view import MemoryViewScreen
from screens.disassembly import DisassemblyScreen

RUN_ICON = "assets/icons/play.svg"
DEBUG_ICON = "assets/icons/bug.svg"
STOP_ICON = "assets/icons/square.svg"
STEP_ICON = "assets/icons/skip-forward.svg"
RESET_ICON = "assets/icons/refresh-cw.svg"


def create_themed_icon(svg_path: str, color: str) -> QIcon:
    """
    Reads an SVG file, replaces its fill/stroke color, and returns a QIcon.
    This allows programmatic, theme-aware coloring of icons.
    """
    with open(svg_path, "r") as f:
        svg_data = f.read()

    # Replace the placeholder 'currentColor' with the desired hex color
    themed_svg_data = svg_data.replace("currentColor", color)

    # Create a QPixmap from the modified SVG data
    pixmap = QPixmap()
    pixmap.loadFromData(QByteArray(themed_svg_data.encode("utf-8")))

    return QIcon(pixmap)


class MainWindow(QMainWindow):
    # Context manager for guaranteed exit code:
    def __enter__(self) -> "MainWindow":
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> None:
        self._leave()

    def _leave(self):
        self._save_config()

    def _save_config(self): ...

    # Actual class:
    def __init__(
        self,
        parent: Optional[QWidget] = None,
        flags: Qt.WindowType = Qt.WindowType.Window,
    ):
        super().__init__(parent=parent, flags=flags)
        self.setWindowTitle("ARM Emulator")

        self._init_widgets()
        self._init_menu()
        self._init_toolbar()
        self._init_layout()

        self.setStyleSheet("""
            QMainWindow {
                background-color: #2b2b2b;
            }
            QToolBar {
                background-color: #3c3f41;
                border: none;
                padding: 5px;
            }
            QToolButton {
                padding: 8px;
                border-radius: 4px;
                background-color: #3c3f41;
                color: #dddddd; /* This now controls BOTH text and icon color */
            }
            QToolButton:hover {
                background-color: #4b5052;
            }
            QToolButton:pressed {
                background-color: #525659;
            }

            QTabWidget::pane {
                border: none;
            }
            QTabBar::tab {
                background: #3c3f41;
                color: #bbbbbb;
                padding: 10px 20px;
                border-top-left-radius: 4px;
                border-top-right-radius: 4px;
            }
            QTabBar::tab:hover {
                background: #4b5052;
            }
            QTabBar::tab:selected {
                background: #444444; /* Match the background of the content area */
                color: white;
            }
        """)

    def _init_widgets(self):
        # 1. Create the QTabWidget
        self.tabs = QTabWidget()

        # 2. Create the screens (the content for each tab)
        self._editor = EditorScreen()
        self._memory_view = MemoryViewScreen()
        self._disassembly = DisassemblyScreen()

        # 3. Add the screens as tabs to the widget
        self.tabs.addTab(self._editor, "Editor")
        self.tabs.addTab(self._memory_view, "Memory View")
        self.tabs.addTab(self._disassembly, "Disassembly")

    def _init_layout(self):
        self.setCentralWidget(self.tabs)

    def _init_toolbar(self):
        self.toolbar = QToolBar("Main Toolbar")
        self.toolbar.setIconSize(
            QSize(20, 20)
        )  # Slightly smaller icon for balance with text
        self.toolbar.setMovable(False)
        self.toolbar.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
        self.addToolBar(Qt.ToolBarArea.TopToolBarArea, self.toolbar)

        run_icon = create_themed_icon(RUN_ICON, "#4CAF50")  # Green
        debug_icon = create_themed_icon(DEBUG_ICON, "#FFC107")  # Amber/Yellow
        stop_icon = create_themed_icon(STOP_ICON, "#F44336")  # Red
        step_icon = create_themed_icon(STEP_ICON, "#2196F3")  # Blue
        reset_icon = create_themed_icon(RESET_ICON, "#9E9E9E")  # Gray

        self.run_action = QAction(run_icon, "Run", self)
        self.debug_action = QAction(debug_icon, "Debug", self)
        self.stop_action = QAction(stop_icon, "Stop", self)
        self.step_action = QAction(step_icon, "Step", self)
        self.reset_action = QAction(reset_icon, "Reset", self)

        self.toolbar.addAction(self.run_action)
        self.toolbar.addAction(self.debug_action)
        self.toolbar.addSeparator()
        self.toolbar.addAction(self.stop_action)
        self.toolbar.addAction(self.step_action)
        self.toolbar.addAction(self.reset_action)

        # Connect the toolbar actions
        self.run_action.triggered.connect(self._on_run)
        self.debug_action.triggered.connect(self._on_debug)
        self.stop_action.triggered.connect(self._on_stop)
        self.step_action.triggered.connect(self._on_step)
        self.reset_action.triggered.connect(self._on_reset)

    # Slots
    def _on_run(self):
        code = self._editor.get_code()
        print(f"--- Running Code ---\n{code}\n--------------------")

    def _on_debug(self):
        code = self._editor.get_code()
        print(f"--- Launching Debugger ---\n{code}\n--------------------")

    def _on_stop(self):
        print("Execution stopped.")

    def _on_step(self):
        print("Stepping to next instruction.")

    def _on_reset(self):
        print("Simulator reset.")

    # Menu
    def _init_menu(self):
        menu_bar = self.menuBar()
        self._build_file_menu(menu_bar.addMenu("&File"))  # type: ignore

    def _build_file_menu(self, file_menu: QMenu):
        load_file_action = QAction("Load File", self)
        load_file_action.triggered.connect(self._load_file_selected)
        file_menu.addAction(load_file_action)

    def _load_file_selected(self):
        dialog = QFileDialog(self)
        dialog.setFileMode(QFileDialog.FileMode.ExistingFile)
        if dialog.exec():
            file_path = dialog.selectedFiles()[0]
            self._load_file(file_path)

    def _load_file(self, file_path: str):
        print(f"Loading file: {file_path}")
