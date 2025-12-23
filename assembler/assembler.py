from typing import Dict, List, Optional, Union
import re

from keystone import (
    KS_ARCH_ARM,
    KS_MODE_ARM,
    KS_MODE_BIG_ENDIAN,
    KS_MODE_LITTLE_ENDIAN,
    Ks,
    KsError,
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
            value[0] = addr
            return True
        return False

    def add_symbol(self, name: str, address: int) -> None:
        """
        Register a custom symbol (e.g. a constant mapped to a specific memory region).
        """
        self.symbols[name] = address

    def assemble(self, code: str, start_address: int = 0) -> AssembledOutput:
        """
        Assemble the string.
        :param code: Assembly code.
        :param start_address: Base address for the code (default 0x00000000).
        """
        # 1. Assemble the raw binary using Keystone
        try:
            encoding, count = self.ctx.asm(code, addr=start_address)
            binary = bytes(encoding)
        except KsError as e:
            return AssembledOutput(success=False, error=str(e))

        # Generate Source Map (Line <-> Address)
        # We assume standard ARM instructions (4 bytes).
        # We must filter out labels and comments to match the binary offset.
        # Generally, this should work well enough. So we won't bother with making it more complex.
        source_map = {}
        reverse_map = {}
        current_addr = start_address

        lines = code.split("\n")

        # Regex to detect labels (e.g., "loop:") and directives (e.g., ".global")
        # Everything else is considered an instruction.
        label_pattern = re.compile(r"^.*\s*[a-zA-Z0-9_]+:$")
        directive_pattern = re.compile(r"^.*\s*\.")
        comment_pattern = re.compile(r"^.*\s*(@|;)")
        empty_pattern = re.compile(r"^.*\s*$")

        instruction_count = 0

        for line_num, line in enumerate(lines):
            # Remove inline comments for checking
            clean = line.split("@")[0].split(";")[0].strip()

            if not clean:
                continue  # Empty line
            if label_pattern.match(clean):
                continue  # Label only
            if directive_pattern.match(clean):
                continue  # Directive

            # If we are here, we assume it's an instruction
            source_map[line_num] = current_addr
            reverse_map[current_addr] = line_num

            current_addr += 4
            instruction_count += 1

        return AssembledOutput(
            success=True, text=binary, source_map=source_map, reverse_map=reverse_map
        )


def arm_assembler_with_mode(mode) -> Assembler:
    return Assembler(KS_ARCH_ARM, KS_MODE_ARM | mode)


def arm_little_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_LITTLE_ENDIAN)


def arm_big_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_BIG_ENDIAN)
