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
pub(crate) struct PyRangeInclusiveU32(RangeInclusive<u32>);

impl PyRangeInclusiveU32 {
    pub(crate) fn range(&self) -> RangeInclusive<u32> {
        self.0.clone()
    }
}

#[pymethods]
impl PyRangeInclusiveU32 {
    #[new]
    pub(crate) fn __new__(start: u32, end: u32) -> Self {
        Self(start..=end)
    }
}
