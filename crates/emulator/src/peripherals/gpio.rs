use crate::memory::{
    MemoryAccessError, MemoryAccessResult, Peripheral, Word,
};
use std::sync::atomic::{AtomicU32, Ordering};

/// The internal state of the gpio port
struct GpioState {
    moder: AtomicU32,
    odr: AtomicU32,
}

/// The public-facing struct representing a gpio port
pub struct GpioPort {
    state: GpioState,
}

impl GpioPort {
    pub fn new() -> Self {
        Self {
            state: GpioState {
                moder: 0.into(),
                odr: 0.into(),
            },
        }
    }

    // A new public method for a UI thread to safely read the LED state
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
    fn read(&self, offset: Word) -> MemoryAccessResult<u32> {
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

    fn write(&self, offset: Word, value: u32) -> MemoryAccessResult<()> {
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
}
