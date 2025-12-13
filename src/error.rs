use pyo3::{PyErrArguments, PyTypeInfo, prelude::*};

use emulator::{
    Breakpoint, cpu::Exception, instructions::InstructionConversionError,
    memory::MemoryAccessError, prelude::ExecutionError,
};

#[allow(unused_imports)]
pub use pyo3::exceptions::*;

use crate::PyExecutionError;

#[allow(dead_code)]
#[inline(always)]
pub fn create_py_err<T, A>(msg: A) -> PyErr
where
    T: PyTypeInfo,
    A: PyErrArguments + Send + Sync + 'static,
{
    PyErr::new::<T, _>(msg)
}

pub trait ToPyError {
    fn to_py_error(self) -> PyErr;
}

pub trait ToPyResult<T> {
    fn to_py_result(self) -> PyResult<T>;
}

impl ToPyError for PyExecutionError {
    fn to_py_error(self) -> PyErr {
        create_py_err::<PyRuntimeError, _>(format!("{self}"))
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
