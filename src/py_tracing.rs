use pyo3::prelude::*;

use crate::py_app_dir::py_app_dir_root_raw;

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
    (layer { with ansi : $expr:expr $(,)? }$(,)?) => {{ ::tracing_subscriber::fmt::layer().with_ansi($expr) }};

    (json: {file: $file:ident, filter: $filter:expr $(,)?}$(,)?) => {{
        subscriber_layer!(layer{ with ansi: false })
            .json()
            .with_writer($file)
            .with_filter($filter)
    }};
}

#[pyfunction(name = "init_tracing")]
fn py_init_tracing() -> PyResult<()> {
    use std::fs::create_dir_all;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing::Level;
    use tracing_subscriber::{Layer, filter, layer::SubscriberExt};

    {
        use std::sync::atomic::{AtomicBool, Ordering};

        static TOKEN: AtomicBool = AtomicBool::new(true);

        if !TOKEN
            .compare_exchange(
                true,
                false,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok_and(|b| b)
        {
            tracing::warn!("Tracing already initialized");
            return Ok(());
        }
    }

    let root = py_app_dir_root_raw()?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let session_folder = format!("session_{}", timestamp);
    let log_root = root.join("logs").join(session_folder);

    create_dir_all(&log_root)?;

    let err_file = create_file!(log_root, "log-error.log")?;
    let debug_file = create_file!(log_root, "log-debug.log")?;
    let trace_file = create_file!(log_root, "log-trace.log")?;

    let subscriber = tracing_subscriber::Registry::default()
        .with(subscriber_layer!(layer{ with ansi: true }).compact())
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

    tracing::info!("Tracing initialized");

    Ok(())
}

#[pymodule]
pub(crate) fn py_tracing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_init_tracing, m)?)?;

    Ok(())
}
