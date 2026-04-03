from arm_emulator_rs import GpioPort  # type: ignore


class PyGpioPort(GpioPort):
    _on_count: int = 0
    _off_count: int = 0

    name: str | None
    begin: int | None
    end: int | None

    @property
    def on_count(self) -> int:
        return self._on_count

    @property
    def off_count(self) -> int:
        return self._off_count

    def __new__(cls, *args, **kwargs) -> "PyGpioPort":
        # Ignore the passed in args and kwargs.
        # This instantiates super, but does not call __init__.
        # This calls the rust constructor provided.
        return super().__new__(cls)

    def __init__(
        self, name: str | None = None, begin: int | None = None, end: int | None = None
    ) -> None:
        super().__init__()
        self.name = name
        self.begin = begin
        self.end = end

    def read32(self, addr: int) -> int:
        return super().read32(addr)

    def write32(self, addr: int, data: int) -> None:
        pre = self.is_led_on()
        super().write32(addr, data)
        post = self.is_led_on()
        if pre != post:
            if post:
                self._on_count += 1
            else:
                self._off_count += 1

    def read_byte(self, addr: int) -> int:
        return super().read_byte(addr)

    def write_byte(self, addr: int, data: int) -> None:
        pre = self.is_led_on()
        super().write_byte(addr, data)
        post = self.is_led_on()
        if pre != post:
            if post:
                self._on_count += 1
            else:
                self._off_count += 1

    def reset(self) -> None:
        super().reset()

    def is_led_on(self) -> bool:
        return ((self.read32(0) & 0x400) == 0x400) and (
            (self.read32(0x14) & 0x20) == 0x20
        )
