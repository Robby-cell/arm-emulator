use pyo3::prelude::*;

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

fn add_submodule<'py>(
    m: &Bound<'py, PyModule>,
    submodule_name: &str,
) -> PyResult<Bound<'py, PyModule>> {
    let submodule = PyModule::new(m.py(), submodule_name)?;
    m.add_submodule(&submodule)?;
    let sys = m.py().import("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item(
        &format!("{}.{}", m.name()?, submodule_name),
        &submodule,
    )?;
    Ok(submodule)
}

/// A Python module implemented in Rust.
#[pymodule]
fn arm_emulator_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    init_tracing()?;

    tracing::info!("Initializing arm_emulator_rs");

    {
        let emulator_m = add_submodule(m, "emulator")?;
        py_emulator::py_emulator(&emulator_m)?;
    }

    {
        let memory_m = add_submodule(m, "memory")?;
        py_memory::py_memory(&memory_m)?;
    }

    {
        let peripheral_m = add_submodule(m, "peripheral")?;
        py_peripheral::py_peripheral(&peripheral_m)?;
        py_gpio_port::py_gpio_port(&peripheral_m)?;
    }

    {
        py_range::py_range(m)?;
    }

    tracing::info!("Initialized arm_emulator_rs");

    Ok(())
}
