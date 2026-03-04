import os
import sys


def get_resource_path(relative_path: str) -> str:
    """Get the absolute path to a resource, works for dev and for PyInstaller"""
    if hasattr(sys, "_MEIPASS"):
        # PyInstaller creates a temp folder and stores path in _MEIPASS
        base_path = sys._MEIPASS  # type: ignore
    else:
        # We are running normally from Python
        base_path = os.path.abspath(os.getcwd())

    # Return the safely joined path
    return os.path.join(base_path, relative_path)
