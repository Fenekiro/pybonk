use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AvatarLayer {
    pub id: i32,
    pub scale: f64,
    pub angle: f64,
    pub x: f64,
    pub y: f64,
    pub flip_x: bool,
    pub flip_y: bool,
    pub color: i32,
}

impl Default for AvatarLayer {
    fn default() -> Self {
        Self {
            id: 0,
            scale: 1.0,
            angle: 0.0,
            x: 0.0,
            y: 0.0,
            flip_x: false,
            flip_y: false,
            color: 0,
        }
    }
}

#[pymethods]
impl AvatarLayer {
    #[new]
    #[pyo3(signature = (id=0, scale=1.0, angle=0.0, x=0.0, y=0.0, flip_x=false, flip_y=false, color=0))]
    pub fn new(
        id: i32,
        scale: f64,
        angle: f64,
        x: f64,
        y: f64,
        flip_x: bool,
        flip_y: bool,
        color: i32,
    ) -> Self {
        Self {
            id,
            scale,
            angle,
            x,
            y,
            flip_x,
            flip_y,
            color,
        }
    }
}
