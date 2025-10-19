# hook-arm_emulator_rs.py
from PyInstaller.utils.hooks import collect_dynamic_libs

binaries = collect_dynamic_libs('arm_emulator_rs')
