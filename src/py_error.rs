use pyo3::prelude::*;

use emulator::{
    Breakpoint, cpu::Exception, instructions::InstructionConversionError,
    memory::MemoryAccessError, prelude::ExecutionError,
};

#[allow(unused_imports)]
pub use pyo3::exceptions::*;

use crate::PyExecutionError;

pub trait ToPyError {
    fn to_py_error(self) -> PyErr;
}

pub trait ToPyResult<T> {
    fn to_py_result(self) -> PyResult<T>;
}

impl ToPyError for PyExecutionError {
    fn to_py_error(self) -> PyErr {
        self.into()
    }
}

impl From<PyExecutionError> for PyErr {
    fn from(value: PyExecutionError) -> Self {
        Python::attach(|py| match Bound::new(py, value) {
            Ok(bound) => PyErr::from_value(bound.into_any()),
            Err(e) => e,
        })
    }
}

impl<T> ToPyResult<T> for Result<T, PyExecutionError> {
    fn to_py_result(self) -> PyResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => Err(e.to_py_error()),
        }
    }
}

pub trait ToPyExecutionError {
    fn to_py_execution_error(self) -> PyExecutionError;
}

pub trait ToPyExecutionResult<T> {
    fn to_py_execution_result(self) -> Result<T, PyExecutionError>;
}

macro_rules! impl_to_py_exec_error {
    ($type:ty) => {
        impl ToPyExecutionError for $type {
            fn to_py_execution_error(self) -> PyExecutionError {
                <Self as Into<ExecutionError>>::into(self).into()
            }
        }

        impl<T> ToPyExecutionResult<T> for Result<T, $type> {
            fn to_py_execution_result(
                self,
            ) -> Result<T, PyExecutionError> {
                match self {
                    Ok(value) => Ok(value),
                    Err(e) => Err(e.to_py_execution_error()),
                }
            }
        }
    };
}

impl_to_py_exec_error!(ExecutionError);
impl_to_py_exec_error!(MemoryAccessError);
impl_to_py_exec_error!(Exception);
impl_to_py_exec_error!(InstructionConversionError);
impl_to_py_exec_error!(Breakpoint);
