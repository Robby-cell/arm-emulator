#!/usr/bin/env bash

pyi-makespec gui/main.py --name emulator
pyinstaller --name emulator --additional-hooks-dir hooks -y gui/main.py
