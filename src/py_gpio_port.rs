use pyo3::prelude::*;

use emulator::{
    memory::{MemoryAccessResult, Peripheral},
    peripherals::gpio::GpioPort,
};

use crate::error::ToPyResult;

#[pyclass(name = "GpioPort", subclass)]
struct PyGpioPort {
    gpio: GpioPort,
}

#[pymethods]
impl PyGpioPort {
    #[new]
    fn new() -> Self {
        Self {
            gpio: GpioPort::new(),
        }
    }

    fn is_led_on(&self) -> bool {
        self.gpio.is_led_on()
    }

    #[pyo3(name = "read32")]
    fn py_read32(&self, offset: u32) -> PyResult<u32> {
        self.read32(offset).to_py_result()
    }

    #[pyo3(name = "write32")]
    fn py_write32(&self, offset: u32, value: u32) -> PyResult<()> {
        self.write32(offset, value).to_py_result()
    }
}

impl Peripheral for PyGpioPort {
    fn read32(&self, offset: u32) -> MemoryAccessResult<u32> {
        self.gpio.read32(offset)
    }

    fn write32(&self, offset: u32, value: u32) -> MemoryAccessResult<()> {
        self.gpio.write32(offset, value)
    }
}

#[pymodule]
pub(crate) fn py_gpio_port(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGpioPort>()?;
    Ok(())
}
