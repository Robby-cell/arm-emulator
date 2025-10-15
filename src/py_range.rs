use pyo3::prelude::*;

use std::ops::RangeInclusive;

/// Inclusive range object wrapper for python, to allow python to map the [Peripheral]
///
/// # Example
///
/// ```py
/// mapped_region = RangeInclusive32(0, 15) # maps 0 to 15 (inclusive)
/// ```
#[pyclass(name = "RangeInclusiveU32")]
pub struct PyRangeInclusiveU32(RangeInclusive<u32>);

impl PyRangeInclusiveU32 {
    pub fn range(&self) -> RangeInclusive<u32> {
        self.0.clone()
    }
}

#[pymethods]
impl PyRangeInclusiveU32 {
    #[new]
    fn new(start: u32, end: u32) -> Self {
        Self(start..=end)
    }
}

#[pymodule]
pub fn py_range(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRangeInclusiveU32>()?;
    Ok(())
}
