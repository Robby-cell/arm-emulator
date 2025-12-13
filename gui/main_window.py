from typing import Optional

from arm_emulator_rs import emulator  # type: ignore : import exists
from keystone.keystone import KsError
from PyQt6.QtCore import QByteArray, QCoreApplication, QSize, Qt, QTranslator
from PyQt6.QtGui import QAction, QIcon, QPixmap
from PyQt6.QtWidgets import (
    QFileDialog,
    QMainWindow,
    QMenu,
    QMessageBox,
    QTabWidget,
    QToolBar,
    QWidget,
)

from assembler import AssembledOutput, Assembler

from .controllers.debugger_controller import DebuggerController
from .screens.disassembly import DisassemblyScreen
from .screens.editor import EditorScreen
from .screens.memory_view import MemoryViewScreen

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

    def _leave(self) -> None:
        self._save_config()

    def _save_config(self) -> None: ...

    # Actual class:
    def __init__(
        self,
        emulator: emulator.Emulator,
        assembler: Assembler,
        parent: Optional[QWidget] = None,
        flags: Qt.WindowType = Qt.WindowType.Window,
    ) -> None:
        super().__init__(parent=parent, flags=flags)
        self.setWindowTitle("ARM Emulator")

        self._translator = QTranslator()
        QCoreApplication.instance().installTranslator(self._translator)  # type: ignore : not None

        self._emulator = emulator
        self._assembler = assembler

        self._debugger_controller = DebuggerController(emulator=self._emulator)

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

        self._init_widgets()
        self._init_menu()
        self._init_toolbar()
        self._init_layout()

        self._init_debugger_connections()
        self._on_execution_stopped()  # Set initial button states

        self.retranslateUI()

    def _init_widgets(self) -> None:
        # Create the QTabWidget
        self.tabs = QTabWidget()

        # Create the screens (the content for each tab)
        self._editor = EditorScreen()
        self._memory_view = MemoryViewScreen(emulator=self._emulator)
        self._disassembly = DisassemblyScreen()

        # Add the screens as tabs to the widget
        self.tabs.addTab(self._editor, None)
        self.tabs.addTab(self._memory_view, None)
        self.tabs.addTab(self._disassembly, None)

    def _init_layout(self) -> None:
        self.setCentralWidget(self.tabs)

    def _init_toolbar(self) -> None:
        self.toolbar = QToolBar(self.tr("Main Toolbar"))
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

        self.run_action = QAction(run_icon, self.tr("Run"), self)
        self.debug_action = QAction(debug_icon, self.tr("Debug"), self)
        self.stop_action = QAction(stop_icon, self.tr("Stop"), self)
        self.step_action = QAction(step_icon, self.tr("Step"), self)
        self.reset_action = QAction(reset_icon, self.tr("Reset"), self)

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

    def _init_debugger_connections(self):
        # Connect UI actions (buttons) to the controller's slots
        self.run_action.triggered.connect(self._debugger_controller.run)
        # Run with breakpoints enabled
        self.debug_action.triggered.connect(self._debugger_controller.run)
        self.stop_action.triggered.connect(self._debugger_controller.stop)
        self.step_action.triggered.connect(self._debugger_controller.step)
        self.reset_action.triggered.connect(self._debugger_controller.reset_emulator)

        # Connect the editor's breakpoint toggle signal to the controller
        # This assumes your RichCodeEditor emits a signal like this.
        # self._editor.breakpoint_toggled.connect(self._debugger_controller.toggle_breakpoint)

        # Connect the controller's signals BACK to the UI's update methods
        self._debugger_controller.execution_started.connect(self._on_execution_started)
        self._debugger_controller.execution_stopped.connect(self._on_execution_stopped)
        self._debugger_controller.state_changed.connect(self._on_state_changed)
        self._debugger_controller.error_occurred.connect(self._on_error)
        self._debugger_controller.breakpoint_hit.connect(self._on_breakpoint_hit)

    # Slots
    def _on_execution_started(self):
        """Update button states when the emulator is running."""
        self.run_action.setEnabled(False)
        self.debug_action.setEnabled(False)
        self.step_action.setEnabled(True)
        self.reset_action.setEnabled(True)
        self.stop_action.setEnabled(True)

    def _on_execution_stopped(self):
        """Update button states when the emulator is paused."""
        self.run_action.setEnabled(True)
        self.debug_action.setEnabled(True)
        self.step_action.setEnabled(True)
        self.reset_action.setEnabled(True)
        self.stop_action.setEnabled(False)

    def _on_state_changed(self):
        """Master update function for all views."""
        self._memory_view.update_view()
        # self._register_view.update_view() # Add when register view added
        # self._disassembly_view.update_view() # Add when disassembly view added

    def _on_breakpoint_hit(self, address: int):
        print(f"UI notified: Breakpoint hit at {hex(address)}")
        # Could add UI feedback here, like highlighting the line in the editor.

    def _on_error(self, message: str):
        """Show a critical error message to the user."""
        QMessageBox.critical(self, self.tr("Execution Error"), message)

    def _on_run(self) -> None:
        """Assembles, loads, and then runs the code from the editor."""
        if self._assemble_and_load():
            self._debugger_controller.run()

    def _on_debug(self) -> None:
        """Assembles and loads the code, then prepares for debugging."""
        if self._assemble_and_load():
            print("Ready to debug. Press 'Step' to begin.")
            # We don't call run(), leaving the UI ready for the user to step.

    def _on_stop(self) -> None:
        """Slot to stop execution. Delegates directly to the controller."""
        self._debugger_controller.stop()

    def _on_step(self) -> None:
        """Slot to perform a single step. Delegates directly to the controller."""
        self._debugger_controller.step()
        print(f"Step completed. New state:\n{self._emulator}")

    def _on_reset(self) -> None:
        """Slot to reset the emulator. Delegates directly to the controller."""
        self._debugger_controller.reset_emulator()

    # Menu
    def _init_menu(self) -> None:
        menu_bar = self.menuBar()
        if menu_bar is None:
            return

        self._file_menu: QMenu = menu_bar.addMenu(self.tr("&File"))  # type: ignore : not None
        self._build_file_menu()

        self._language_menu: QMenu = menu_bar.addMenu(self.tr("&Language"))  # type: ignore : not None
        self._build_language_menu()

    def _build_file_menu(self) -> None:
        self._load_file_action = QAction(self)
        self._load_file_action.triggered.connect(self._load_file_selected)
        self._file_menu.addAction(self._load_file_action)  # type: ignore : not None

    def _build_language_menu(self):
        # English Action
        english_action = QAction(self.tr("English"), self)
        english_action.triggered.connect(lambda: self.load_language("en"))
        self._language_menu.addAction(english_action)

        # Russian Action
        russian_action = QAction(self.tr("Русский"), self)
        russian_action.triggered.connect(lambda: self.load_language("ru"))
        self._language_menu.addAction(russian_action)

        # Polish Action
        polish_action = QAction(self.tr("Polski"), self)
        polish_action.triggered.connect(lambda: self.load_language("pl"))
        self._language_menu.addAction(polish_action)

        # Spanish Action
        spanish_action = QAction(self.tr("Español"), self)
        spanish_action.triggered.connect(lambda: self.load_language("es"))
        self._language_menu.addAction(spanish_action)

    def load_language(self, lang_code: str):
        app = QCoreApplication.instance()
        if app is None:
            return

        app.removeTranslator(self._translator)  # type: ignore : not None

        self._translator = QTranslator()
        # Load the new .qm file
        if self._translator.load(f"assets/translations/app_{lang_code}.qm"):
            app.installTranslator(self._translator)

        self.retranslateUI()

    def retranslateUI(self):
        """Updates all user-visible text in the application."""
        # Main Window
        self.setWindowTitle(self.tr("ARM Simulator"))

        # Menu Bar
        self._file_menu.setTitle(self.tr("&File"))
        self._language_menu.setTitle(self.tr("&Language"))
        self._load_file_action.setText(self.tr("Load File"))

        # Toolbar
        self.toolbar.setWindowTitle(self.tr("Main Toolbar"))
        self.run_action.setText(self.tr("Run"))
        self.debug_action.setText(self.tr("Debug"))
        self.stop_action.setText(self.tr("Stop"))
        self.step_action.setText(self.tr("Step"))
        self.reset_action.setText(self.tr("Reset"))

        # Tabs
        self.tabs.setTabText(0, self.tr("Editor"))
        self.tabs.setTabText(1, self.tr("Memory View"))
        self.tabs.setTabText(2, self.tr("Disassembly"))

    def _load_file_selected(self) -> None:
        dialog = QFileDialog(self)
        dialog.setFileMode(QFileDialog.FileMode.ExistingFile)
        if dialog.exec():
            file_path = dialog.selectedFiles()[0]
            self._load_file(file_path)

    def _load_file(self, file_path: str) -> None:
        print(f"Loading file: {file_path}")

    def _assemble_and_load(self) -> bool:
        """
        Handles the core logic of assembling and loading.
        Returns True on success, False on failure.
        """
        code = self._editor.get_code()
        print("Assembling...")

        try:
            assembled_output: AssembledOutput = self._assembler.assemble(code)

            if assembled_output.text is None:
                # Handle case where assembly produces no output (e.g., empty string)
                QMessageBox.warning(
                    self,
                    self.tr("Assembly Warning"),
                    self.tr("Assembly produced no code."),
                )
                return False
        except KsError as e:
            # Catch specific Keystone assembler errors and display them
            # TODO: Decouple Keystone from the assembler. Implementation detail
            QMessageBox.critical(
                self,
                self.tr("Assembler Error"),
                f"{self.tr('Failed to assemble code:')}\n{e}",
            )
            return False

        print(
            f"Assembly successful. .text section is {len(assembled_output.text)} bytes."
        )

        # Pass the entire successful output object to the controller for loading
        self._debugger_controller.load_program(assembled_output)

        return True
