//! Note: <InstructionType>::from(u32) is not affected by endianness.
//! We will call a function that we have defined to make this conversion

use crate::{
    Emulator,
    cpu::Cpu,
    memory::{Bus, Endian},
};

mod branch;
mod data_processing;
mod memory_access;

fn ramless_emulator(endian: Endian) -> Emulator {
    Emulator::new(Cpu::new(), Bus::new(0), endian)
}
