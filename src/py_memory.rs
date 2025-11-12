use pyo3::prelude::*;

use emulator::memory::Bus;

#[pyclass]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RamSize(pub u32);

#[pymethods]
impl RamSize {
    #[new]
    fn new(size: u32) -> Self {
        Self(size)
    }
}

/// Python representation of memory regions.
/// Can't export constants directly, so use an enum.
#[allow(non_camel_case_types)] // For the enum variant names
#[pyclass(name = "MemoryRegion")]
#[repr(u32)]
pub enum PyMemoryRegion {
    CODE_BEGIN = Bus::CODE_BEGIN,
    CODE_END = Bus::CODE_END,

    SRAM_BEGIN = Bus::SRAM_BEGIN,
    SRAM_END = Bus::SRAM_END,

    PERIPHERAL_BEGIN = Bus::PERIPHERAL_BEGIN,
    PERIPHERAL_END = Bus::PERIPHERAL_END,

    EXTERNAL_BEGIN = Bus::EXTERNAL_BEGIN,
    EXTERNAL_END = Bus::EXTERNAL_END,
}

#[pymodule]
pub(crate) fn py_memory(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMemoryRegion>()?;
    m.add_class::<RamSize>()?;

    Ok(())
}
