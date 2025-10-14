use pyo3::prelude::*;

mod error;
mod py_emulator;
mod py_gpio_port;
mod py_peripheral;
mod py_range;

use py_emulator::{PyEmulator, emulator_with_ram_size};
use py_gpio_port::PyGpioPort;
use py_peripheral::PyPeripheral;
use py_range::PyRangeInclusiveU32;

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
        );

    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set global default subscriber");

    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn arm_emulator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    init_tracing()?;

    // Functions
    m.add_function(wrap_pyfunction!(emulator_with_ram_size, m)?)?;

    // Classes
    m.add_class::<PyEmulator>()?;
    m.add_class::<PyGpioPort>()?;
    m.add_class::<PyPeripheral>()?;
    m.add_class::<PyRangeInclusiveU32>()?;

    Ok(())
}
