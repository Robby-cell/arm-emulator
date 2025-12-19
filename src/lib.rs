use std::fmt;

use emulator::prelude::ExecutionError;
use pyo3::{exceptions::PyException, prelude::*};
use thiserror::Error;

mod error;
mod py_emulator;
mod py_gpio_port;
mod py_memory;
mod py_peripheral;
mod py_range;

fn init_tracing() -> PyResult<()> {
    use std::fs::OpenOptions;
    use tracing::Level;
    use tracing_subscriber::{Layer, filter, fmt, layer::SubscriberExt};

    let err_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log-error.log")?;

    let debug_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log-debug.log")?;

    let trace_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log-trace.log")?;

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

#[derive(Debug, Error)]
#[pyclass(name = "ExecutionError", extends = PyException)]
struct PyExecutionError {
    error: ExecutionError,
}

impl fmt::Display for PyExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<ExecutionError> for PyExecutionError {
    fn from(error: ExecutionError) -> Self {
        PyExecutionError { error }
    }
}

#[pymethods]
impl PyExecutionError {
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

/// A Python module implemented in Rust.
#[pymodule]
fn arm_emulator_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    init_tracing()?;

    tracing::info!("Initializing arm_emulator_rs");

    {
        py_emulator::py_emulator(m)?;
    }

    {
        py_memory::py_memory(m)?;
    }

    {
        py_peripheral::py_peripheral(m)?;
        py_gpio_port::py_gpio_port(m)?;
    }

    {
        py_range::py_range(m)?;

        m.add_class::<PyExecutionError>()?;
    }

    tracing::info!("Initialized arm_emulator_rs");

    Ok(())
}
