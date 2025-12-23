from dataclasses import dataclass, field
from typing import Dict, Optional


@dataclass
class AssembledOutput:
    text: Optional[bytes] = None
    sram: Optional[bytes] = None
    external: Optional[bytes] = None
    success: bool = False
    error: Optional[str] = None
    source_map: Dict[int, int] = field(default_factory=dict)
    reverse_map: Dict[int, int] = field(default_factory=dict)
