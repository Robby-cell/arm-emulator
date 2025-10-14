use pyo3::prelude::*;

use emulator::memory::{
    MemoryAccessError, MemoryAccessResult, Peripheral,
};

#[pyclass(name = "Peripheral")]
pub(crate) struct PyPeripheral {
    obj: Py<PyAny>,
}

impl PyPeripheral {
    /// Name of the read method on the python class. For correctness.
    const READ: &str = "read";

    /// Name of the write method on the python class. For correctness.
    const WRITE: &str = "write";
}

impl Clone for PyPeripheral {
    fn clone(&self) -> Self {
        Self {
            obj: Python::attach(|py| self.obj.clone_ref(py)),
        }
    }
}

impl Peripheral for PyPeripheral {
    fn read(&self, offset: u32) -> MemoryAccessResult<u32> {
        let result = Python::attach(|py| {
            let result = self.obj.call_method1(py, Self::READ, (offset,));
            match result {
                Ok(result) => {
                    Ok(result.extract::<u32>(py).map_err(|_| {
                        MemoryAccessError::InvalidReadPermission {
                            addr: offset,
                        }
                    })?)
                }
                Err(e) => {
                    tracing::error!(
                        "Could not extract u32 from the type returned by the PyPeripheral inner: {e:?}"
                    );
                    Err(MemoryAccessError::InvalidPeripheralRead {
                        offset,
                    })
                }
            }
        });
        result
    }

    fn write(&self, offset: u32, value: u32) -> MemoryAccessResult<()> {
        _ = Python::attach(|py| {
            self.obj.call_method1(py, Self::WRITE, (offset, value))
        })
        .map_err(|_| {
            MemoryAccessError::InvalidPeripheralWrite { offset }
        })?;
        Ok(())
    }
}

impl PyPeripheral {
    const PERIPHERAL_METHODS: &[&str] = &[Self::READ, Self::WRITE];

    pub(crate) fn verify_valid_object(
        obj: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        for &attr in Self::PERIPHERAL_METHODS {
            _ = obj.getattr(attr)?;
        }
        Ok(())
    }
}

#[pymethods]
impl PyPeripheral {
    #[new]
    pub(crate) fn __new__(obj: Bound<'_, PyAny>) -> PyResult<Self> {
        PyPeripheral::verify_valid_object(obj.clone())?;

        Ok(Self { obj: obj.into() })
    }
}
