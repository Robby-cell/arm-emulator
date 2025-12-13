from dataclasses import dataclass
from typing import Optional


@dataclass
class AssembledOutput:
    text: Optional[bytes] = None
    sram: Optional[bytes] = None
    external: Optional[bytes] = None
