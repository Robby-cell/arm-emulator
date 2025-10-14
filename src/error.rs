use pyo3::{PyErrArguments, PyTypeInfo, prelude::*};

use emulator::memory::MemoryAccessResult;

#[allow(unused_imports)]
pub use pyo3::exceptions::*;

#[allow(dead_code)]
#[inline(always)]
pub fn create_py_err<T, A>(msg: A) -> PyErr
where
    T: PyTypeInfo,
    A: PyErrArguments + Send + Sync + 'static,
{
    PyErr::new::<T, _>(msg)
}

pub trait ToPyResult<T> {
    fn to_py_result(self) -> PyResult<T>;
}

impl<T> ToPyResult<T> for MemoryAccessResult<T> {
    fn to_py_result(self) -> PyResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                Err(create_py_err::<PyRuntimeError, _>(format!("{e:?}")))
            }
        }
    }
}
