from PyQt6.QtWidgets import QApplication
from gui.main_window import MainWindow
from sys import argv, exit

from arm_emulator_rs import emulator, memory  # type: ignore : import exists
from assembler.assembler import arm_little_endian_assembler


DEFAULT_RAM_SIZE: int = 0x20000


def main():
    app = QApplication(argv)
    with MainWindow(
        emulator=emulator.Emulator(memory.RamSize(DEFAULT_RAM_SIZE)),
        assembler=arm_little_endian_assembler(),
    ) as window:
        window.showMaximized()
        exit_code: int = app.exec()
        exit(exit_code)


if __name__ == "__main__":
    main()
