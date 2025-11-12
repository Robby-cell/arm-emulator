from PyQt6.QtWidgets import QApplication
from main_window import MainWindow
from sys import argv, exit

from arm_emulator_rs import emulator  # type: ignore : import exists


def main():
    app = QApplication(argv)
    with MainWindow(emulator=emulator.Emulator(0xFFFFFFFF)) as window:
        window.showMaximized()
        exit_code: int = app.exec()
        exit(exit_code)


if __name__ == "__main__":
    main()
