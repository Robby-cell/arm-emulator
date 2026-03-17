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

macro_rules! create_file {
    ($log_root:ident, $path:expr $(,)?) => {{
        {
            ::std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .write(true)
                .open(($log_root).join($path))
        }
    }};
}

macro_rules! subscriber_layer {
    (basic with ansi) => {{ ::tracing_subscriber::fmt::layer().with_ansi(false) }};

    (json: {file: $file:ident, filter: $filter:expr $(,)?}$(,)?) => {{
        subscriber_layer!(basic with ansi)
            .json()
            .with_writer($file)
            .with_filter($filter)
    }};
}

#[pyfunction(name = "init_tracing")]
fn py_init_tracing() -> PyResult<()> {
    use std::fs::create_dir_all;
    use tracing::Level;
    use tracing_subscriber::{Layer, filter, fmt, layer::SubscriberExt};

    {
        static TOKEN: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);

        if TOKEN
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .is_ok()
        {
            tracing::warn!("Tracing already initialized");
            return Ok(());
        }
    }

    let root = py_app_dir_root_raw()?;
    let log_root = root.join("logs");

    create_dir_all(&log_root)?;

    let err_file = create_file!(log_root, "log-error.log")?;
    let debug_file = create_file!(log_root, "log-debug.log")?;
    let trace_file = create_file!(log_root, "log-trace.log")?;

    let subscriber = tracing_subscriber::Registry::default()
        .with(fmt::layer().compact().with_ansi(true))
        .with(subscriber_layer!(
            json: {
                file: err_file,
                filter: filter::LevelFilter::from_level(Level::ERROR),
            },
        ))
        .with(subscriber_layer!(
            json: {
                file: debug_file,
                filter: filter::LevelFilter::from_level(Level::DEBUG),
            },
        ))
        .with(subscriber_layer!(
            json: {
                file: trace_file,
                filter: filter::LevelFilter::from_level(Level::TRACE),
            },
        ));

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

fn py_app_dir_root_raw() -> PyResult<PathBuf> {
    match app_dir_root() {
        Ok(path) => Ok(path),
        Err(e) => Err(Into::<PyAppDirsError>::into(e).into()),
    }
}

#[pyfunction(name = "app_dir_root")]
fn py_app_dir_root() -> PyResult<PathBuf> {
    py_app_dir_root_raw()
}

#[pymodule]
pub(crate) fn py_tracing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_app_dir_root, m)?)?;
    m.add_function(wrap_pyfunction!(py_init_tracing, m)?)?;

    Ok(())
}
