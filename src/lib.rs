use pyo3::prelude::*;

mod py_emulator;
mod py_error;
mod py_gpio_port;
mod py_memory;
mod py_peripheral;
mod py_range;
mod py_tracing;

/// A Python module implemented in Rust.
#[pymodule]
fn arm_emulator_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    tracing::info!("Initializing arm_emulator_rs");

    {
        py_emulator::py_emulator(m)?;
    }

    {
        py_error::py_error(m)?;
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
    }

    {
        py_tracing::py_tracing(m)?;
    }

    tracing::info!("Initialized arm_emulator_rs");

    Ok(())
}
