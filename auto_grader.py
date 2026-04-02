import sys
import argparse
from pathlib import Path

from arm_emulator_rs import Emulator, RangeInclusiveU32  # type: ignore
from assembler import (
    arm_little_endian_assembler,
    AssembledOutput,
)

# =============================================================================
# PERIPHERAL DEFINITIONS
# =============================================================================


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


# =============================================================================
# THE GRADING SCRIPT (Logic for specific assignments)
# =============================================================================


def run_grading_scenario(em: Emulator) -> None:
    """
    This is the core function where graders define their tests.
    Modify this function to match the requirements of the specific assignment.
    """

    mock_hw = MockHardware()
    em.add_peripheral(RangeInclusiveU32(0x40000000, 0x40000FFF), mock_hw)

    print("Executing student code...")
    cycles = 0
    while not em.is_finished() and cycles < 100:
        em.step()
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
    if em.registers[0] == 0:
        print("\033[92m[PASS]\033[0m Exit Code: Program returned 0.")
    else:
        print(
            f"\033[91m[FAIL]\033[0m Exit Code: Expected R0=0, got R0={em.registers[0]}"
        )


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

    # 3. Execute the user-defined grading logic
    try:
        run_grading_scenario(em)
    except AssertionError as e:
        print(f"\033[91m[ASSERTION FAILED]\033[0m: {e}")
    except Exception as e:
        print(f"\033[91m[RUNTIME ERROR]\033[0m: {e}")


if __name__ == "__main__":
    main()
