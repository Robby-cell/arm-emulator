import os
from typing import Optional
import sys
import ctypes
from pathlib import Path
import json

from arm_emulator_rs import Emulator  # type: ignore : import exists
from keystone.keystone import KsError
from PyQt6.QtCore import QByteArray, QCoreApplication, QLocale, QSize, Qt, QTranslator
from PyQt6.QtGui import QAction, QActionGroup, QIcon, QPixmap
from PyQt6.QtWidgets import (
    QApplication,
    QFileDialog,
    QMainWindow,
    QMenu,
    QMenuBar,
    QMessageBox,
    QSplitter,
    QTabWidget,
    QToolBar,
    QVBoxLayout,
    QWidget,
)

from assembler import Assembler, arm_big_endian_assembler, arm_little_endian_assembler
from gui.resource import get_resource_path

from .controllers.debugger_controller import DebuggerController
from .language import get_languages_and_codes
from .screens.disassembly import DisassemblyScreen
from .screens.editor import EditorScreen
from .screens.memory_view import MemoryViewScreen
from .screens.tutorial_dialog import TutorialDialog
from .widgets.cpu_panel import CpuPanel

# from .widgets.title_bar import TitleBar
from .sample.starter_code import EXAMPLE_BLINK, EXAMPLE_FIBONACCI

RUN_ICON = "assets/icons/play.svg"
DEBUG_ICON = "assets/icons/bug.svg"
BUILD_ICON = "assets/icons/download.svg"
STOP_ICON = "assets/icons/square.svg"
STEP_ICON = "assets/icons/skip-forward.svg"
RESET_ICON = "assets/icons/refresh-cw.svg"


def get_icon_path() -> str:
    def inner() -> Path:
        base_path = Path("assets") / "icons"
        if sys.platform == "win32":
            return base_path / "favicon.ico"
        elif sys.platform == "darwin":
            return base_path / "favicon.icns"
        else:
            return base_path / "favicon.png"

    return str(inner())


def create_themed_icon(svg_path: str, color: str) -> QIcon:
    """
    Reads an SVG file, replaces its fill/stroke color, and returns a QIcon.
    This allows programmatic, theme-aware coloring of icons.
    """
    actual_path = get_resource_path(svg_path)

    with open(actual_path, "r") as f:
        svg_data = f.read()

    # Replace the placeholder 'currentColor' with the desired hex color
    themed_svg_data = svg_data.replace("currentColor", color)

    # Create a QPixmap from the modified SVG data
    pixmap = QPixmap()
    pixmap.loadFromData(QByteArray(themed_svg_data.encode("utf-8")))

    return QIcon(pixmap)


class MainWindow(QMainWindow):
    # Context manager for guaranteed exit with state saving and cleanup:
    def __enter__(self) -> "MainWindow":
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> None:
        self._leave()

    def _leave(self) -> None:
        self._save_config()

    def _save_config(self) -> None: ...

    def __init__(
        self,
        emulator: Emulator,
        assembler: Assembler,
        parent: Optional[QWidget] = None,
        flags: Qt.WindowType = Qt.WindowType.Window,
    ) -> None:
        super().__init__(parent=parent, flags=flags)

        if sys.platform == "win32":
            myappid = "ac.uk.qub.arm_emulator"  # Arbitrary string
            try:
                ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID(myappid)
            except AttributeError:
                pass

        self.setWindowIcon(QIcon(get_resource_path(get_icon_path())))

        # Setup title bar
        # self.setWindowFlags(Qt.WindowType.FramelessWindowHint)
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)

        self.setMinimumSize(800, 600)
        # self.title_bar = TitleBar("ARM Emulator", self)
        # self.title_bar.setLayoutDirection(Qt.LayoutDirection.LeftToRight)
        # self.layout.addWidget(self.title_bar)
        # self.setWindowTitle("ARM Emulator")

        self._translator = QTranslator()
        QCoreApplication.instance().installTranslator(self._translator)  # type: ignore : not None

        self._emulator = emulator
        self._assembler = assembler

        self._debugger_controller = DebuggerController(emulator=self._emulator)

        self._set_styling()

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

        self.tabs.currentChanged.connect(self._on_tab_changed)

        # Create the screens (the content for each tab)
        self._editor = EditorScreen()
        self._memory_view = MemoryViewScreen(emulator=self._emulator)
        self._disassembly = DisassemblyScreen(emulator=self._emulator)

        # Add the screens as tabs to the widget
        self.tabs.addTab(self._editor, None)
        self.tabs.addTab(self._memory_view, None)
        self.tabs.addTab(self._disassembly, None)

        self._cpu_panel = CpuPanel(emulator=self._emulator)

    def _init_layout(self) -> None:
        self._root_widget = QWidget(self)
        self._root_layout = QVBoxLayout(self._root_widget)

        self._root_layout.setSpacing(0)
        self._root_widget.setLayout(self._root_layout)

        # self._root_layout.addWidget(self.title_bar, 0)
        self._root_layout.addWidget(self._menu, 0)
        self._root_layout.addWidget(self.toolbar, 0)

        # Create a horizontal splitter
        self._main_splitter = QSplitter(Qt.Orientation.Horizontal)

        # Add the Tabs (Left side, index 0)
        self._main_splitter.addWidget(self.tabs)

        # Add the CPU Panel (Right side, index 1)
        self._main_splitter.addWidget(self._cpu_panel)

        # Set initial sizes (Give most space to the tabs)
        self._main_splitter.setSizes([800, 250])
        self._main_splitter.setCollapsible(1, True)  # Allow CPU panel to be hidden
        self._root_layout.addWidget(self._main_splitter, 1)

        self.setCentralWidget(self._root_widget)

    def _init_toolbar(self) -> None:
        self.toolbar = QToolBar(self.tr("Main Toolbar"))
        self.toolbar.setIconSize(
            QSize(20, 20)
        )  # Slightly smaller icon for balance with text
        self.toolbar.setMovable(False)
        self.toolbar.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
        # self.addToolBar(Qt.ToolBarArea.TopToolBarArea, self.toolbar)

        run_icon = create_themed_icon(RUN_ICON, "#4CAF50")  # Green
        debug_icon = create_themed_icon(DEBUG_ICON, "#FFC107")  # Amber/Yellow
        build_icon = create_themed_icon(BUILD_ICON, "#9C27B0")  # Purple for build
        stop_icon = create_themed_icon(STOP_ICON, "#F44336")  # Red
        step_icon = create_themed_icon(STEP_ICON, "#2196F3")  # Blue
        reset_icon = create_themed_icon(RESET_ICON, "#9E9E9E")  # Gray

        self.build_action = QAction(build_icon, self.tr("Load"), self)
        self.build_action.setShortcut("F7")

        self.run_action = QAction(run_icon, self.tr("Run"), self)
        self.run_action.setShortcut("F5")

        self.debug_action = QAction(debug_icon, self.tr("Debug"), self)

        self.stop_action = QAction(stop_icon, self.tr("Stop"), self)
        self.stop_action.setShortcut("Shift+F5")

        self.step_action = QAction(step_icon, self.tr("Step"), self)
        self.step_action.setShortcut("F10")

        self.reset_action = QAction(reset_icon, self.tr("Reset"), self)
        self.reset_action.setShortcut("Ctrl+R")

        self.toolbar.addAction(self.build_action)
        self.toolbar.addSeparator()
        self.toolbar.addAction(self.run_action)
        self.toolbar.addAction(self.debug_action)
        self.toolbar.addSeparator()
        self.toolbar.addAction(self.stop_action)
        self.toolbar.addAction(self.step_action)
        self.toolbar.addAction(self.reset_action)

        # Connect the toolbar actions
        self.build_action.triggered.connect(self._on_build_and_load)
        self.run_action.triggered.connect(self._on_run)
        self.debug_action.triggered.connect(self._on_debug)
        self.stop_action.triggered.connect(self._on_stop)
        self.step_action.triggered.connect(self._on_step)
        self.reset_action.triggered.connect(self._on_reset)

    def _init_debugger_connections(self) -> None:
        # Connect UI actions (buttons) to the controller's slots

        # Connect the editor's breakpoint toggle signal to the controller
        # This assumes your RichCodeEditor emits a signal like this.
        # self._editor.breakpoint_toggled.connect(self._debugger_controller.toggle_breakpoint)

        # Connect the controller's signals BACK to the UI's update methods
        self._debugger_controller.execution_started.connect(self._on_execution_started)
        self._debugger_controller.execution_stopped.connect(self._on_execution_stopped)
        self._debugger_controller.state_changed.connect(self._on_state_changed)
        self._debugger_controller.error_occurred.connect(self._on_error)
        self._debugger_controller.breakpoint_hit.connect(self._on_breakpoint_hit)
        self._debugger_controller.highlight_line.connect(
            self._editor._editor.set_execution_line
        )
        self._editor._editor.breakpoint_toggled.connect(
            self._debugger_controller.on_breakpoint_toggled
        )

    # Slots
    def _on_execution_started(self) -> None:
        """Update button states when the emulator is running."""
        self.run_action.setEnabled(False)
        self.debug_action.setEnabled(False)
        self.step_action.setEnabled(True)
        self.reset_action.setEnabled(True)
        self.stop_action.setEnabled(True)

    def _on_execution_stopped(self):
        """Update button states when the emulator is paused."""
        is_finished = self._emulator.is_finished()

        self.run_action.setEnabled(True)
        self.debug_action.setEnabled(True)

        self.step_action.setEnabled(not is_finished)

        self.reset_action.setEnabled(True)

    def _on_state_changed(self) -> None:
        """
        Master update function.
        Optimized: Only updates the CPU panel and the currently visible tab.
        """
        # Always update the side panel (it's always visible)
        self._cpu_panel.update_view()
        self._editor._peripherals.update_view()

        # Only update the currently active tab
        self._update_active_tab()

    def _on_tab_changed(self, index: int) -> None:
        """
        Called when the user clicks a different tab.
        We must update the view immediately because it might be stale.
        """
        self._update_active_tab()

    def _update_active_tab(self) -> None:
        """Finds the current tab and calls its update_view method."""
        current_widget = self.tabs.currentWidget()

        # Check if the widget has an update_view method (duck typing)
        if hasattr(current_widget, "update_view"):
            current_widget.update_view()  # type: ignore : not None

    def _on_breakpoint_hit(self, address: int) -> None:
        print(f"UI notified: Breakpoint hit at {hex(address)}")
        # Could add UI feedback here, like highlighting the line in the editor.

    def _on_error(self, message: str):
        """Show a critical error message to the user."""
        QMessageBox.critical(self, self.tr("Execution Error"), message)

    def _on_build_and_load(self) -> None:
        """Assembles and loads the binary, refreshing the memory view, but does NOT run."""
        if self._assemble_and_load():
            print("Program loaded. Ready to execute.")
            self._ready_for_debugging()
            # The controller.load_program emits state_changed, so UI updates automatically.

    def _on_run(self) -> None:
        """
        F5 Action:
        1. If paused (loaded & not halted): Resume execution.
        2. If finished or not loaded: Assemble, Load, and Restart execution.
        """
        # If the program is loaded, we check the state of the CPU
        if self._debugger_controller.is_program_loaded:
            # If the CPU is NOT halted (meaning we are just paused/stepped),
            # we simply resume the timer.
            if not self._emulator.is_halted():
                print("Resuming execution...")
                self._debugger_controller.run()
                return

            # If self._emulator.is_halted() is True (e.g. SVC 0 exit),
            # we fall through to the logic below to Re-Assemble and Restart.

        # Standard Build & Run flow (Initial run, or Restart after finish)
        if self._assemble_and_load():
            print("Starting execution...")
            self.step_action.setEnabled(True)
            self._debugger_controller.run()

    def _on_debug(self) -> None:
        """Assembles and loads the code, then prepares for debugging."""
        if self._assemble_and_load():
            print("Ready to debug. Press 'Step' to begin.")
            self._ready_for_debugging()
            # We don't call run(), leaving the UI ready for the user to step.

    def _on_stop(self) -> None:
        """Slot to stop execution. Delegates directly to the controller."""
        self._debugger_controller.stop()
        self._set_buttons_stopped()

    def _on_step(self) -> None:
        """Slot to perform a single step. Delegates directly to the controller."""
        self._debugger_controller.step()
        print(f"Step completed. New state:\n{self._emulator}")

    def _on_reset(self) -> None:
        """Slot to reset the emulator. Delegates directly to the controller."""
        self._debugger_controller.reset_emulator()
        self._ready_for_debugging()

    def _set_buttons_stopped(self) -> None:
        self.run_action.setEnabled(True)
        self.step_action.setEnabled(False)
        self.reset_action.setEnabled(True)
        self.stop_action.setEnabled(False)

    def _ready_for_debugging(self) -> None:
        self.run_action.setEnabled(True)
        self.build_action.setEnabled(True)
        self.debug_action.setEnabled(True)
        self.stop_action.setEnabled(True)
        self.step_action.setEnabled(True)
        self.reset_action.setEnabled(True)

    # Menu
    def _init_menu(self) -> None:
        self._menu: QMenuBar = QMenuBar()
        self._menu.setNativeMenuBar(False)
        self._menu.setContentsMargins(0, 0, 0, 0)

        self._file_menu: QMenu = self._menu.addMenu(self.tr("&File"))  # type: ignore : not None
        self._build_file_menu()

        self._build_menu: QMenu = self._menu.addMenu(self.tr("&Build"))  # type: ignore : not None
        self._build_build_menu_actions()

        self._options_menu: QMenu = self._menu.addMenu(self.tr("&Options"))  # type: ignore : not None
        self._build_options_menu()

        self._language_menu: QMenu = self._menu.addMenu(self.tr("&Language"))  # type: ignore : not None
        self._build_language_menu()

        self._help_menu: QMenu = self._menu.addMenu(self.tr("&Help"))  # type: ignore : not None
        self._build_help_menu()

    def _show_tutorial(self) -> None:
        """Spawns the tutorial dialog."""
        dialog = TutorialDialog(self)
        dialog.exec()  # Blocks interaction with main window until closed

    def _load_example_code(self, code: str) -> None:
        """Loads example code into the editor and warns about overwriting."""
        reply = QMessageBox.question(
            self,
            self.tr("Load Example"),
            self.tr("This will overwrite your current code. Continue?"),
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
        )

        if reply == QMessageBox.StandardButton.Yes:
            # Assumes your EditorScreen exposes the text edit widget
            # If your code uses self._editor._editor.setPlainText, use that:
            self._editor._editor.setPlainText(code)

            # Switch to the Editor tab so they can see it
            self.tabs.setCurrentIndex(0)

    def _build_help_menu(self) -> None:
        # 1. The Tutorial Guide
        self.tutorial_action = QAction(self.tr("Quick Start Guide"), self)
        self.tutorial_action.triggered.connect(self._show_tutorial)
        self._help_menu.addAction(self.tutorial_action)

        self._help_menu.addSeparator()

        # 2. Examples Sub-menu
        examples_menu: QMenu = self._help_menu.addMenu(self.tr("Load Example..."))  # type: ignore : not None

        ex_blink = QAction(self.tr("Blinking LED"), self)
        ex_blink.triggered.connect(lambda: self._load_example_code(EXAMPLE_BLINK))
        examples_menu.addAction(ex_blink)

        ex_fib = QAction(self.tr("Fibonacci Sequence"), self)
        ex_fib.triggered.connect(lambda: self._load_example_code(EXAMPLE_FIBONACCI))
        examples_menu.addAction(ex_fib)

    def _build_build_menu_actions(self) -> None:
        # Action for the menu
        self.build_action_menu = QAction(self.tr("Build and Load"), self)
        self.build_action_menu.triggered.connect(self._on_build_and_load)
        self._build_menu.addAction(self.build_action_menu)

    def _build_file_menu(self) -> None:
        self._load_file_action = QAction(self)
        self._load_file_action.setShortcut("Ctrl+O")
        self._load_file_action.triggered.connect(self._load_file_selected)
        self._file_menu.addAction(self._load_file_action)  # type: ignore : not None

        self._save_file_action = QAction(self)
        self._save_file_action.setShortcut("Ctrl+S")
        self._save_file_action.triggered.connect(self._save_config_as)
        self._file_menu.addAction(self._save_file_action)

    def _build_options_menu(self) -> None:
        # Endianness Submenu
        endian_menu: QMenu = self._options_menu.addMenu(self.tr("Endianness"))  # type: ignore : not None

        # Create an exclusive group (Radio button behavior)
        self._endian_group = QActionGroup(self)

        # Little Endian Action
        self._action_le = QAction(self.tr("Little Endian"), self)
        self._action_le.setCheckable(True)
        self._action_le.setChecked(True)  # Default
        self._action_le.triggered.connect(lambda: self._set_endianness(little=True))
        self._endian_group.addAction(self._action_le)
        endian_menu.addAction(self._action_le)

        # Big Endian Action
        self._action_be = QAction(self.tr("Big Endian"), self)
        self._action_be.setCheckable(True)
        self._action_be.triggered.connect(lambda: self._set_endianness(little=False))
        self._endian_group.addAction(self._action_be)
        endian_menu.addAction(self._action_be)

    def _build_language_menu(self) -> None:
        for lang, code in get_languages_and_codes(self):
            action = QAction(lang, self)
            action.triggered.connect(lambda checked, c=code: self.load_language(c))
            self._language_menu.addAction(action)

    def _set_endianness(self, little: bool) -> None:
        print(f"Switching to {'Little' if little else 'Big'} Endian...")

        self._debugger_controller.unload_program()

        # Update Assembler
        old_symbols = self._assembler.symbols
        if little:
            self._assembler = arm_little_endian_assembler()
        else:
            self._assembler = arm_big_endian_assembler()
        self._assembler.symbols = old_symbols

        # Update Emulator
        if little:
            self._emulator.use_little_endian()
        else:
            self._emulator.use_big_endian()

        # Update Disassembly Screen
        self._disassembly.set_endianness(little)

        # Trigger a refresh of the current view
        self._update_active_tab()

        if hasattr(self, "_update_button_states"):
            self._update_button_states()

    def _update_button_states(self) -> None:
        is_loaded = self._debugger_controller.is_program_loaded  # This is now False

        # Run/Debug should be enabled (so user can reload/rebuild)
        self.run_action.setEnabled(True)
        self.debug_action.setEnabled(True)

        # Step should be DISABLED (No code to step through)
        self.step_action.setEnabled(False)  # is_loaded is False

        # Reset should be DISABLED (Nothing to reset)
        self.reset_action.setEnabled(is_loaded)

    def load_language(self, lang_code: str) -> None:
        app = QCoreApplication.instance()
        if app is None:
            return

        app.removeTranslator(self._translator)  # type: ignore : not None

        relative_path = os.path.join("assets", "translations", f"app_{lang_code}.qm")
        file_path = get_resource_path(relative_path)

        print(f"Attempting to load translation: {file_path}")

        self._translator = QTranslator()
        # Load the new .qm file
        if self._translator.load(file_path):
            app.installTranslator(self._translator)
            print(f"Successfully loaded language: {lang_code}")
        else:
            print(f"FAILED to load language file: {file_path}")
            # Optional: Fallback to English or show an error
            return

        locale = QLocale(lang_code)
        direction = locale.textDirection()
        QApplication.setLayoutDirection(direction)

        self.retranslateUI()

    def _load_file_selected(self) -> None:
        dialog = QFileDialog(self)
        dialog.setFileMode(QFileDialog.FileMode.ExistingFile)
        # Allow both our custom config and standard assembly files
        dialog.setNameFilter(
            self.tr(
                "ARM Emulator Config (*.armcfg);;Assembly Files (*.s *.asm);;All Files (*)"
            )
        )
        if dialog.exec():
            file_path = dialog.selectedFiles()[0]
            self._load_file(file_path)

    def _save_config_as(self) -> None:
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            self.tr("Save Configuration"),
            "",
            self.tr("ARM Emulator Config (*.armcfg);;All Files (*)"),
        )

        if not file_path:
            return

        # Ensure extension
        if not file_path.endswith(".armcfg"):
            file_path += ".armcfg"

        # Build the configuration dictionary
        config = {
            "version": "1.0",
            "code": self._editor.get_code(),
            "breakpoints": self._editor._editor.get_breakpoints(),
            "peripherals": self._editor._peripherals.get_config(),
        }

        try:
            with open(file_path, "w", encoding="utf-8") as f:
                json.dump(config, f, indent=4)
            print(f"Configuration successfully saved to {file_path}")
            QMessageBox.information(
                self, self.tr("Success"), self.tr("Configuration saved successfully.")
            )
        except Exception as e:
            QMessageBox.critical(
                self,
                self.tr("Save Error"),
                f"{self.tr('Failed to save configuration:')}\n{e}",
            )

    def _load_file(self, file_path: str) -> None:
        print(f"Loading file: {file_path}")

        try:
            with open(file_path, "r", encoding="utf-8") as f:
                content = f.read()

            try:
                # 1. Try parsing as our custom JSON Configuration
                config = json.loads(content)

                # Load Code
                if "code" in config:
                    self._editor._editor.setPlainText(config["code"])

                # Load Peripherals
                if "peripherals" in config:
                    self._editor._peripherals.load_from_config(config["peripherals"])

                # Load Breakpoints (Must be done AFTER code is loaded so lines exist)
                if "breakpoints" in config:
                    self._editor._editor.set_breakpoints(config["breakpoints"])

                print("Successfully loaded .armcfg workspace.")

            except json.JSONDecodeError:
                # 2. Fallback: If it's not JSON, it's just raw assembly code
                print("File is not a JSON config. Loading as plain assembly text.")
                self._editor._editor.setPlainText(content)
                # Clear peripherals and breakpoints for a clean slate
                self._editor._editor.clear_breakpoints()
                self._editor._peripherals.clear_peripherals()

        except Exception as e:
            QMessageBox.critical(
                self, self.tr("Load Error"), f"{self.tr('Failed to load file:')}\n{e}"
            )

    def _assemble_and_load(self) -> bool:
        """
        Helper: Gets code, Assembles it, and Loads it into the Emulator.
        Returns True if successful, False otherwise.
        """
        self._debugger_controller.stop()
        code = self._editor.get_code()
        self._assembler.symbols.clear()

        panel = self._editor._peripherals
        peripherals_map = panel.get_defined_symbols()

        for name, addr in peripherals_map.items():
            self._assembler.add_symbol(name, addr)
            print(f"Registered peripheral symbol: {name} -> {hex(addr)}")

        try:
            # Assemble
            assembled = self._assembler.assemble(code)

            # Check for Keystone errors or empty output
            if assembled.text is None or len(assembled.text) == 0:
                # If keystone returns None or empty bytes without raising exception
                QMessageBox.warning(
                    self,
                    self.tr("Assembly Warning"),
                    self.tr("Assembly produced no code."),
                )
                return False

        except KsError as e:
            QMessageBox.critical(
                self,
                self.tr("Assembler Error"),
                "{}\n{}".format(self.tr("Failed to assemble code:"), e),
            )
            return False
        except Exception as e:
            QMessageBox.critical(
                self,
                self.tr("Assembler Error"),
                "{}\n{}".format(self.tr("An unexpected error occurred:"), e),
            )
            return False

        # Load into Emulator via Controller
        print(f"Assembly successful. Text size: {len(assembled.text)} bytes.")

        peripherals = panel.get_peripherals()
        self._debugger_controller.set_peripherals(peripherals)

        self._debugger_controller.load_program(assembled)

        return True

    def _set_styling(self) -> None:
        self.setStyleSheet("""
            QWidget {
                background-color: #1e1e1e;
                color: #f0f0f0;
            }
            QMainWindow {
                background-color: #2b2b2b;
                color: white;
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

    def retranslateUI(self) -> None:
        """Updates all user-visible text in the application."""
        # Main Window
        self.setWindowTitle(self.tr("ARM Emulator"))

        # Menu Bar
        self._file_menu.setTitle(self.tr("&File"))
        self._build_menu.setTitle(self.tr("&Build"))
        self._options_menu.setTitle(self.tr("&Options"))
        self._language_menu.setTitle(self.tr("&Language"))

        self._load_file_action.setText(self.tr("Load File"))
        self._save_file_action.setText(self.tr("Save Config As..."))
        self.build_action_menu.setText(self.tr("Build and Load"))

        self._help_menu.setTitle(self.tr("&Help"))
        self.tutorial_action.setText(self.tr("Quick Start Guide"))

        # Toolbar
        self.toolbar.setWindowTitle(self.tr("Main Toolbar"))
        self.build_action.setText(self.tr("Build"))
        self.run_action.setText(self.tr("Run"))
        self.debug_action.setText(self.tr("Debug"))
        self.stop_action.setText(self.tr("Stop"))
        self.step_action.setText(self.tr("Step"))
        self.reset_action.setText(self.tr("Reset"))

        self.build_action.setToolTip(self.tr("Assemble and Load (F7)"))
        self.run_action.setToolTip(self.tr("Run (F5)"))
        self.debug_action.setToolTip(self.tr("Prepare for Debugging"))
        self.stop_action.setToolTip(self.tr("Stop Execution (Shift+F5)"))
        self.step_action.setToolTip(self.tr("Step Instruction (F10)"))
        self.reset_action.setToolTip(self.tr("Reset Emulator (Ctrl+R)"))

        # Tabs
        self.tabs.setTabText(0, self.tr("Editor"))
        self.tabs.setTabText(1, self.tr("Memory View"))
        self.tabs.setTabText(2, self.tr("Disassembly"))

        self._editor.retranslateUi()
        self._memory_view.retranslateUi()
        self._cpu_panel.retranslateUi()
        self._disassembly.retranslateUi()
