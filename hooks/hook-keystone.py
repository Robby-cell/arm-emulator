# hook-keystone.py
from PyInstaller.utils.hooks import collect_all

# Collect everything: compiled DLLs, python files, and data files
datas, binaries, hiddenimports = collect_all("keystone")
