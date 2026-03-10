use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use flate2::read::ZlibDecoder;
use serde_json::{Value, json};
use std::io::Read;

pub struct CoreByteBuffer {
    data: Vec<u8>,
    index: usize,
    implicit_class_alias_array: Vec<String>,
    implicit_string_array: Vec<String>,
    bodge_capture_zone_data_identifier_array: Vec<String>,
}

impl CoreByteBuffer {
    pub const T_UNDEFINED: u8 = 0x00;
    pub const T_NULL: u8 = 0x01;
    pub const T_FALSE: u8 = 0x02;
    pub const T_TRUE: u8 = 0x03;
    pub const T_INT: u8 = 0x04;
    pub const T_DOUBLE: u8 = 0x05;
    pub const T_STRING: u8 = 0x06;
    pub const T_ARRAY: u8 = 0x09;
    pub const T_OBJ: u8 = 0x0A;

    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            index: 0,
            implicit_class_alias_array: Vec::new(),
            implicit_string_array: Vec::new(),
            bodge_capture_zone_data_identifier_array: Vec::new(),
        }
    }

    pub fn from_base64(data: &str, encoded: bool) -> Result<Self, String> {
        let mut new_byte_buffer = Self::new();
        let mut decoded = BASE64
            .decode(data)
            .map_err(|e| format!("invalid base64: {e}"))?;

        if encoded {
            let mut z = ZlibDecoder::new(decoded.as_slice());
            let mut out = Vec::new();

            z.read_to_end(&mut out)
                .map_err(|e| format!("failed to inflate zlib payload: {e}"))?;

            decoded = out;
        }

        new_byte_buffer.data = decoded;
        new_byte_buffer.index = 0;
        new_byte_buffer.implicit_class_alias_array.clear();
        new_byte_buffer.implicit_string_array.clear();
        new_byte_buffer
            .bodge_capture_zone_data_identifier_array
            .clear();

        Ok(new_byte_buffer)
    }

    pub fn read_object(&mut self) -> Result<Value, String> {
        match self.read_byte()? {
            Self::T_NULL | Self::T_UNDEFINED => Ok(Value::Null),
            Self::T_OBJ => self.decode_object(),
            Self::T_ARRAY => self.decode_array(),
            marker => Err(format!(
                "trying to readObject on unsupported marker {marker:#x}"
            )),
        }
    }

    fn decode_object(&mut self) -> Result<Value, String> {
        let marker = self.read_byte()?;

        if marker == 0x07 {
            let alias_len_marker = self.read_byte()?;
            let alias_len = ((alias_len_marker as i32 - 1) / 2).max(0) as usize;
            let alias = self.read_string_bytes(alias_len)?;

            self.implicit_class_alias_array.push(alias.clone());
            self.decode_alias_external(&alias)
        } else {
            let alias_index = ((marker as i32 - 1) / 4).max(0) as usize;
            let alias = self
                .implicit_class_alias_array
                .get(alias_index)
                .cloned()
                .ok_or_else(|| format!("invalid implicit class alias index {alias_index}"))?;

            self.decode_alias_external(&alias)
        }
    }

    fn decode_array(&mut self) -> Result<Value, String> {
        let mut array_length = 0usize;

        loop {
            let chunk = ((self.read_byte()? as i32 - 1) / 2).max(0) as usize;

            array_length += chunk;

            if chunk != 64 {
                break;
            }
        }

        let _array_type_marker = self.read_byte()?;
        let mut out = Vec::with_capacity(array_length);

        for _ in 0..array_length {
            let element_type = self.read_byte()?;

            match element_type {
                Self::T_UNDEFINED | Self::T_NULL => out.push(Value::Null),
                Self::T_TRUE => out.push(Value::Bool(true)),
                Self::T_FALSE => out.push(Value::Bool(false)),
                Self::T_STRING => {
                    let string_marker = self.read_byte()?;
                    let s = if string_marker % 2 == 0 {
                        let idx = (string_marker / 2) as usize;

                        self.implicit_string_array
                            .get(idx)
                            .cloned()
                            .ok_or_else(|| format!("invalid implicit string index {idx}"))?
                    } else {
                        let mut len = ((string_marker as i32 - 1) / 2).max(0) as usize;

                        while ((len % 64) == 0) && len != 0 {
                            let next = ((self.read_byte()? as i32 - 1) / 2).max(0) as usize;

                            len += next;

                            if next != 64 {
                                break;
                            }
                        }

                        let s = self.read_string_bytes(len)?;

                        self.implicit_string_array.push(s.clone());

                        s
                    };

                    out.push(Value::String(s));
                }
                Self::T_OBJ => {
                    let object_marker = self.read_byte()?;

                    if object_marker == 7 {
                        let alias_len_marker = self.read_byte()?;
                        let alias_len = ((alias_len_marker as i32 - 1) / 2).max(0) as usize;
                        let alias = self.read_string_bytes(alias_len)?;

                        self.implicit_class_alias_array.push(alias.clone());

                        out.push(self.decode_alias_external(&alias)?);
                    } else if object_marker > 128 {
                        let _discard = self.read_byte()?;
                        let alias_len_marker = self.read_byte()?;
                        let alias_len = ((alias_len_marker as i32 - 1) / 2).max(0) as usize;
                        let alias = self.read_string_bytes(alias_len)?;

                        self.implicit_class_alias_array.push(alias.clone());

                        out.push(self.decode_alias_anonymous(&alias)?);
                    } else {
                        let alias_idx = ((object_marker as i32 - 1) / 4).max(0) as usize;
                        let alias = self
                            .implicit_class_alias_array
                            .get(alias_idx)
                            .cloned()
                            .ok_or_else(|| {
                                format!("invalid implicit class alias index {alias_idx}")
                            })?;

                        out.push(self.decode_alias_external(&alias)?);
                    }
                }
                _ => {}
            }
        }

        Ok(Value::Array(out))
    }

    fn decode_alias_external(&mut self, alias: &str) -> Result<Value, String> {
        match alias {
            "gmp" => self.read_gmp_external(),
            "ps" => self.read_ps_external(),
            "mspn" => Ok(json!(
                {
                    "x": self.read_double()?,
                    "y": self.read_double()?,
                    "xv": self.read_double()?,
                    "yv": self.read_double()?,
                    "ffa": self.read_boolean()?,
                    "blue": self.read_boolean()?,
                    "red": self.read_boolean()?,
                    "priority": self.read_short()? as i64,
                }
            )),
            "czs" => self.read_cap_zone_external(),
            _ => Err(format!(
                "trying to decode object with unknown alias '{alias}'"
            )),
        }
    }

    fn decode_alias_anonymous(&mut self, alias: &str) -> Result<Value, String> {
        match alias {
            "czs" => {
                self.bodge_capture_zone_data_identifier_array.clear();

                for _ in 0..8 {
                    let len_marker = self.read_byte()?;
                    let len = ((len_marker as i32 - 1) / 2).max(0) as usize;
                    let field_name = self.read_string_bytes(len)?;

                    self.bodge_capture_zone_data_identifier_array
                        .push(field_name);
                }

                self.read_cap_zone_external()
            }
            _ => Err(format!(
                "trying to decode anonymous object with alias '{alias}'"
            )),
        }
    }

    fn read_cap_zone_external(&mut self) -> Result<Value, String> {
        let mut obj = serde_json::Map::new();

        for i in 0..8 {
            let t = self.read_byte()?;
            let key = self
                .bodge_capture_zone_data_identifier_array
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("field{i}"));

            match t {
                Self::T_INT => {
                    obj.insert(key, json!(self.read_int29()?));
                }
                Self::T_DOUBLE => {
                    obj.insert(key, json!(self.read_double()?));
                }
                _ => {}
            }
        }

        Ok(Value::Object(obj))
    }

    fn read_gmp_external(&mut self) -> Result<Value, String> {
        let version = self.read_short()? as i64;
        let mut mapname = self.read_utf()?;

        if mapname.len() > 25 {
            mapname.truncate(25);
        }

        let mut author = self.read_utf()?;

        if author.len() > 35 {
            author.truncate(35);
        }

        let gravity = self.read_double()?;
        let mut ppm = self.read_double()?;

        ppm = ppm.clamp(5.0, 30.0);

        let disc_friction = self.read_double()?;
        let disc_restitution = self.read_double()?;
        let disc_density = self.read_double()?;
        let disc_linear_damping = self.read_double()?;
        let disc_radius = self.read_double()?;
        let disc_all_force = self.read_double()?;

        let platform_count = self.read_short()? as usize;
        let mut platform_array = Vec::with_capacity(platform_count);
        let mut platform_names = Vec::with_capacity(platform_count);

        for _ in 0..platform_count {
            platform_array.push(self.read_object()?);
            platform_names.push(Value::String(self.read_utf()?));
        }

        let spawn_count = self.read_short()? as usize;
        let mut spawn_array = Vec::with_capacity(spawn_count);
        let mut spawn_names = Vec::with_capacity(spawn_count);

        for _ in 0..spawn_count {
            spawn_array.push(self.read_object()?);
            spawn_names.push(Value::String(self.read_utf()?));
        }

        let cap_zone_array = if version >= 2 {
            self.read_object()?
        } else {
            Value::Array(vec![])
        };
        let cap_zone_names = if version >= 3 {
            self.read_object()?
        } else {
            Value::Array(vec![])
        };
        let (no_collide, respawn) = if version == 4 {
            let v = self.read_boolean()?;
            (v, v)
        } else if version > 4 {
            (self.read_boolean()?, self.read_boolean()?)
        } else {
            (false, false)
        };

        Ok(json!(
            {
                "version": version,
                "mapname": mapname,
                "author": author,
                "platformArray": platform_array,
                "spawnArray": spawn_array,
                "platformNames": platform_names,
                "spawnNames": spawn_names,
                "capZoneArray": cap_zone_array,
                "capZoneNames": cap_zone_names,
                "gravity": gravity,
                "ppm": ppm,
                "discFriction": disc_friction,
                "discRestitution": disc_restitution,
                "discDensity": disc_density,
                "discLinearDamping": disc_linear_damping,
                "discRadius": disc_radius,
                "discAllForce": disc_all_force,
                "respawn": respawn,
                "noCollide": no_collide
            }
        ))
    }

    fn read_ps_external(&mut self) -> Result<Value, String> {
        let version = self.read_short()? as i64;

        let mut x = 0.0;
        let mut y = 0.0;
        let mut shape_array: Vec<Value> = Vec::new();
        let mut restitution = 0.8;
        let mut friction = 0.0;
        let mut density = 1.0;
        let mut angle = 0.0;
        let mut angular_velocity = 0.0;
        let mut is_dynamic = false;
        let mut xv = 0.0;
        let mut yv = 0.0;
        let mut linear_damping = 0.0;
        let mut angular_damping = 0.0;
        let mut rotates = false;
        let mut rotate_pivot_x = 0.0;
        let mut rotate_pivot_y = 0.0;
        let mut stiffness = 0.0;
        let mut springy = false;
        let mut springy_upper = 100.0;
        let mut springy_anchor_x = 0.0;
        let mut springy_anchor_y = 0.0;
        let mut springy_force = 1_000_000.0;
        let mut path = false;
        let mut path_angle = std::f64::consts::PI / 2.0;
        let mut path_upper = 100.0;
        let mut path_anchor_x = 0.0;
        let mut path_anchor_y = 0.0;
        let mut path_max_speed = 50.0;
        let mut path_force = 1_000_000.0;

        if version <= 3 {
            x = self.read_double()?;
            y = self.read_double()?;

            let width = self.read_double()?;
            let height = self.read_double()?;

            restitution = self.read_double()?;
            density = self.read_double()?;
            friction = self.read_double()?;
            angle = self.read_double()?;
            angular_velocity = self.read_double()?;

            let shape_id = self.read_short()? as i64;

            is_dynamic = self.read_boolean()?;

            if is_dynamic {
                xv = self.read_double()?;
                yv = self.read_double()?;
                linear_damping = self.read_double()?;
                angular_damping = self.read_double()?;
                rotates = self.read_boolean()?;
                if rotates {
                    rotate_pivot_x = self.read_double()?;
                    rotate_pivot_y = self.read_double()?;
                    stiffness = self.read_double()?;
                }
            }

            let color = if version >= 2 {
                self.read_uint()? as i64
            } else if is_dynamic {
                0x8bc34a
            } else {
                0x58b173
            };
            let death = if version >= 3 {
                let _unknown = self.read_short()?;
                self.read_boolean()?
            } else {
                false
            };

            shape_array.push(json!(
                {
                    "localX": 0.0,
                    "localY": 0.0,
                    "localAngle": 0.0,
                    "width": width,
                    "height": height,
                    "shapeID": shape_id,
                    "color": color,
                    "death": death,
                    "noPhysics": false
                }
            ));
        }

        if version >= 4 {
            x = self.read_double()?;
            y = self.read_double()?;

            let shape_count = self.read_short()? as usize;

            shape_array.clear();

            for _ in 0..shape_count {
                let local_x = self.read_double()?;
                let local_y = self.read_double()?;
                let width = self.read_double()?;
                let mut height = self.read_double()?;
                let local_angle = self.read_double()?;
                let shape_id = self.read_short()? as i64;
                let color = self.read_uint()? as i64;
                let death = self.read_boolean()?;
                let no_physics = if version >= 7 {
                    self.read_boolean()?
                } else {
                    false
                };

                if version == 4 && shape_id == 5 {
                    height = (0.866 * width).round();
                }

                shape_array.push(json!(
                    {
                        "localX": local_x,
                        "localY": local_y,
                        "width": width,
                        "height": height,
                        "localAngle": local_angle,
                        "shapeID": shape_id,
                        "color": color,
                        "death": death,
                        "noPhysics": no_physics
                    }
                ));
            }

            restitution = self.read_double()?;
            density = self.read_double()?;
            friction = self.read_double()?;
            angle = self.read_double()?;
            angular_velocity = self.read_double()?;
            is_dynamic = self.read_boolean()?;

            if is_dynamic {
                xv = self.read_double()?;
                yv = self.read_double()?;
                linear_damping = self.read_double()?;
                angular_damping = self.read_double()?;
                rotates = self.read_boolean()?;

                if rotates {
                    rotate_pivot_x = self.read_double()?;
                    rotate_pivot_y = self.read_double()?;
                    stiffness = self.read_double()?;
                }

                if version >= 6 {
                    springy = self.read_boolean()?;

                    if springy {
                        springy_upper = self.read_double()?;
                        springy_force = self.read_double()?;
                        springy_anchor_x = self.read_double()?;
                        springy_anchor_y = self.read_double()?;
                    }

                    path = self.read_boolean()?;

                    if path {
                        path_angle = self.read_double()?;
                        path_upper = self.read_double()?;
                        path_max_speed = self.read_double()?;
                        path_force = self.read_double()?;
                        path_anchor_x = self.read_double()?;
                        path_anchor_y = self.read_double()?;
                    }
                }
            }
        }

        Ok(json!(
            {
                "version": version,
                "x": x,
                "y": y,
                "shapeArray": shape_array,
                "restitution": restitution,
                "friction": friction,
                "density": density,
                "angle": angle,
                "angularVelocity": angular_velocity,
                "isDynamic": is_dynamic,
                "xv": xv,
                "yv": yv,
                "linearDamping": linear_damping,
                "angularDamping": angular_damping,
                "rotates": rotates,
                "rotatePivotX": rotate_pivot_x,
                "rotatePivotY": rotate_pivot_y,
                "stiffness": stiffness,
                "springy": springy,
                "springyUpper": springy_upper,
                "springyAnchorX": springy_anchor_x,
                "springyAnchorY": springy_anchor_y,
                "springyForce": springy_force,
                "path": path,
                "pathAngle": path_angle,
                "pathUpper": path_upper,
                "pathAnchorX": path_anchor_x,
                "pathAnchorY": path_anchor_y,
                "pathMaxSpeed": path_max_speed,
                "pathForce": path_force
            }
        ))
    }

    fn read_string_bytes(&mut self, len: usize) -> Result<String, String> {
        let bytes = self.read_slice(len)?;

        std::str::from_utf8(bytes)
            .map(|s| s.to_owned())
            .map_err(|e| format!("invalid utf8 string: {e}"))
    }

    fn read_slice(&mut self, len: usize) -> Result<&[u8], String> {
        if self.index + len > self.data.len() {
            return Err("buffer underflow".to_string());
        }

        let s = &self.data[self.index..self.index + len];

        self.index += len;

        Ok(s)
    }

    fn read_byte(&mut self) -> Result<u8, String> {
        if self.index >= self.data.len() {
            return Err("buffer underflow".to_string());
        }

        let value = self.data[self.index];
        self.index += 1;

        Ok(value)
    }

    fn read_boolean(&mut self) -> Result<bool, String> {
        Ok(self.read_byte()? == 1)
    }

    fn read_short(&mut self) -> Result<i16, String> {
        let bytes = self.read_slice(2)?;

        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_uint(&mut self) -> Result<u32, String> {
        let bytes = self.read_slice(4)?;

        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_utf(&mut self) -> Result<String, String> {
        let len_bytes = self.read_slice(2)?;
        let b1 = len_bytes[0] as usize;
        let b2 = len_bytes[1] as usize;
        let len = (b1 << 8) | b2;

        self.read_string_bytes(len)
    }

    fn read_double(&mut self) -> Result<f64, String> {
        let bytes = self.read_slice(8)?;

        Ok(f64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_int29(&mut self) -> Result<i32, String> {
        let byte1 = self.read_byte()?;
        let mut result: u32;

        if (byte1 & 0x80) == 0 {
            result = (byte1 & 0x7F) as u32;
        } else {
            let byte2 = self.read_byte()?;

            if (byte2 & 0x80) == 0 {
                result = (((byte1 & 0x7F) as u32) << 7) | ((byte2 & 0x7F) as u32);
            } else {
                let byte3 = self.read_byte()?;

                if (byte3 & 0x80) == 0 {
                    result = (((byte1 & 0x7F) as u32) << 14)
                        | (((byte2 & 0x7F) as u32) << 7)
                        | ((byte3 & 0x7F) as u32);
                } else {
                    let byte4 = self.read_byte()?;

                    result = (((byte1 & 0x7F) as u32) << 22)
                        | (((byte2 & 0x7F) as u32) << 15)
                        | (((byte3 & 0x7F) as u32) << 8)
                        | (byte4 as u32);
                }
            }
        }

        if (result & 0x1000_0000) != 0 {
            result |= 0xE000_0000;
        }

        Ok(result as i32)
    }
}
