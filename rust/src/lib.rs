pub mod byte_parsers;
pub mod avatars;
pub mod maps;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn rust(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    maps::register_python_bindings(m)?;
    avatars::register_python_bindings(m)?;
    Ok(())
}
