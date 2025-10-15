from arm_emulator import *


class MyPeripheral:
    def __init__(self): ...

    def read32(self, addr: int) -> int:
        return 0

    def write32(self, addr: int, data: int) -> None: ...


class PyGpioPort(GpioPort):
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


emulator = Emulator(20)
emulator.add_peripheral(RangeInclusiveU32(4096, 4127), GpioPort())
emulator.add_peripheral(RangeInclusiveU32(4128, 4159), MyPeripheral())
emulator.add_peripheral(RangeInclusiveU32(4160, 4191), PyGpioPort())

emulator.write32(4096, 0x33)
emulator.write32(4160, 0x44)

print(emulator.read32(4096))
emulator.write32(4128, 0x2B)
print(emulator.read32(4160))

print(emulator)
