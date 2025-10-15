use pyo3::prelude::*;

mod error;
mod py_emulator;
mod py_gpio_port;
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

    {
        let emulator_m = PyModule::new(m.py(), "emulator")?;
        py_emulator::py_emulator(&emulator_m)?;
        m.add_submodule(&emulator_m)?;
    }

    {
        let peripheral_m = PyModule::new(m.py(), "peripheral")?;
        py_peripheral::py_peripheral(&peripheral_m)?;
        py_gpio_port::py_gpio_port(&peripheral_m)?;
        m.add_submodule(&peripheral_m)?;
    }

    {
        py_range::py_range(m)?;
    }

    Ok(())
}
