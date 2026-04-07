begin_addr = 0x40000000
end_addr = 0x40000FFF

mock_hw = PyGpioPort("IO_BASE", begin_addr, end_addr)
add_symbol("IO_BASE", begin_addr)
load_program()
# Map peripheral after loading program. Loading will reset peripherals.
map_peripheral(begin_addr, end_addr, mock_hw)

print("Executing student code...")

cycles = 0
while not is_finished() and cycles < 100:
    step()
    cycles += 1
# 4. Assertions (The actual Grading)
print(f"Executed {cycles} instructions.")

# Test 1: Did they write the correct value (30) to the hardware?
if mock_hw.read32(0) == 30:
    output_success("Hardware Write: Student wrote 30 to IO_BASE.")
else:
    output_failure(f"Hardware Write: Expected 30, got {mock_hw.read32(0)}")

# We can even test the LED state
# We didn't configure moder and write to odr, so it should be off
assert mock_hw.is_led_on() is False
assert mock_hw.off_count == 0 and mock_hw.on_count == 0

# Test 2: Did the program exit with code 0?
if get_register(R0) == 0:
    output_success("Exit Code: Program returned 0.")
else:
    output_failure(f"Exit Code: Expected R0=0, got R0={get_register(R0)}")
