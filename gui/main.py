from PyQt6.QtWidgets import QApplication
from main_window import MainWindow
from sys import argv, exit


def main():
    app = QApplication(argv)
    with MainWindow() as window:
        window.showMaximized()
        exit_code: int = app.exec()
        exit(exit_code)


if __name__ == "__main__":
    main()
