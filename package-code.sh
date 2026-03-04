#!/usr/bin/env bash

export PYI_MAKESPEC
export PYINSTALLER

set -e

function _source_venv() {
    # Since bash CAN be run on windows, and we will use it in CI/CD pipelines.
    # Windows, for whatever reason, puts it in `Scripts` instead of `bin`.
    local NORMAL_VENV=".venv/bin/activate"
    local WINDOWS_VENV=".venv/Scripts/activate"
    if test -f $NORMAL_VENV; then
        echo "Using normal venv"
        source ./$NORMAL_VENV
    elif test -f $WINDOWS_VENV; then
        echo "Using windows venv"
        source ./$WINDOWS_VENV
    fi
}

function _ensure_venv() {
    if [ ! -d ".venv" ]; then
        python3 -m venv .venv
    fi
}

function _build_translations() {
    echo "Building translations..."

    bash build-translations.sh

    echo "Translations built."
}

function main() {
    echo "Building standalone executable using PyInstaller."

    echo "Creating spec file..."
    $PYI_MAKESPEC gui_main.py --name arm_emulator

    _build_translations

    echo "Building executable..."
    # use pyinstaller to create a standalone binary.
    # --onefile: create a single executable file, no additional directories needed.
    # --name emulator: name the output executable "emulator"
    # --additional-hooks-dir hooks: include additional hooks from the "hooks" directory.
    # --add-data "assets:assets": include the assets directory in the executable.
    export PYTHONOPTIMIZE=1
    $PYINSTALLER --noconfirm --onefile --windowed --name arm_emulator --additional-hooks-dir hooks --add-data "assets:assets" -y gui_main.py --exclude-module PySide6
}

main
