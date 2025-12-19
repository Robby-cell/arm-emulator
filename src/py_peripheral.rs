use pyo3::prelude::*;

use emulator::memory::{
    MemoryAccessError, MemoryAccessResult, Peripheral,
};

#[pyclass(name = "Peripheral", subclass)]
pub struct PyPeripheral {
    obj: Py<PyAny>,
}

impl PyPeripheral {
    /// Name of the read method on the python class. For correctness.
    const READ32: &str = "read32";

    /// Name of the write method on the python class. For correctness.
    const WRITE32: &str = "write32";

    /// Name of the read method on the python class.
    const READ_BYTE: &str = "read_byte";

    /// Name of the write method on the python class.
    const WRITE_BYTE: &str = "write_byte";

    const RESET: &str = "reset";
}

impl Clone for PyPeripheral {
    fn clone(&self) -> Self {
        Self {
            obj: Python::attach(|py| self.obj.clone_ref(py)),
        }
    }
}

impl Peripheral for PyPeripheral {
    fn read32(&self, offset: u32) -> MemoryAccessResult<u32> {
        let result = Python::attach(|py| {
            let result =
                self.obj.call_method1(py, Self::READ32, (offset,));
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

    fn write32(&self, offset: u32, value: u32) -> MemoryAccessResult<()> {
        Python::attach(|py| {
            self.obj.call_method1(py, Self::WRITE32, (offset, value))
        })
        .map_err(|_| {
            MemoryAccessError::InvalidPeripheralWrite { offset }
        })?;
        Ok(())
    }

    fn read_byte(&self, offset: u32) -> MemoryAccessResult<u8> {
        let result = Python::attach(|py| {
            let result =
                self.obj.call_method1(py, Self::READ_BYTE, (offset,));
            match result {
                Ok(result) => {
                    Ok(result.extract::<u8>(py).map_err(|_| {
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

    fn write_byte(
        &self,
        offset: u32,
        value: u8,
    ) -> MemoryAccessResult<()> {
        Python::attach(|py| {
            self.obj.call_method1(py, Self::WRITE_BYTE, (offset, value))
        })
        .map_err(|_| {
            MemoryAccessError::InvalidPeripheralWrite { offset }
        })?;
        Ok(())
    }

    fn reset(&self) {
        match Python::attach(|py| {
            self.obj.call_method1(py, Self::RESET, ())
        }) {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("PyPeripheral::reset: {e}");
            }
        }
    }
}

impl PyPeripheral {
    const PERIPHERAL_METHODS: &[&str] = &[
        Self::READ32,
        Self::WRITE32,
        Self::READ_BYTE,
        Self::WRITE_BYTE,
        Self::RESET,
    ];

    fn verify_valid_object(obj: Bound<'_, PyAny>) -> PyResult<()> {
        for &attr in Self::PERIPHERAL_METHODS {
            _ = obj.getattr(attr)?;
        }
        Ok(())
    }
}

#[pymethods]
impl PyPeripheral {
    #[new]
    pub fn new(obj: Bound<'_, PyAny>) -> PyResult<Self> {
        PyPeripheral::verify_valid_object(obj.clone())?;

        Ok(Self { obj: obj.into() })
    }
}

#[pymodule]
pub(crate) fn py_peripheral(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPeripheral>()?;
    Ok(())
}
