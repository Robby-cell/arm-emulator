use pyo3::prelude::*;

use std::{fmt, sync::Arc};

use emulator::{
    Emulator,
    cpu::Cpu,
    memory::{Bus, Endian, MemoryMappedPeripheral},
};

use crate::{
    error::ToPyResult, py_memory::RamSize, py_peripheral::PyPeripheral,
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
    #[pyo3(signature = (ram_size = RamSize(0)))]
    fn new(ram_size: RamSize) -> Self {
        Self {
            emulator: emulator_with_ram_size(ram_size.0),
        }
    }

    fn read32(&self, addr: u32) -> PyResult<u32> {
        self.emulator.read32(addr).to_py_result()
    }

    fn write32(&mut self, addr: u32, value: u32) -> PyResult<()> {
        self.emulator.write32(addr, value).to_py_result()
    }

    fn read_byte(&self, addr: u32) -> PyResult<u8> {
        self.emulator.read_byte(addr).to_py_result()
    }

    fn write_byte(&mut self, addr: u32, value: u8) -> PyResult<()> {
        self.emulator.write_byte(addr, value).to_py_result()
    }

    fn use_little_endian(&mut self) {
        self.emulator.use_little_endian()
    }

    fn use_big_endian(&mut self) {
        self.emulator.use_big_endian()
    }

    fn max_address(&self) -> u32 {
        self.emulator.max_address()
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
                mapped_peripheral.into(),
            )?),
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
