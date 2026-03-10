pub mod avatar;
pub mod avatar_data_handler;
pub mod avatar_layer;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::avatars::avatar::Avatar;
use crate::avatars::avatar_layer::AvatarLayer;

pub fn register_python_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Avatar>()?;
    m.add_class::<AvatarLayer>()?;
    m.add_function(wrap_pyfunction!(avatar_data_handler::py_decode_avatar, m)?)?;
    Ok(())
}
