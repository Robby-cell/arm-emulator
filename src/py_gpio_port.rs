use pyo3::prelude::*;

use emulator::{
    memory::{MemoryAccessResult, Peripheral},
    peripherals::gpio::GpioPort,
};

use crate::error::ToPyResult;

#[pyclass(name = "GpioPort")]
pub(crate) struct PyGpioPort {
    gpio: GpioPort,
}

#[pymethods]
impl PyGpioPort {
    #[new]
    pub(crate) fn __new__() -> Self {
        Self {
            gpio: GpioPort::new(),
        }
    }

    pub(crate) fn is_led_on(&self) -> bool {
        self.gpio.is_led_on()
    }

    pub(crate) fn read(&self, offset: u32) -> PyResult<u32> {
        Peripheral::read(self, offset).to_py_result()
    }

    pub(crate) fn write(&self, offset: u32, value: u32) -> PyResult<()> {
        Peripheral::write(self, offset, value).to_py_result()
    }
}

impl Peripheral for PyGpioPort {
    fn read(&self, offset: u32) -> MemoryAccessResult<u32> {
        self.gpio.read(offset)
    }

    fn write(&self, offset: u32, value: u32) -> MemoryAccessResult<()> {
        self.gpio.write(offset, value)
    }
}
