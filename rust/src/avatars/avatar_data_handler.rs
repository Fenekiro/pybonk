use crate::avatars::avatar::Avatar;
use crate::avatars::avatar_layer::AvatarLayer;
use crate::byte_parsers::byte_buffer::ByteBuffer;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use percent_encoding::percent_decode_str;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn decode_layer(buffer: &mut ByteBuffer) -> Result<Option<AvatarLayer>, String> {
    if buffer.read_byte() == 10 {
        if buffer.read_byte() == 7 {
            for _ in 0..3 {
                buffer.read_byte();
            }
        }

        buffer.read_short();

        let id = buffer.read_short() as i32;
        let scale = buffer.read_float() as f64;
        let angle = buffer.read_float() as f64;
        let x = buffer.read_float() as f64;
        let y = buffer.read_float() as f64;
        let flip_x = buffer.read_boolean();
        let flip_y = buffer.read_boolean();
        let color = buffer.read_int();

        return Ok(Some(AvatarLayer {
            id,
            scale,
            angle,
            x,
            y,
            flip_x,
            flip_y,
            color,
        }));
    }

    Ok(None)
}

pub fn decode_avatar(encoded_avatar: &str) -> Result<Avatar, String> {
    if encoded_avatar.is_empty() {
        let mut default_avatar = Avatar::new();

        default_avatar.base_color = 4492031;

        return Ok(default_avatar);
    }

    let unquoted = percent_decode_str(encoded_avatar)
        .decode_utf8_lossy()
        .into_owned();
    let decoded_bytes = BASE64.decode(unquoted).expect("Invalid Base64");

    let mut byte_buffer = ByteBuffer::new(decoded_bytes);
    let mut avatar = Avatar::new();

    byte_buffer.read_padding();

    let shapes_count = (byte_buffer.read_byte() as i32 - 1) / 2;
    let mut wtf_chaz = byte_buffer.read_byte();

    while wtf_chaz != 1 {
        if wtf_chaz == 3 {
            byte_buffer.read_byte();
        } else if wtf_chaz == 5 {
            byte_buffer.read_byte();
            byte_buffer.read_byte();
        }

        wtf_chaz = byte_buffer.read_byte();
    }

    for _ in 0..shapes_count {
        let layer = decode_layer(&mut byte_buffer)?;

        if layer.is_some() {
            avatar.layers.push(layer.unwrap());
        }
    }

    avatar.base_color = byte_buffer.read_int();

    Ok(avatar)
}

#[pyfunction(name = "decode_avatar")]
pub fn py_decode_avatar(encoded_avatar: &str) -> PyResult<Avatar> {
    decode_avatar(encoded_avatar).map_err(PyValueError::new_err)
}
