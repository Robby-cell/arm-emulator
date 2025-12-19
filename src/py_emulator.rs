use pyo3::prelude::*;

use std::{collections::HashMap, fmt, sync::Arc};

use emulator::{
    Emulator,
    cpu::Cpu,
    memory::{Bus, Endian, MemoryMappedPeripheral},
};

use crate::{
    error::{ToPyExecutionResult, ToPyResult},
    py_peripheral::PyPeripheral,
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
    #[pyo3(signature = (code_size = 0, sram_size = 0, external_size = 0))]
    fn new(code_size: u32, sram_size: u32, external_size: u32) -> Self {
        Self {
            emulator: emulator_with_ram_size(
                code_size,
                sram_size,
                external_size,
            ),
        }
    }

    #[getter]
    fn registers(&self) -> Vec<u32> {
        self.emulator.cpu.registers.to_vec()
    }

    #[getter]
    fn flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("N".to_string(), self.emulator.cpu.n());
        flags.insert("Z".to_string(), self.emulator.cpu.z());
        flags.insert("C".to_string(), self.emulator.cpu.c());
        flags.insert("V".to_string(), self.emulator.cpu.v());
        flags
    }

    fn load_code(&mut self, code: &[u8]) {
        self.emulator.load_code(code);
    }

    fn load_sram(&mut self, sram: &[u8]) {
        self.emulator.load_sram(sram);
    }

    fn load_external(&mut self, external: &[u8]) {
        self.emulator.load_external(external);
    }

    fn reset(&mut self) {
        self.emulator.reset();
    }

    fn read32(&self, addr: u32) -> PyResult<u32> {
        Ok(self
            .emulator
            .read32(addr)
            .to_py_execution_result()
            .to_py_result()?)
    }

    fn write32(&mut self, addr: u32, value: u32) -> PyResult<()> {
        self.emulator
            .write32(addr, value)
            .to_py_execution_result()
            .to_py_result()?;
        Ok(())
    }

    fn read_byte(&self, addr: u32) -> PyResult<u8> {
        Ok(self
            .emulator
            .read_byte(addr)
            .to_py_execution_result()
            .to_py_result()?)
    }

    fn write_byte(&mut self, addr: u32, value: u8) -> PyResult<()> {
        self.emulator
            .write_byte(addr, value)
            .to_py_execution_result()
            .to_py_result()?;
        Ok(())
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

    fn step_over_breakpoint(&mut self) -> PyResult<()> {
        self.emulator
            .step_over_breakpoint()
            .to_py_execution_result()
            .to_py_result()
    }

    fn execute(&mut self) -> PyResult<()> {
        self.emulator
            .execute()
            .to_py_execution_result()
            .to_py_result()
    }

    fn step(&mut self) -> PyResult<()> {
        self.emulator.step().to_py_execution_result().to_py_result()
    }

    pub fn add_breakpoint_at(&mut self, address: u32) -> PyResult<()> {
        self.emulator
            .add_breakpoint_at(address)
            .to_py_execution_result()
            .to_py_result()
    }

    pub fn restore_instruction_at(
        &mut self,
        address: u32,
    ) -> PyResult<()> {
        self.emulator
            .restore_instruction_at(address)
            .to_py_execution_result()
            .to_py_result()
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

fn emulator_with_ram_size(
    code_size: u32,
    sram_size: u32,
    external_size: u32,
) -> Emulator {
    Emulator::new(
        Cpu::new(),
        Bus::new(code_size, sram_size, external_size),
        Endian::Little,
    )
}

#[pymodule]
pub(crate) fn py_emulator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Classes
    m.add_class::<PyEmulator>()?;

    // Functions

    Ok(())
}
