use pyo3::{exceptions::PyValueError, prelude::*};

use std::{collections::HashMap, fmt, sync::Arc};

use emulator::{
    Emulator,
    cpu::{Cpu, ExitStatus},
    memory::{Bus, Endian, MemoryMappedPeripheral},
};

use crate::{
    mpe, py_peripheral::PyPeripheral, py_range::PyRangeInclusiveU32,
};

/// A python wrapper around the `Emulator`
#[pyclass(name = "Emulator", str)]
struct PyEmulator {
    emulator: Emulator,
    py_handles: Vec<Py<PyAny>>,
}

impl fmt::Display for PyEmulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[pymethods]
impl PyEmulator {
    #[new]
    #[pyo3(signature = (*, code_size = 0, sram_size = 0, external_size = 0))]
    fn new(code_size: u32, sram_size: u32, external_size: u32) -> Self {
        Self {
            emulator: emulator_with_ram_size(
                code_size,
                sram_size,
                external_size,
            ),
            py_handles: Vec::new(),
        }
    }

    fn get_exit_code(&self) -> Option<i32> {
        self.emulator
            .get_exit_status()
            .map(|ExitStatus { exit_code }| exit_code)
    }

    #[getter]
    fn registers(&self) -> Vec<u32> {
        self.emulator.cpu.registers.to_vec()
    }

    fn get_register(&self, index: u32) -> PyResult<u32> {
        if index >= self.emulator.cpu.registers.len() as u32 {
            Err(PyValueError::new_err(format!(
                "Register index out of range: {}",
                index
            )))
        } else {
            Ok(self.emulator.cpu.register(index as _))
        }
    }

    fn set_register(&mut self, index: u32, value: u32) -> PyResult<()> {
        if index >= self.emulator.cpu.registers.len() as u32 {
            Err(PyValueError::new_err(format!(
                "Register index out of range: {}",
                index
            )))
        } else {
            self.emulator.cpu.set_register(index as _, value);
            Ok(())
        }
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

    #[pyo3(signature = (code, /, *, sram = None, external = None))]
    fn load_program(
        &mut self,
        code: &[u8],
        sram: Option<&[u8]>,
        external: Option<&[u8]>,
    ) {
        // Signature allows:
        // load_program(code), load_program(code, sram=sram, external=external)
        // Only code is positional, the rest are keyword-only, with defaults to `None`
        self.emulator.load_program(code, sram, external);
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

    fn complete_reset(&mut self) {
        self.emulator.reset();
        self.py_handles.clear(); // Clears the Python object references
    }

    fn reset_cpu(&mut self) {
        self.emulator.reset_cpu();
    }

    fn is_halted(&self) -> bool {
        self.emulator.is_halted()
    }

    fn is_finished(&self) -> bool {
        self.emulator.is_finished()
    }

    fn read32(&self, addr: u32) -> PyResult<u32> {
        Ok(mpe!(self.emulator.read32(addr))?)
    }

    fn write32(&mut self, addr: u32, value: u32) -> PyResult<()> {
        mpe!(self.emulator.write32(addr, value))?;
        Ok(())
    }

    pub fn try_read_chunk(
        &self,
        addr: u32,
        size: u32,
    ) -> PyResult<Vec<Option<u8>>> {
        let mut result = Vec::with_capacity(size as usize);
        for i in 0..size {
            let addr = addr.wrapping_add(i);
            match self.emulator.read_byte(addr) {
                Ok(byte) => result.push(Some(byte)),
                Err(_) => result.push(None), // None indicates "?? / unmapped"
            }
        }
        Ok(result)
    }

    fn try_read_byte(&self, addr: u32) -> Option<u8> {
        self.emulator.read_byte(addr).ok()
    }

    fn read_byte(&self, addr: u32) -> PyResult<u8> {
        Ok(mpe!(self.emulator.read_byte(addr))?)
    }

    fn write_byte(&mut self, addr: u32, value: u8) -> PyResult<()> {
        mpe!(self.emulator.write_byte(addr, value))?;
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
        mpe!(self.emulator.step_over_breakpoint())?;
        Ok(())
    }

    fn execute(&mut self) -> PyResult<()> {
        mpe!(self.emulator.execute())?;
        Ok(())
    }

    fn step(&mut self) -> PyResult<()> {
        mpe!(self.emulator.step())?;
        Ok(())
    }

    pub fn add_breakpoint_at(&mut self, address: u32) -> PyResult<()> {
        mpe!(self.emulator.add_breakpoint_at(address))?;
        Ok(())
    }

    pub fn remove_breakpoint_at(&mut self, address: u32) -> PyResult<()> {
        mpe!(self.emulator.remove_breakpoint_at(address))?;
        Ok(())
    }

    pub fn restore_instruction_at(
        &mut self,
        address: u32,
    ) -> PyResult<()> {
        mpe!(self.emulator.restore_instruction_at(address))?;
        Ok(())
    }

    fn add_peripheral(
        &mut self,
        range: &PyRangeInclusiveU32,
        mapped_peripheral: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Create the Rust wrapper. Pass the Bound reference directly.
        // Make sure PyPeripheral::new accepts Bound<'_, PyAny>
        let wrapper =
            Arc::new(PyPeripheral::new(mapped_peripheral.clone())?);

        self.emulator.add_peripheral(MemoryMappedPeripheral {
            range: range.range(),
            peripheral: wrapper,
        });

        // Store the Python reference.
        // .clone().unbind() creates an independent Py<PyAny> we can store safely.
        self.py_handles.push(mapped_peripheral.clone().unbind());

        Ok(())
    }

    #[getter]
    fn peripherals<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Vec<Bound<'py, PyAny>>> {
        let mut list = Vec::new();

        for handle in &self.py_handles {
            // To get a Bound object back, use handle.bind(py)
            // handle is a Py<PyAny>, so .bind(py) returns Bound<'py, PyAny>
            list.push(handle.bind(py).clone());
        }

        Ok(list)
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
