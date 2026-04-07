//! GPIO (General Purpose Input/Output) peripheral implementation.
//!
//! This module provides a simulated GPIO peripheral similar to STM32 microcontrollers.
//! It models:
//! - MODER register: GPIO port mode configuration (input/output/alternate function)
//! - ODR register: Output data register for controlling pins
//!
//! The implementation uses atomic operations for thread-safety and supports
//! LED visualization (specifically PA5) for the educational demo.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::memory::{
    MemoryAccessError, MemoryAccessResult, Peripheral, Word,
    u32_from_native_bytes, u32_to_native_bytes,
};

/// The internal state of the gpio port
#[must_use]
struct GpioState {
    moder: AtomicU32,
    odr: AtomicU32,
}

impl GpioState {
    pub const fn new(moder: u32, odr: u32) -> Self {
        Self {
            moder: AtomicU32::new(moder),
            odr: AtomicU32::new(odr),
        }
    }

    pub const fn zero() -> Self {
        Self::new(0, 0)
    }
}

/// The public-facing struct representing a gpio port
#[must_use]
pub struct GpioPort {
    state: GpioState,
}

impl GpioPort {
    pub const fn new() -> Self {
        Self {
            state: GpioState::zero(),
        }
    }

    // A new public method for a UI thread to safely read the LED state
    #[must_use]
    pub fn is_led_on(&self) -> bool {
        // Check if PA5 is configured as an output and its ODR bit is set
        let is_output = (self.state.moder.load(Ordering::Relaxed) >> 10)
            & 0b11
            == 0b01;
        let is_high =
            (self.state.odr.load(Ordering::Relaxed) & (1 << 5)) != 0;
        is_output && is_high
    }
}

impl Peripheral for GpioPort {
    fn read32(&self, offset: Word) -> MemoryAccessResult<u32> {
        let value = match offset {
            0x00 => self.state.moder.load(Ordering::Relaxed),
            0x14 => self.state.odr.load(Ordering::Relaxed),
            _ => {
                tracing::warn!(
                    "Read from invalid offset in GpioPort: {offset:#X}."
                );
                return Err(MemoryAccessError::InvalidPeripheralRead {
                    offset,
                });
            }
        };

        Ok(value)
    }

    fn write32(&self, offset: Word, value: u32) -> MemoryAccessResult<()> {
        match offset {
            0x00 => self.state.moder.store(value, Ordering::Relaxed),
            0x14 => self.state.odr.store(value, Ordering::Relaxed),
            0x18 => {
                // BSRR
                let set_bits = value & 0xFFFF;
                let reset_bits = (value >> 16) & 0xFFFF;

                // state.odr |= set_bits;
                // state.odr &= !reset_bits;

                self.state.odr.store(
                    self.state.odr.fetch_or(set_bits, Ordering::Relaxed)
                        & !reset_bits,
                    Ordering::Relaxed,
                );
                // Side effects can be triggered here
            }
            _ => {
                tracing::warn!(
                    "Write at invalid offset in GpioPort: {offset:#X}."
                );
                return Err(MemoryAccessError::InvalidPeripheralWrite {
                    offset,
                });
            }
        }
        Ok(())
    }

    fn read_byte(&self, offset: u32) -> MemoryAccessResult<u8> {
        let spare = offset % 4;
        let offset = offset - spare;
        let value = self.read32(offset)?;
        let bytes = u32_to_native_bytes(value);

        Ok(bytes[spare as usize])
    }

    fn write_byte(
        &self,
        offset: u32,
        value: u8,
    ) -> MemoryAccessResult<()> {
        let spare = offset % 4;
        let offset = offset - spare;
        let v = self.read32(offset)?;
        let mut v = u32_to_native_bytes(v);
        v[spare as usize] = value;
        let v = u32_from_native_bytes(v);

        self.write32(offset, v)
    }

    fn reset(&self) {
        self.state.moder.store(0, Ordering::Relaxed);
        self.state.moder.store(0, Ordering::Relaxed);
    }
}
