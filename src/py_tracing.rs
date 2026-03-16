use std::{fmt, path::PathBuf};

use app_dirs2::{AppDataType, AppDirsError, AppInfo, get_app_root};
use pyo3::{exceptions::PyException, prelude::*};
use thiserror::Error;

const APP_INFO: AppInfo = AppInfo {
    name: "arm_emulator_rs",
    author: "Robert Williamson",
};

pub(crate) fn app_dir_root() -> Result<PathBuf, AppDirsError> {
    get_app_root(AppDataType::UserConfig, &APP_INFO)
}

pub(crate) fn init_tracing() -> PyResult<()> {
    use std::fs::OpenOptions;
    use tracing::Level;
    use tracing_subscriber::{Layer, filter, fmt, layer::SubscriberExt};

    let root = py_app_dir_root()?;
    let log_root = root.join("logs");

    let err_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_root.join("log-error.log"))?;

    let debug_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_root.join("log-debug.log"))?;

    let trace_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_root.join("log-trace.log"))?;

    let subscriber = tracing_subscriber::Registry::default()
        .with(fmt::layer().compact().with_ansi(true))
        .with(
            fmt::layer()
                .with_ansi(false)
                .json()
                .with_writer(err_file)
                .with_filter(filter::LevelFilter::from_level(
                    Level::ERROR,
                )),
        )
        .with(
            fmt::layer()
                .with_ansi(false)
                .json()
                .with_writer(debug_file)
                .with_filter(filter::LevelFilter::from_level(
                    Level::DEBUG,
                )),
        )
        .with(
            fmt::layer()
                .with_ansi(false)
                .json()
                .with_writer(trace_file)
                .with_filter(filter::LevelFilter::from_level(
                    Level::TRACE,
                )),
        );

    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set global default subscriber");

    Ok(())
}

#[pyclass(name = "AppDirsError", extends = PyException)]
#[derive(Debug, Error)]
pub struct PyAppDirsError {
    error: AppDirsError,
}

impl fmt::Display for PyAppDirsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<AppDirsError> for PyAppDirsError {
    fn from(value: AppDirsError) -> Self {
        Self { error: value }
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

#[pyfunction(name = "app_dir_root")]
fn py_app_dir_root() -> PyResult<PathBuf> {
    match app_dir_root() {
        Ok(path) => Ok(path),
        Err(e) => Err(Into::<PyAppDirsError>::into(e).into()),
    }
}

#[pymodule]
pub(crate) fn py_tracing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_app_dir_root, m)?)?;

    Ok(())
}
