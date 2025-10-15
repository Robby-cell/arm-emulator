use pyo3::prelude::*;

use std::{fmt, sync::Arc};

use emulator::{Emulator, memory::MemoryMappedPeripheral};

use crate::{
    error::ToPyResult, py_peripheral::PyPeripheral,
    py_range::PyRangeInclusiveU32,
};

/// A python wrapper around the `Emulator`
#[pyclass(name = "Emulator", str)]
pub(crate) struct PyEmulator {
    emulator: Emulator,
}

impl fmt::Display for PyEmulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[pymethods]
impl PyEmulator {
    #[new]
    pub(crate) fn __new__(ram_size: u32) -> Self {
        Self {
            emulator: Emulator::with_ram_size(ram_size),
        }
    }

    pub(crate) fn read32(&self, addr: u32) -> PyResult<u32> {
        self.emulator.read32(addr).to_py_result()
    }

    pub(crate) fn write32(
        &mut self,
        addr: u32,
        value: u32,
    ) -> PyResult<()> {
        self.emulator.write32(addr, value).to_py_result()
    }

    pub(crate) fn execute_until_breakpoint(&mut self) -> PyResult<()> {
        Ok(())
    }

    pub(crate) fn execute(&mut self) -> PyResult<()> {
        Ok(())
    }

    pub(crate) fn step(&mut self) -> PyResult<()> {
        Ok(())
    }

    pub(crate) fn add_peripheral(
        &mut self,
        range: &PyRangeInclusiveU32,
        mapped_peripheral: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.emulator.add_peripheral(MemoryMappedPeripheral {
            range: range.range(),
            peripheral: Arc::new(PyPeripheral::__new__(
                mapped_peripheral.into()
            ).expect("Failed to create peripheral. Exception thrown, missing required methods")),
        });
        Ok(())
    }
}

impl fmt::Debug for PyEmulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.emulator)
    }
}

#[pyfunction()]
pub(crate) fn emulator_with_ram_size(ram_size: u32) -> PyEmulator {
    PyEmulator {
        emulator: Emulator::with_ram_size(ram_size),
    }
}
