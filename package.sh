#!/usr/bin/env bash

set -e

function _source_venv() {
    # Since bash CAN be run on windows, and we will use it in CI/CD pipelines.
    # Windows, for whatever reason, puts it in `Scripts` instead of `bin`.
    NORMAL_VENV=".venv/bin/activate"
    WINDOWS_VENV=".venv/Scripts/activate"
    if test -f $NORMAL_VENV; then
        echo "Using normal venv"
        source ./$NORMAL_VENV
    else
        echo "Using windows venv"
        ./$WINDOWS_VENV
    fi
}

function _ensure_venv() {
    if [ ! -d ".venv" ]; then
        python3 -m venv .venv
    fi
}

function _ensure_pyinstaller() {
    if ! command -v pyinstaller >/dev/null 2>&1; then
        _ensure_venv
        _source_venv
        echo "pip install pyinstaller PyQt6"
        pip install pyinstaller PyQt6
    fi
}

function main() {
    echo "Building standalone executable using PyInstaller."

    echo "Ensuring environment is ready..."
    _ensure_pyinstaller
    echo "Environment ready."

    echo "Creating spec file..."
    pyi-makespec gui/main.py --name emulator

    echo "Building executable..."
    # use pyinstaller to create a standalone binary.
    # --onefile: create a single executable file, no additional directories needed.
    # --name emulator: name the output executable "emulator"
    # --additional-hooks-dir hooks: include additional hooks from the "hooks" directory.
    PYTHONOPTIMIZE=1 pyinstaller --onefile --name emulator --additional-hooks-dir hooks -y gui/main.py
}

main