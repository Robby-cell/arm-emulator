from typing import Dict, Union

from keystone import (
    KS_ARCH_ARM,
    KS_MODE_ARM,
    KS_MODE_BIG_ENDIAN,
    KS_MODE_LITTLE_ENDIAN,
    Ks,
)

from .assembled_output import AssembledOutput


class Assembler:
    _ctx: Ks
    symbols: Dict[str, int]

    def __init__(self, arch, mode):
        self.symbols = {}
        # Initialize context (this will also trigger the setter to register the resolver)
        self.ctx = Ks(arch=arch, mode=mode)

    @property
    def ctx(self) -> Ks:
        return self._ctx

    @ctx.setter
    def ctx(self, ctx: Ks) -> None:
        self._ctx = ctx
        # Register the symbol resolver callback with the new Keystone instance
        self._ctx.sym_resolver = self._resolve_symbol

    def _resolve_symbol(self, symbol: Union[str, bytes], value) -> bool:
        """
        Callback for Keystone to resolve symbols.
        'value' is a mutable list/array where the resolved address must be stored at index 0.
        """
        sym_str = symbol.decode("utf-8") if isinstance(symbol, bytes) else symbol

        if sym_str in self.symbols:
            addr = self.symbols[sym_str]
            print(f"[Assembler] Resolved '{sym_str}' to {hex(addr)}")
            value[0] = addr
            return True

        print(f"[Assembler] Failed to resolve '{sym_str}'")
        return False

    def add_symbol(self, name: str, address: int) -> None:
        """
        Register a custom symbol (e.g. a constant mapped to a specific memory region).
        """
        self.symbols[name] = address

    def assemble(self, string: str, address: int = 0) -> AssembledOutput:
        """
        Assemble the string.
        :param string: Assembly code.
        :param address: Base address for the code (default 0x00000000).
        """
        # Keystone asm returns [encoding, count]
        [text, _count] = self.ctx.asm(string=string, addr=address)
        text = bytes(bytearray(text))
        return AssembledOutput(text=text)


def arm_assembler_with_mode(mode) -> Assembler:
    return Assembler(KS_ARCH_ARM, KS_MODE_ARM | mode)


def arm_little_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_LITTLE_ENDIAN)


def arm_big_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_BIG_ENDIAN)
