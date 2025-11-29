from dataclasses import dataclass
from typing import Optional


@dataclass
class AssembledOutput:
    text: Optional[bytearray] = None
    sram: Optional[bytearray] = None
    external: Optional[bytearray] = None
