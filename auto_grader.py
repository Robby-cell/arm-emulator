import sys
import argparse
from pathlib import Path
from typing import List

from arm_emulator_rs import Emulator, Peripheral, RangeInclusiveU32
from assembler import (
    arm_little_endian_assembler,
    AssembledOutput,
)

# =============================================================================
# PERIPHERAL DEFINITIONS
# =============================================================================


class MyPeripheral:
    def read32(self, addr: int) -> int:
        return self.data

    def write32(self, addr: int, data: int) -> None:
        self.data = data

    def read_byte(self, addr: int) -> int:
        return self.data & 0xFF

    def write_byte(self, addr: int, data: int) -> None:
        self.data = (self.data & ~0xFF) | (data & 0xFF)

    def reset(self) -> None:
        self.data = 0


# =============================================================================
# THE GRADING SCRIPT (Logic for specific assignments)
# =============================================================================


def run_grading_scenario(em: Emulator) -> None:
    """
    This is the core function where graders define their tests.
    Modify this function to match the requirements of the specific assignment.
    """
    print("--- Starting Grading Scenario ---")

    # Example: Manually write to hardware peripheral
    em.write32(0x40000000, 0x1234)

    # Perform assertions on peripheral state
    peripheral_val = em.read32(0x40000000)
    assert peripheral_val == 0x1234, f"Expected 0x1234, got {hex(peripheral_val)}"
    print(f"Peripheral State Verified: {hex(peripheral_val)}")

    # Execute a specific number of cycles
    print("Executing 5 steps...")
    for _ in range(5):
        if em.is_halted():  # Check if program reached SVC 0 or end
            break
        em.step()

    # Check register results
    # Assuming the lab expects R0 to be 0 at the end
    result = em.registers[0]
    print(f"Final R0 value: {result}")

    if result == 0:
        print("\033[92m[PASS]\033[0m: Assignment logic correct.")
    else:
        print("\033[91m[FAIL]\033[0m: R0 should be 0.")


# =============================================================================
# BOILERPLATE (Setup and Execution)
# =============================================================================


def main():
    parser = argparse.ArgumentParser(description="ARMv7 Headless Auto-Grader")
    parser.add_argument(
        "-c", "--code", type=str, required=True, help="Path to .asm file"
    )
    args = parser.parse_args()

    # 1. Initialize Emulator
    # Using defaults as the Rust constructor handles allocation etc.
    # Any allocations will also be handled when loading the code
    em = Emulator()

    # 2. Assemble and Load code
    asm_path = Path(args.code)
    if not asm_path.exists():
        print(f"Error: File {args.code} not found.")
        sys.exit(1)

    with open(asm_path, "r") as f:
        code_text = f.read()

    print(f"Assembling {asm_path.name}...")
    assembler = arm_little_endian_assembler()

    # We resolve symbols here just like the GUI does
    # This example maps 'IO_BASE' to the peripheral address
    assembler.add_symbol("IO_BASE", 0x40000000)

    try:
        out: AssembledOutput = assembler.assemble(code_text, start_address=0)
        em.load_program(out.text, out.sram, out.external)
    except Exception as e:
        print(f"Assembly/Loading Failed: {e}")
        sys.exit(1)

    # 3. Map Peripherals
    # Standard lab setup: A GPIO-like peripheral at 0x40000000
    em.add_peripheral(RangeInclusiveU32(0x40000000, 0x40000100), MyPeripheral())

    # 4. Execute the user-defined grading logic
    try:
        run_grading_scenario(em)
    except AssertionError as e:
        print(f"\033[91m[ASSERTION FAILED]\033[0m: {e}")
    except Exception as e:
        print(f"\033[91m[RUNTIME ERROR]\033[0m: {e}")


if __name__ == "__main__":
    main()
