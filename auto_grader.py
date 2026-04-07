import sys
import argparse
import json
from pathlib import Path
import logging

from arm_emulator_rs import Emulator, Peripheral, RangeInclusiveU32  # type: ignore

from assembler import arm_little_endian_assembler
from hardware import PyGpioPort

# 1. Global State
system: Emulator = Emulator()
assembler = arm_little_endian_assembler()
asm_filename: str
instruction_quota = 500


class InstructionQuotaExceeded(Exception):
    _message: str

    def __init__(self, message) -> None:
        self._message = message

    def __str__(self) -> str:
        return self._message


# 2. Helper Functions (Exposed to the test script)
def set_instruction_quota(quota: int) -> None:
    """Sets the maximum number of instructions to execute."""
    global instruction_quota
    instruction_quota = quota


def get_instruction_quota() -> int:
    """Returns the maximum number of instructions to execute."""
    global instruction_quota
    return instruction_quota


def step(count: int = 1) -> None:
    """Advances the system by N instructions."""
    global instruction_quota
    for _ in range(count):
        if instruction_quota <= 0:
            logging.error("Exceeded instruction quota.")
            raise InstructionQuotaExceeded("Exceeded instruction quota.")
        if system.is_finished():
            break
        system.step()
        instruction_quota -= 1


def is_finished() -> bool:
    return system.is_finished()


def run() -> None:
    """Runs the system until a halt or breakpoint is hit."""
    while not system.is_finished():
        system.step()


def set_register(index: int, value: int) -> None:
    """Sets the value of register R0-R15."""
    system.set_register(index, value)


def get_register(index: int) -> int:
    """Returns the value of register R0-R15."""
    return system.registers[index]


def read_byte(addr: int) -> int:
    """Reads a byte from the bus."""
    return system.read_byte(addr)


def write_byte(addr: int, value: int) -> None:
    """Writes a byte to the bus."""
    system.write_byte(addr, value)


def read32(addr: int) -> int:
    """Reads a 32-bit word from the bus."""
    return system.read32(addr)


def write32(addr: int, value: int) -> None:
    """Writes a 32-bit word to the bus."""
    system.write32(addr, value)


def map_peripheral(start: int, end: int, obj) -> None:
    """Maps a Python object to the memory bus."""
    system.add_peripheral(RangeInclusiveU32(start, end), obj)


def get_peripherals() -> list[Peripheral]:
    """Returns a list of all mapped peripherals."""
    return system.peripherals


def add_symbol(name: str, addr: int) -> None:
    assembler.add_symbol(name, addr)


def load_program() -> None:
    """
    Loads a program into the emulator.
    Supports both raw assembly (.asm/.s) and workspace configurations (.armcfg).
    """
    global asm_filename
    path = Path(asm_filename)

    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    code_to_assemble = ""
    peripherals = []

    # 1. Check if the file is a JSON workspace configuration
    if path.suffix == ".armcfg":
        try:
            config = json.loads(content)
            code_to_assemble = config.get("code", "")

            # 2. Automatically map peripherals found in the configuration
            if "peripherals" in config:
                for p in config["peripherals"]:
                    # Use the helper functions already defined in your script
                    # to keep the global state synchronized
                    start = p["start"]
                    end = p["end"]
                    name = p["name"]

                    # Instantiate the appropriate Python hardware type
                    # (Assuming PyGpioPort is the standard for the LED type)
                    obj = PyGpioPort()

                    peripherals.append((start, end, obj))
                    add_symbol(name, start)
                    logging.info(f"Auto-configured peripheral: {name} at {hex(start)}")

            logging.info(f"Successfully loaded workspace: {path.name}")

        except json.JSONDecodeError:
            logging.error(f"{path.name} has .armcfg extension but is not valid JSON.")
            sys.exit(1)
    else:
        # 3. Fallback to treating the file as raw assembly text
        code_to_assemble = content

    # 4. Assemble and Load into the Rust Backend
    out = assembler.assemble(code_to_assemble, start_address=0)

    if out.text is None:
        logging.error("Error: Failed to assemble code...")
        exit(1)

    system.load_program(out.text, out.sram, out.external)
    for begin, end, obj in peripherals:
        map_peripheral(begin, end, obj)


# 3. The Orchestrator
def main() -> None:
    global system
    global asm_filename

    logging.basicConfig(level=logging.DEBUG)

    parser = argparse.ArgumentParser()
    parser.add_argument("-s", "--script", required=True, help="The .py test script")
    parser.add_argument("-a", "--asm", required=True, help="The student .asm file")
    parser.add_argument(
        "--no-pre-load",
        action="store_true",
        help=(
            "Do not pre-load the student program. "
            "If specified, the program will no load the students code before running the grading script. "
            "Must be loaded within the script"
        ),
    )
    args = parser.parse_args()

    asm_filename = args.asm

    # Read the professor's test script
    with open(args.script, "r") as f:
        script_code = f.read()

    # Define the environment for exec()
    # This makes 'system', 'step', etc., available globally in the script
    env = {
        **{
            "system": system,
            "is_finished": is_finished,
            "set_instruction_quota": set_instruction_quota,
            "get_instruction_quota": get_instruction_quota,
            "step": step,
            "run": run,
            "get_register": get_register,
            "set_register": set_register,
            "read_byte": read_byte,
            "write_byte": write_byte,
            "read32": read32,
            "write32": write32,
            "map_peripheral": map_peripheral,
            "get_peripherals": get_peripherals,
            "hex": hex,
            "print": print,
            "add_symbol": add_symbol,
            "load_program": load_program,
            "PyGpioPort": PyGpioPort,
            "AssertionError": AssertionError,
            "InstructionQuotaExceeded": InstructionQuotaExceeded,
        },
        **{f"R{i}": i for i in range(16)},
        "SP": 13,
        "LR": 14,
        "PC": 15,
    }
    pre_load = not args.no_pre_load
    if pre_load:
        load_program()

    logging.info(f"Executing Test Script: {args.script}")
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
