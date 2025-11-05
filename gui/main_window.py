from PyQt6.QtWidgets import (
    QMainWindow,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QToolBar,
    QMenuBar,
    QMenu,
    QFileDialog,
)
from PyQt6.QtGui import (
    QAction,
)
from PyQt6.QtCore import Qt
from typing import Optional

from widgets.tab import Tab

from screens.editor import EditorScreen
from screens.memory_view import MemoryViewScreen
from screens.disassembly import DisassemblyScreen

class MainWindow(QMainWindow):
    def __init__(self, parent: Optional[QWidget]=None, flags: Qt.WindowType=Qt.WindowType.Window):
        super().__init__(parent=parent, flags=flags)

        self._menu_bar = QMenuBar(parent=self)
        self.setMenuBar(self._menu_bar)
        self._build_menu_bar()

        self._editor_tab = Tab(text="Editor", parent=self)
        self._memory_view_tab = Tab(text="Memory View", parent=self)
        self._disassembly_tab = Tab(text="Disassembly", parent=self)

        self._editor = EditorScreen(parent=self)
        self._memory_view = MemoryViewScreen(parent=self)
        self._disassembly = DisassemblyScreen(parent=self)

        self._editor.show()
        self._memory_view.hide()
        self._disassembly.hide()

        self._editor_tab.clicked.connect(self._show_editor)
        self._memory_view_tab.clicked.connect(self._show_memory_view)
        self._disassembly_tab.clicked.connect(self._show_disassembly)

        internal_base = QWidget(parent=self)
        self.setCentralWidget(internal_base)
        self._base_layout = QVBoxLayout(internal_base)
        self._base_layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        # self._base_layout.addStretch()
        internal_base.setLayout(self._base_layout)

        tabs = QWidget(parent=internal_base)
        self._tabs_layout = QHBoxLayout(tabs)
        tabs.setLayout(self._tabs_layout)
        self._base_layout.addWidget(tabs)

        main_content = QWidget(parent=internal_base)
        self._main_content_layout = QVBoxLayout(main_content)
        main_content.setLayout(self._main_content_layout)
        self._base_layout.addWidget(main_content)

        self.setupUI()

    def setupUI(self):
        self.setWindowTitle("ARM Simulator")

        for tab in (self._editor_tab, self._memory_view_tab, self._disassembly_tab):
            self._tabs_layout.addWidget(tab)

        for screen in (self._editor, self._memory_view, self._disassembly):
            self._main_content_layout.addWidget(screen)

    def _show_editor(self):
        self._editor.show(); self._memory_view.hide(); self._disassembly.hide()

    def _show_memory_view(self):
        self._memory_view.show(); self._editor.hide(); self._disassembly.hide()

    def _show_disassembly(self):
        self._disassembly.show(); self._editor.hide(); self._memory_view.hide()

    # Menu Bar:
    def _build_menu_bar(self):
        # Linter sees this can be None. We know better.
        file_menu: QMenu = self._menu_bar.addMenu("&File")  # type: ignore

        self._build_file_menu(file_menu)

    def _build_file_menu(self, file_menu: QMenu):
        load_file_action = QAction("Load File", self)
        load_file_action.triggered.connect(self._load_file_selected)
        file_menu.addAction(load_file_action)

    # Actions:
    def _load_file_selected(self):
        dialog = QFileDialog(self)
        dialog.setFileMode(QFileDialog.FileMode.ExistingFile)
        if dialog.exec():
            file_path = dialog.selectedFiles()[0]
            self._load_file(file_path)

    def _load_file(self, file_path: str):
        print(file_path)
