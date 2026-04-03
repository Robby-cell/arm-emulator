import sys
import argparse
from arm_emulator_rs import Emulator, RangeInclusiveU32  # type: ignore

from assembler import arm_little_endian_assembler
from hardware import PyGpioPort

# 1. Global State
system: Emulator = Emulator()
assembler = arm_little_endian_assembler()
asm_filename: str


# 2. Helper Functions (Exposed to the test script)
def step(count: int = 1) -> None:
    """Advances the system by N instructions."""
    for _ in range(count):
        if system.is_finished():
            break
        system.step()


def is_finished() -> bool:
    return system.is_finished()


def run() -> None:
    """Runs the system until a halt or breakpoint is hit."""
    while not system.is_finished():
        system.step()


def reg(index: int) -> int:
    """Returns the value of register R0-R15."""
    return system.registers[index]


def mem(addr: int) -> int:
    """Reads a byte from the bus."""
    return system.read_byte(addr)


def map_peripheral(start: int, end: int, obj) -> None:
    """Maps a Python object to the memory bus."""
    system.add_peripheral(RangeInclusiveU32(start, end), obj)


def add_symbol(name: str, addr: int) -> None:
    assembler.add_symbol(name, addr)


def load_program() -> None:
    # Load and Assemble student code
    global asm_filename
    with open(asm_filename, "r") as f:
        asm_raw = f.read()
        out = assembler.assemble(asm_raw, start_address=0)
    system.load_program(out.text, out.sram, out.external)


# 3. The Orchestrator
def main() -> None:
    global system
    global asm_filename

    parser = argparse.ArgumentParser()
    parser.add_argument("-s", "--script", required=True, help="The .py test script")
    parser.add_argument("-a", "--asm", required=True, help="The student .asm file")
    args = parser.parse_args()

    asm_filename = args.asm

    # Read the professor's test script
    with open(args.script, "r") as f:
        script_code = f.read()

    # Define the environment for exec()
    # This makes 'system', 'step', etc., available globally in the script
    env = {
        "system": system,
        "is_finished": is_finished,
        "step": step,
        "run": run,
        "reg": reg,
        "mem": mem,
        "map_peripheral": map_peripheral,
        "hex": hex,
        "print": print,
        "add_symbol": add_symbol,
        "load_program": load_program,
        "PyGpioPort": PyGpioPort,
        "AssertionError": AssertionError,
    }

    print(f"--- Executing Test Script: {args.script} ---")
    try:
        exec(script_code, env)
        print("\033[92m[FINAL RESULT]: ALL TESTS PASSED\033[0m")
    except AssertionError as e:
        print(f"\033[91m[ASSERTION FAILED]:\033[0m {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\033[91m[RUNTIME ERROR]:\033[0m {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
