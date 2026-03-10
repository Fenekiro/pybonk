use serde_json::{Map, Number, Value};
use std::collections::HashMap;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PsonToken {
    Zero = 0,
    Max = 239,
    Null = 240,
    True = 241,
    False = 242,
    EObject = 243,
    EArray = 244,
    EString = 245,
    Object = 246,
    Array = 247,
    Integer = 248,
    Long = 249,
    Float = 250,
    Double = 251,
    String = 252,
    StringAdd = 253,
    StringGet = 254,
    Binary = 255,
}

pub struct Pson {
    dict_enc: HashMap<String, u32>,
    dict_dec: Vec<String>,
    progressive: bool,
    next_index: u32,
}

impl Pson {
    pub fn new(initial_dict: Vec<String>, progressive: bool) -> Self {
        let mut dict_enc = HashMap::with_capacity(initial_dict.len());

        for (i, s) in initial_dict.iter().enumerate() {
            dict_enc.insert(s.clone(), i as u32);
        }

        Self {
            next_index: initial_dict.len() as u32,
            dict_enc,
            dict_dec: initial_dict,
            progressive,
        }
    }

    // --- ENCODING ---

    pub fn encode(&mut self, value: &Value) -> Vec<u8> {
        let mut buf = Vec::new();

        self.encode_value(value, &mut buf);

        buf
    }

    fn encode_value(&mut self, value: &Value, buf: &mut Vec<u8>) {
        match value {
            Value::Null => buf.push(PsonToken::Null as u8),
            Value::Bool(b) => buf.push(if *b {
                PsonToken::True as u8
            } else {
                PsonToken::False as u8
            }),
            Value::Number(n) => self.encode_number(n, buf),
            Value::String(s) => self.encode_string(s, buf),
            Value::Array(arr) => self.encode_array(arr, buf),
            Value::Object(obj) => self.encode_object(obj, buf),
        }
    }

    fn encode_number(&self, n: &Number, buf: &mut Vec<u8>) {
        if let Some(i) = n.as_i64() {
            let zz = self.zigzag_encode(i);

            if zz <= PsonToken::Max as u64 {
                buf.push(zz as u8);
            } else {
                buf.push(PsonToken::Integer as u8);

                self.write_varint(zz, buf);
            }
        } else if let Some(f) = n.as_f64() {
            let f32_val = f as f32;

            if f32_val as f64 == f {
                buf.push(PsonToken::Float as u8);
                buf.extend_from_slice(&f32_val.to_le_bytes());
            } else {
                buf.push(PsonToken::Double as u8);
                buf.extend_from_slice(&f.to_le_bytes());
            }
        }
    }

    fn encode_string(&mut self, s: &str, buf: &mut Vec<u8>) {
        if s.is_empty() {
            buf.push(PsonToken::EString as u8);
            return;
        }

        if let Some(&index) = self.dict_enc.get(s) {
            buf.push(PsonToken::StringGet as u8);
            self.write_varint(index as u64, buf);
        } else {
            if self.progressive {
                buf.push(PsonToken::StringAdd as u8);

                self.dict_enc.insert(s.to_owned(), self.next_index);

                self.next_index += 1;
            } else {
                buf.push(PsonToken::String as u8);
            }

            self.write_vstring(s, buf);
        }
    }

    fn encode_array(&mut self, arr: &[Value], buf: &mut Vec<u8>) {
        if arr.is_empty() {
            buf.push(PsonToken::EArray as u8);
        } else {
            buf.push(PsonToken::Array as u8);

            self.write_varint(arr.len() as u64, buf);

            for v in arr {
                self.encode_value(v, buf);
            }
        }
    }

    fn encode_object(&mut self, obj: &Map<String, Value>, buf: &mut Vec<u8>) {
        if obj.is_empty() {
            buf.push(PsonToken::EObject as u8);
        } else {
            buf.push(PsonToken::Object as u8);

            self.write_varint(obj.len() as u64, buf);

            for (k, v) in obj {
                self.encode_string(k, buf);
                self.encode_value(v, buf);
            }
        }
    }

    // --- DECODING ---

    pub fn decode(&mut self, data: &[u8]) -> Result<Value, String> {
        let mut pos = 0;
        self.decode_value(data, &mut pos)
    }

    fn decode_value(&mut self, data: &[u8], pos: &mut usize) -> Result<Value, String> {
        let token = *data
            .get(*pos)
            .ok_or_else(|| format!("Unexpected end of data at pos {}", *pos))?;
        *pos += 1;

        if token <= PsonToken::Max as u8 {
            return Ok(Value::Number(Number::from(
                self.zigzag_decode(token as u64),
            )));
        }

        match token {
            t if t == PsonToken::Null as u8 => Ok(Value::Null),
            t if t == PsonToken::True as u8 => Ok(Value::Bool(true)),
            t if t == PsonToken::False as u8 => Ok(Value::Bool(false)),
            t if t == PsonToken::EString as u8 => Ok(Value::String(String::new())),
            t if t == PsonToken::EArray as u8 => Ok(Value::Array(vec![])),
            t if t == PsonToken::EObject as u8 => Ok(Value::Object(Map::new())),

            t if t == PsonToken::Integer as u8 => {
                let val = self.read_varint(data, pos);

                Ok(Value::Number(Number::from(self.zigzag_decode(val))))
            }
            t if t == PsonToken::Float as u8 => {
                let mut b = [0u8; 4];
                let end = *pos + 4;
                b.copy_from_slice(
                    data.get(*pos..end)
                        .ok_or_else(|| format!("Unexpected end of data at pos {}", *pos))?,
                );

                *pos += 4;

                Ok(Value::from(f32::from_le_bytes(b)))
            }
            t if t == PsonToken::Double as u8 => {
                let mut b = [0u8; 8];
                let end = *pos + 8;
                b.copy_from_slice(
                    data.get(*pos..end)
                        .ok_or_else(|| format!("Unexpected end of data at pos {}", *pos))?,
                );

                *pos += 8;

                Ok(Value::from(f64::from_le_bytes(b)))
            }
            t if t == PsonToken::String as u8 => Ok(Value::String(self.read_vstring(data, pos)?)),
            t if t == PsonToken::StringAdd as u8 => {
                let s = self.read_vstring(data, pos)?;

                self.dict_dec.push(s.clone());

                Ok(Value::String(s))
            }
            t if t == PsonToken::StringGet as u8 => {
                let idx = self.read_varint(data, pos) as usize;

                self.dict_dec
                    .get(idx)
                    .cloned()
                    .map(Value::String)
                    .ok_or_else(|| format!("String dictionary index out of range: {}", idx))
            }
            t if t == PsonToken::Array as u8 => {
                let len = self.read_varint(data, pos) as usize;
                let mut arr = Vec::with_capacity(len);

                for _ in 0..len {
                    arr.push(self.decode_value(data, pos)?);
                }

                Ok(Value::Array(arr))
            }
            t if t == PsonToken::Object as u8 => {
                let len = self.read_varint(data, pos) as usize;
                let mut obj = Map::new();

                for _ in 0..len {
                    let key = self.decode_string_value(data, pos)?;
                    let val = self.decode_value(data, pos)?;
                    obj.insert(key, val);
                }

                Ok(Value::Object(obj))
            }
            _ => Err(format!("Unknown token {} at pos {}", token, pos)),
        }
    }

    fn decode_string_value(&mut self, data: &[u8], pos: &mut usize) -> Result<String, String> {
        let token = *data
            .get(*pos)
            .ok_or_else(|| format!("Unexpected end of data at pos {}", *pos))?;
        *pos += 1;

        match token {
            t if t == PsonToken::EString as u8 => Ok(String::new()),
            t if t == PsonToken::String as u8 => self.read_vstring(data, pos),
            t if t == PsonToken::StringAdd as u8 => {
                let s = self.read_vstring(data, pos)?;
                self.dict_dec.push(s.clone());
                Ok(s)
            }
            t if t == PsonToken::StringGet as u8 => {
                let idx = self.read_varint(data, pos) as usize;
                self.dict_dec
                    .get(idx)
                    .cloned()
                    .ok_or_else(|| format!("String dictionary index out of range: {}", idx))
            }
            _ => Err(format!(
                "Expected string token at pos {}, found {}",
                *pos - 1,
                token
            )),
        }
    }

    // --- HELPERS (Varint, ZigZag) ---

    fn write_varint(&self, mut val: u64, buf: &mut Vec<u8>) {
        while val >= 0x80 {
            buf.push((val as u8 & 0x7f) | 0x80);
            val >>= 7;
        }

        buf.push(val as u8);
    }

    fn read_varint(&self, data: &[u8], pos: &mut usize) -> u64 {
        let mut val = 0u64;
        let mut shift = 0;

        loop {
            let b = data[*pos];

            *pos += 1;

            val |= ((b & 0x7f) as u64) << shift;

            if b < 0x80 {
                break;
            }

            shift += 7;
        }

        val
    }

    fn write_vstring(&self, s: &str, buf: &mut Vec<u8>) {
        let bytes = s.as_bytes();

        self.write_varint(bytes.len() as u64, buf);

        buf.extend_from_slice(bytes);
    }

    fn read_vstring(&self, data: &[u8], pos: &mut usize) -> Result<String, String> {
        let len = self.read_varint(data, pos) as usize;
        let end = pos
            .checked_add(len)
            .ok_or_else(|| "String length overflow".to_string())?;
        let bytes = data
            .get(*pos..end)
            .ok_or_else(|| format!("Unexpected end of data at pos {}", *pos))?;
        *pos = end;

        std::str::from_utf8(bytes)
            .map(|s| s.to_owned())
            .map_err(|e| format!("Invalid UTF-8 string: {e}"))
    }

    fn zigzag_encode(&self, n: i64) -> u64 {
        ((n << 1) ^ (n >> 63)) as u64
    }

    fn zigzag_decode(&self, n: u64) -> i64 {
        ((n >> 1) as i64) ^ (-((n & 1) as i64))
    }
}
