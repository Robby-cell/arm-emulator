use std::{fmt, path::PathBuf};

use app_dirs2::{AppDataType, AppDirsError, AppInfo, get_app_root};
use pyo3::{exceptions::PyException, prelude::*};
use thiserror::Error;

const APP_INFO: AppInfo = AppInfo {
    name: "Arm Emulator",
    author: "Robert Williamson",
};

pub(crate) fn app_dir_root() -> Result<PathBuf, AppDirsError> {
    get_app_root(AppDataType::UserConfig, &APP_INFO)
}

#[pyclass(name = "AppDirsError", extends = PyException)]
#[derive(Debug, Error, derive_more::From)]
pub struct PyAppDirsError {
    error: AppDirsError,
}

impl fmt::Display for PyAppDirsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<PyAppDirsError> for PyErr {
    fn from(value: PyAppDirsError) -> Self {
        Python::attach(|py| match Bound::new(py, value) {
            Ok(bound) => PyErr::from_value(bound.into_any()),
            Err(e) => e,
        })
    }
}

pub(crate) fn py_app_dir_root_raw() -> PyResult<PathBuf> {
    match app_dir_root() {
        Ok(path) => Ok(path),
        Err(e) => Err(Into::<PyAppDirsError>::into(e).into()),
    }
}

#[pyfunction(name = "app_dir_root")]
pub(crate) fn py_app_dir_root() -> PyResult<PathBuf> {
    py_app_dir_root_raw()
}

#[pymodule]
pub(crate) fn py_app_dir(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_app_dir_root, m)?)?;

    Ok(())
}
