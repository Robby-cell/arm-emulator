from .gpio import PyGpioPort

from typing import Any

_PERIPHERAL_REGISTRY: dict[str, type] = {
    "LED": PyGpioPort,
}


class DefaultTraits:
    def __init__(self, the_type: type) -> None:
        self._type = the_type

    def create(self, name: str, start: int, end: int) -> Any:
        return self._type(name, start, end)


def create_registry(traits: dict[str, Any] = {}) -> dict[str, Any]:
    return {
        key: traits[key]
        if traits is not None and key in traits
        else DefaultTraits(value)
        for key, value in _PERIPHERAL_REGISTRY.items()
    }


__all__ = ["PyGpioPort", "create_registry"]
