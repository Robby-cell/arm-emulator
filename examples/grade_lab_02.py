# Assignment: 3 Blinks, then Fibonacci(R8) -> R0

led = get_peripherals()[0]
print(led)

# 1. Setup the test parameters
test_n = 7
expected_fib = 13  # Fib sequence: 0, 1, 1, 2, 3, 5, 8, 13...
system.set_register(R8, test_n)


print(f"Starting test with n={test_n}")

# 2. Execution loop with transition tracking
steps = 0
MAX_STEPS = 500

while not is_finished() and steps < MAX_STEPS:
    step()
    steps += 1

# 3. Assertions
print("\n--- Final Results ---")

on_count = 3
off_count = 3

# Verify Blinking Logic
assert led.on_count == 3, f"LED should turn ON exactly 3 times. Found: {led.on_count}"
assert (
    led.off_count == 3
), f"LED should turn OFF exactly 3 times. Found: {led.off_count}"
assert led.is_led_on() is False, "LED must be OFF when the program finishes."
print("Requirement [Blink 3 Times]: PASS")

# Verify Fibonacci Result
result = get_register(R0)
assert result == expected_fib, f"Fibonacci error! Expected {expected_fib}, got {result}"
print(f"Requirement [Fibonacci({test_n})]: PASS (Result: {result})")

# Verify Exit Status
assert get_register(R7) == 1, "Program did not use exit syscall (R7=1)"
print("Requirement [Exit Syscall]: PASS")
