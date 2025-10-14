# CPU Architecture

The actual CPU part of the project can be broken into further parts.

```rs
struct Cpu {
    registers: [u32, 16],
    /* other fields, like cpsr, etc. */
}
struct Bus {
    memory: Vec<u8>,
    mapped_peripherals: /* more detail in the implementation */,
}
struct Emulator {
    cpu: Cpu,
    memory_bus: Bus,
    endian: Endian,
}
```

This side of the project is really just a library.
In this side of the project, there is the modeling of the underlying hardware (the CPU, but also the memory, memory mapped peripherals etc.), and the decoding of the instructions, and execution of the instructions (the actual emulation part).
The CPU in the emulator will have nothing to do with the decoding of instructions, which is unlike a real CPU. This will be handled natively, via software (i.e. `impl TryFrom<u32> for Instruction`).

Bindings can be created using this part of the project, for whatever the desired use is.

# Requirements

This side comprises the actual logic, it should be correct. Error handling is a priority - the application crashing randomly doesn't make much sense, and would greatly hinder the app, if it crashes because of something on this side of the app.
Errors will be made easy with `thiserror`. This will allow error unions to easily be composed, and bubbled up as we move up the call stack.

```rs
use thiserror::Error;

#[derive(Error, Debug)]
enum SimpleError {
    #[error("error variant 1: {0}")]
    ErrorVariant1(String),
}

#[derive(Error, Debug)]
enum ComplexError {
    #[error("simple error: {0}")]
    SimpleError(#[from] SimpleError),

    #[error("foobar error")]
    Foobar,
}

fn first() -> Result<(), SimpleError> {
    Err(SimpleError::ErrorVariant1("Error message".into()))
}

fn second() -> Result<(), ComplexError> {
    first()?;
    Ok(())
}
```

As for internals of the device that will be emulated, using native type that are specific to each device will be avoided (i.e. using `usize`, 64 bits on x86_64, but 32 bits on x86). Instead, concrete types that are more accurate and closer to the real thing will be preferred (i.e. u32).

The reading and writing of memory can be done with little-endian or big-endian. The CPU itself will not worry about handling this, and the memory won't necessarily worry about it either, the emulator will worry about emulating this, by tracking what is being used.
