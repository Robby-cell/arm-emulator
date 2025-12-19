from typing import List, Tuple

from PyQt6.QtWidgets import QWidget


def get_languages_and_codes(translator: QWidget) -> List[Tuple[str, str]]:
    """Returns a list of language names and their corresponding codes"""
    return [
        (translator.tr("English"), "en"),
        (translator.tr("Русский"), "ru"),
        (translator.tr("Polski"), "pl"),
        (translator.tr("Español"), "es"),
    ]
