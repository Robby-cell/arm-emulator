from keystone import (
    KS_ARCH_ARM,
    KS_MODE_ARM,
    KS_MODE_BIG_ENDIAN,
    KS_MODE_LITTLE_ENDIAN,
    Ks,
)

from . import AssembledOutput


class Assembler:
    _ctx: Ks

    def __init__(self, arch, mode):
        self.ctx = Ks(arch=arch, mode=mode)

    @property
    def ctx(self) -> Ks:
        return self._ctx

    @ctx.setter
    def ctx(self, ctx: Ks) -> None:
        self._ctx = ctx

    def assemble(self, string: str) -> AssembledOutput:
        [text, _count] = self.ctx.asm(string=string)
        text = bytes(bytearray(text))
        return AssembledOutput(text=text)


def arm_assembler_with_mode(mode) -> Assembler:
    return Assembler(KS_ARCH_ARM, KS_MODE_ARM | mode)


def arm_little_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_LITTLE_ENDIAN)


def arm_big_endian_assembler() -> Assembler:
    return arm_assembler_with_mode(KS_MODE_BIG_ENDIAN)
