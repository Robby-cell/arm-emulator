import re
from typing import Dict, Union

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
            value[0] = self.symbols[sym_str]
            return True
        return False

    def add_symbol(self, name: str, address: int) -> None:
        """
        Register a custom symbol (e.g. a constant mapped to a specific memory region).
        """
        self.symbols[name] = address

    def _convert_decimals_to_hex(self, full_code: str) -> str:
        """
        Scans the entire code block and replaces standalone decimal numbers with hex.
        Executed in one pass for performance.
        """
        # Regex Explanation:
        # (?<![\w.]) : Negative Lookbehind. Prev char must NOT be word char (a-z,0-9,_) or dot.
        #              This prevents matching 'R10', '0x10', '.10'.
        # (-?\d+)    : Group 1. Optional minus sign followed by digits.
        # (?![\w.])  : Negative Lookahead. Next char must NOT be word char or dot.
        pattern = re.compile(r"(?<![\w.])(-?\d+)(?![\w.])")

        def replacer(match):
            number_str = match.group(1)
            try:
                # Convert "4294967295" -> "0xffffffff"
                value = int(number_str)
                return hex(value)
            except ValueError:
                return number_str

        return pattern.sub(replacer, full_code)

    def assemble(self, code: str, start_address: int = 0) -> AssembledOutput:
        """
        Assemble the string.
        :param code: Assembly code.
        :param start_address: Base address for the code (default 0x00000000).
        """

        # 1. Efficient Pre-process: Convert all decimals to hex in one go
        full_processed_code = self._convert_decimals_to_hex(code)

        # 2. Assemble using Keystone
        try:
            encoding, count = self.ctx.asm(full_processed_code, addr=start_address)
            binary = bytes(encoding)
        except KsError as e:
            return AssembledOutput(success=False, error=str(e))

        # 3. Generate Source Map (Line <-> Address)
        # We must still loop here to calculate addresses for the UI,
        # but we use the original code to ensure line numbers match what the user sees.
        source_map = {}
        reverse_map = {}
        current_addr = start_address

        lines = code.split("\n")

        # Regex to detect labels (e.g., "loop:") and directives (e.g., ".global")
        # Everything else is considered an instruction.
        label_pattern = re.compile(r"^\s*[a-zA-Z0-9_]+:$")
        directive_pattern = re.compile(r"^\s*\.")

        for line_num, line in enumerate(lines):
            clean = line.split("@")[0].split(";")[0].strip()

            if not clean:
                continue
            if label_pattern.match(clean):
                continue

            # Handle Data Directives Memory Usage
            if directive_pattern.match(clean):
                if any(x in clean for x in [".long", ".word", ".int"]):
                    source_map[line_num] = current_addr
                    reverse_map[current_addr] = line_num
                    current_addr += 4
                elif any(x in clean for x in [".short", ".hword"]):
                    source_map[line_num] = current_addr
                    reverse_map[current_addr] = line_num
                    current_addr += 2
                elif ".byte" in clean:
                    try:
                        args = clean.split(".byte")[1]
                        count = args.count(",") + 1
                        source_map[line_num] = current_addr
                        reverse_map[current_addr] = line_num
                        current_addr += count
                    except IndexError:
                        pass
                continue

            # Standard Instruction
            source_map[line_num] = current_addr
            reverse_map[current_addr] = line_num
            current_addr += 4

        return AssembledOutput(
            success=True, text=binary, source_map=source_map, reverse_map=reverse_map
        )


def arm_assembler_with_mode(mode) -> Assembler:
    return Assembler(KS_ARCH_ARM, KS_MODE_ARM | mode)


def arm_little_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_LITTLE_ENDIAN)


def arm_big_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_BIG_ENDIAN)
