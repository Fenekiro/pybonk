pub mod bonk2map;
pub mod map_data_handler;
pub mod object_type;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::maps::bonk2map::{
    Body, BodyForce, BodyForceZoneProperties, BodyShape, Bonk2Map, CapZone, Fixture, Joint,
    JointProperties, MapMetadata, MapPhysics, MapProperties, Shape, Spawn,
};

pub fn register_python_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Bonk2Map>()?;
    m.add_class::<MapMetadata>()?;
    m.add_class::<MapProperties>()?;
    m.add_class::<MapPhysics>()?;
    m.add_class::<Spawn>()?;
    m.add_class::<CapZone>()?;
    m.add_class::<Body>()?;
    m.add_class::<BodyForce>()?;
    m.add_class::<BodyForceZoneProperties>()?;
    m.add_class::<BodyShape>()?;
    m.add_class::<Fixture>()?;
    m.add_class::<Shape>()?;
    m.add_class::<JointProperties>()?;
    m.add_class::<Joint>()?;

    m.add_function(wrap_pyfunction!(map_data_handler::py_get_blank_map, m)?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_get_new_body, m)?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_get_new_fixture, m)?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_get_new_box_shape, m)?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_get_new_circle_shape,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_get_new_poly_shape,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_get_new_chain_shape,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_get_new_revolute_joint,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_get_new_distance_joint,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_get_new_lpj_joint, m)?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_get_new_lsj_joint, m)?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_get_new_gear_joint,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_get_new_cap_zone, m)?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_get_new_spawn, m)?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_pson_encode, m)?)?;
    m.add_function(wrap_pyfunction!(map_data_handler::py_pson_decode, m)?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_decode_from_old_b64_string,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_decode_from_b64_string,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        map_data_handler::py_encode_to_b64_string,
        m
    )?)?;

    Ok(())
}
