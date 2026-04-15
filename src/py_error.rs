use std::fmt;

use pyo3::prelude::*;

use emulator::{
    Breakpoint, cpu::Exception, instructions::InstructionConversionError,
    memory::MemoryAccessError, prelude::ExecutionError,
};

#[allow(unused_imports)]
pub use pyo3::exceptions::*;
use thiserror::Error;

#[derive(Debug, Error, derive_more::From)]
#[pyclass(name = "ExecutionError", extends = PyException, subclass)]
pub(crate) struct PyExecutionError {
    error: ExecutionError,
}

impl fmt::Display for PyExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[pymethods]
impl PyExecutionError {
    #[new]
    fn __new__() -> PyResult<Self> {
        Err(PyErr::new::<PyRuntimeError, _>(
            "PyExecutionError.__new__ should not be used",
        ))
    }

    fn is_breakpoint(&self) -> bool {
        matches!(self.error, ExecutionError::Breakpoint(_))
    }

    fn is_memory_access(&self) -> bool {
        matches!(self.error, ExecutionError::MemoryAccessError(_))
    }

    fn is_instruction_conversion(&self) -> bool {
        matches!(self.error, ExecutionError::InstructionConversionError(_))
    }

    fn is_exception(&self) -> bool {
        matches!(self.error, ExecutionError::Exception(_))
    }
}

impl From<PyExecutionError> for PyErr {
    fn from(value: PyExecutionError) -> Self {
        Python::attach(|py| {
            PyErr::from_value(
                Py::new(py, value)
                    .expect("Failed to create PyExecutionError")
                    .into_bound(py)
                    .into_any(),
            )
        })
    }
}

macro_rules! impl_to_py_execution_error {
    ($ty:ty) => {
        impl From<$ty> for PyExecutionError {
            fn from(value: $ty) -> Self {
                Self {
                    error: value.into(),
                }
            }
        }
    };
}

impl_to_py_execution_error!(MemoryAccessError);
impl_to_py_execution_error!(Exception);
impl_to_py_execution_error!(InstructionConversionError);
impl_to_py_execution_error!(Breakpoint);

/// Map `PyExecutionError`
/// Map a `Result<T, Into<PyExecutionError>>` to a `Result<T, PyExecutionError>`.
/// Useful for converting errors to be less verbose.
#[macro_export]
macro_rules! mpe {
    ($expr:expr) => {{ ($expr).map_err(crate::py_error::PyExecutionError::from) }};
}

#[pymodule]
pub(crate) fn py_error(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyExecutionError>()?;

    Ok(())
}
