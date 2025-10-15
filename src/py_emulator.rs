use pyo3::prelude::*;

use std::{fmt, sync::Arc};

use emulator::{
    Emulator,
    cpu::Cpu,
    memory::{Bus, Endian, MemoryMappedPeripheral},
};

use crate::{
    error::ToPyResult, py_peripheral::PyPeripheral,
    py_range::PyRangeInclusiveU32,
};

/// A python wrapper around the `Emulator`
#[pyclass(name = "Emulator", str)]
struct PyEmulator {
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
    fn new(ram_size: u32) -> Self {
        Self {
            emulator: emulator_with_ram_size(ram_size),
        }
    }

    fn read32(&self, addr: u32) -> PyResult<u32> {
        self.emulator.read32(addr).to_py_result()
    }

    fn write32(&mut self, addr: u32, value: u32) -> PyResult<()> {
        self.emulator.write32(addr, value).to_py_result()
    }

    fn execute_until_breakpoint(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn execute(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn step(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn add_peripheral(
        &mut self,
        range: &PyRangeInclusiveU32,
        mapped_peripheral: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.emulator.add_peripheral(MemoryMappedPeripheral {
            range: range.range(),
            peripheral: Arc::new(PyPeripheral::new(
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

fn emulator_with_ram_size(ram_size: u32) -> Emulator {
    Emulator::new(Cpu::new(), Bus::new(ram_size), Endian::Little)
}

#[pyfunction(name = "emulator_with_ram_size")]
fn py_emulator_with_ram_size(ram_size: u32) -> PyEmulator {
    PyEmulator {
        emulator: emulator_with_ram_size(ram_size),
    }
}

#[pymodule]
pub(crate) fn py_emulator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Classes
    m.add_class::<PyEmulator>()?;

    // Functions
    m.add_function(wrap_pyfunction!(py_emulator_with_ram_size, m)?)?;

    Ok(())
}
