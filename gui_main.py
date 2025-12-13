from sys import argv, exit

from arm_emulator_rs import emulator  # type: ignore : import exists
from PyQt6.QtWidgets import QApplication

from assembler.assembler import arm_little_endian_assembler
from gui.main_window import MainWindow


def main():
    app = QApplication(argv)
    with MainWindow(
        emulator=emulator.Emulator(
            code_size=0,
            sram_size=0,
            external_size=0,
        ),
        assembler=arm_little_endian_assembler(),
    ) as window:
        window.showMaximized()
        exit_code: int = app.exec()
        exit(exit_code)


if __name__ == "__main__":
    main()
