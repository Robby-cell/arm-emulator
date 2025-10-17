from arm_emulator_rs import emulator, peripheral, RangeInclusiveU32

class MyPeripheral:
    def __init__(self): ...

    def read32(self, addr: int) -> int:
        return 0

    def write32(self, addr: int, data: int) -> None: ...


class PyGpioPort(peripheral.GpioPort):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    def read32(self, addr: int) -> int:
        res: int = super().read32(addr)
        print("PyGpioPort read")
        return res

    def write32(self, addr: int, data: int) -> None:
        res: None = super().write32(addr, data)
        print("PyGpioPort write")
        return res


em = emulator.Emulator(20)
em.add_peripheral(RangeInclusiveU32(4096, 4127), peripheral.GpioPort())
em.add_peripheral(RangeInclusiveU32(4128, 4159), MyPeripheral())
em.add_peripheral(RangeInclusiveU32(4160, 4191), PyGpioPort())

em.write32(4096, 0x33)
em.write32(4160, 0x44)

print(em.read32(4096))
em.write32(4128, 0x2B)
print(em.read32(4160))

em.write32(0, 0x1234)
em.write32(4, 0x5678)

print(em)
