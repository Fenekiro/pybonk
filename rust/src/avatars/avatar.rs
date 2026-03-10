use pyo3::BoundObject;
use crate::avatars::avatar_data_handler::decode_avatar;
use crate::avatars::avatar_layer::AvatarLayer;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use pythonize::pythonize;

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Avatar {
    pub layers: Vec<AvatarLayer>,

    #[serde(rename = "bc")]
    pub base_color: i32,
}

impl Default for Avatar {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            base_color: 0,
        }
    }
}

impl Avatar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_encoded_string(encoded_string: &str) -> Result<Self, String> {
        decode_avatar(encoded_string)
    }

    pub fn load_from_json(&mut self, avatar_dict: Value) -> Result<&mut Self, serde_json::Error> {
        let parsed: Avatar = serde_json::from_value(avatar_dict)?;

        self.base_color = parsed.base_color;
        self.layers = parsed.layers;

        Ok(self)
    }

    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[pymethods]
impl Avatar {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    #[staticmethod]
    pub fn from_encoded(encoded_string: &str) -> PyResult<Self> {
        Self::from_encoded_string(encoded_string).map_err(PyValueError::new_err)
    }

    #[staticmethod]
    pub fn from_json_string(avatar_json: &str) -> PyResult<Self> {
        serde_json::from_str(avatar_json).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    pub fn load_from_json_string(&mut self, avatar_json: &str) -> PyResult<()> {
        let avatar_dict: Value =
            serde_json::from_str(avatar_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        self.load_from_json(avatar_dict)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(())
    }

    pub fn to_json_string(&self) -> PyResult<String> {
        serde_json::to_string(self).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let obj = pythonize(py, &self)?;
        let bound_obj = obj.into_bound();
        let dict = bound_obj
            .downcast_into::<PyDict>()
            .map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>("Result was not a dict"))?;

        Ok(dict)
    }
}
