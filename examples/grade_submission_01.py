class MockHardware:
    def __init__(self) -> None:
        self.received_value = 0

    def read32(self, addr: int) -> int:
        return self.received_value

    def write32(self, addr: int, data: int) -> None:
        self.received_value = data

    def read_byte(self, addr: int) -> int:
        return self.received_value & 0xFF

    def write_byte(self, addr: int, data: int) -> None:
        self.received_value = (self.received_value & 0xFFFFFF00) | data

    def reset(self) -> None:
        self.received_value = 0


mock_hw = MockHardware()
add_symbol("IO_BASE", 0x40000000)
load_program()
# Map peripheral after loading program. Loading will reset peripherals.
map_peripheral(0x40000000, 0x40000FFF, mock_hw)

print("Executing student code...")

cycles = 0
while not is_finished() and cycles < 100:
    step()
    cycles += 1
# 4. Assertions (The actual Grading)
print(f"Executed {cycles} instructions.")

# Test 1: Did they write the correct value (30) to the hardware?
if mock_hw.received_value == 30:
    print("\033[92m[PASS]\033[0m Hardware Write: Student wrote 30 to IO_BASE.")
else:
    print(
        f"\033[91m[FAIL]\033[0m Hardware Write: Expected 30, got {mock_hw.received_value}",
    )

# Test 2: Did the program exit with code 0?
if reg(0) == 0:
    print("\033[92m[PASS]\033[0m Exit Code: Program returned 0.")
else:
    print(f"\033[91m[FAIL]\033[0m Exit Code: Expected R0=0, got R0={reg(0)}")
