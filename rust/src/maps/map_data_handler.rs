use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use lz_str::{compress_to_encoded_uri_component, decompress_from_encoded_uri_component};
use percent_encoding::percent_decode_str;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde_json::Value;

use crate::byte_parsers::byte_buffer::ByteBuffer;
use crate::byte_parsers::core_byte_buffer::CoreByteBuffer;
use crate::byte_parsers::pson::Pson;
use crate::maps::bonk2map::{
    Body, Bonk2Map, CapZone, Fixture, Joint, JointProperties, Shape, Spawn,
};

pub const MAP_VERSION: i16 = 15;
pub const PSON_KEYS: &[&str] = &[
    "physics",
    "shapes",
    "fixtures",
    "bodies",
    "bro",
    "joints",
    "ppm",
    "lights",
    "spawns",
    "lasers",
    "capZones",
    "type",
    "w",
    "h",
    "c",
    "a",
    "v",
    "l",
    "s",
    "sh",
    "fr",
    "re",
    "de",
    "sn",
    "fc",
    "fm",
    "f",
    "d",
    "n",
    "bg",
    "lv",
    "av",
    "ld",
    "ad",
    "fr",
    "bu",
    "cf",
    "rv",
    "p",
    "d",
    "bf",
    "ba",
    "bb",
    "aa",
    "ab",
    "axa",
    "dr",
    "em",
    "mmt",
    "mms",
    "ms",
    "ut",
    "lt",
    "New body",
    "Box Shape",
    "Circle Shape",
    "Polygon Shape",
    "EdgeChain Shape",
    "priority",
    "Light",
    "Laser",
    "Cap Zone",
    "BG Shape",
    "Background Layer",
    "Rotate Joint",
    "Slider Joint",
    "Rod Joint",
    "Gear Joint",
    "65535",
    "16777215",
];

pub struct MapDataHandler;

impl MapDataHandler {
    fn estimate_buffer_capacity(map: &Bonk2Map) -> usize {
        let metadata_string_bytes = map.m.rxn.len()
            + map.m.rxa.len()
            + map.m.n.len()
            + map.m.a.len()
            + map.m.mo.len()
            + map.m.cr.iter().map(String::len).sum::<usize>();
        let fixture_string_bytes = map
            .physics
            .fixtures
            .iter()
            .map(|f| f.n.len())
            .sum::<usize>();
        let body_string_bytes = map
            .physics
            .bodies
            .iter()
            .map(|b| b.s.kind.len() + b.s.n.len())
            .sum::<usize>();
        let spawn_string_bytes = map.spawns.iter().map(|s| s.n.len()).sum::<usize>();
        let cap_zone_string_bytes = map.cap_zones.iter().map(|cz| cz.n.len()).sum::<usize>();
        let joint_string_bytes = map
            .physics
            .joints
            .iter()
            .map(|j| j.n.as_deref().map_or(0, str::len))
            .sum::<usize>();

        let string_bytes = metadata_string_bytes
            + fixture_string_bytes
            + body_string_bytes
            + spawn_string_bytes
            + cap_zone_string_bytes
            + joint_string_bytes;

        let estimated_payload = map.physics.shapes.len() * 64
            + map.physics.fixtures.len() * 64
            + map.physics.bodies.len() * 192
            + map.physics.joints.len() * 96
            + map.physics.bro.len() * 2
            + map.spawns.len() * 96
            + map.cap_zones.len() * 40
            + string_bytes;

        estimated_payload.max(512)
    }

    pub fn get_blank_map() -> Bonk2Map {
        Bonk2Map::default()
    }

    pub fn get_new_body() -> Body {
        Body::default()
    }

    pub fn get_new_fixture(shape_id: Option<i16>) -> Fixture {
        Fixture::new(shape_id.unwrap_or_default())
    }

    pub fn get_new_box_shape() -> Shape {
        Shape::new_box()
    }

    pub fn get_new_circle_shape() -> Shape {
        Shape::new_circle()
    }

    pub fn get_new_poly_shape() -> Shape {
        Shape::new_poly()
    }

    pub fn get_new_chain_shape() -> Shape {
        Shape::new_chain()
    }

    pub fn get_new_revolute_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
        Joint::new_revolute(a_id, b_id)
    }

    pub fn get_new_distance_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
        Joint::new_distance(a_id, b_id)
    }

    pub fn get_new_lpj_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
        Joint::new_lpj(a_id, b_id)
    }

    pub fn get_new_lsj_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
        Joint::new_lsj(a_id, b_id)
    }

    pub fn get_new_gear_joint() -> Joint {
        Joint::new_gear()
    }

    pub fn get_new_cap_zone() -> CapZone {
        CapZone::default()
    }

    pub fn get_new_spawn() -> Spawn {
        Spawn::default()
    }

    pub fn pson_encode(map: &Bonk2Map) -> Result<Vec<u8>, String> {
        let mut pson = Pson::new(PSON_KEYS.iter().map(|s| s.to_string()).collect(), false);
        let value =
            serde_json::to_value(map).map_err(|e| format!("failed to serialize map: {e}"))?;

        Ok(pson.encode(&value))
    }

    pub fn pson_decode(map_data: &[u8]) -> Result<Bonk2Map, String> {
        let mut pson = Pson::new(PSON_KEYS.iter().map(|s| s.to_string()).collect(), false);
        let decoded = pson.decode(map_data)?;
        let map: Bonk2Map = serde_json::from_value(decoded)
            .map_err(|e| format!("failed to parse map json: {e}"))?;

        Ok(Self::merge_into_new_map(map))
    }

    pub fn decode_from_old_b64_string(old_string: &str) -> Result<Bonk2Map, String> {
        let decoded = percent_decode_str(old_string)
            .decode_utf8()
            .map_err(|e| format!("invalid percent-encoded old map string: {e}"))?;
        let mut buffer = CoreByteBuffer::from_base64(&decoded, true)?;

        let map_data = buffer.read_object()?;
        let map = Self::from_old_format(&map_data)?;

        Ok(Self::merge_into_new_map(map))
    }

    pub fn encode_to_b64_string(map: &mut Bonk2Map) -> String {
        let mut buf = ByteBuffer::with_capacity(Self::estimate_buffer_capacity(map));

        map.v = MAP_VERSION;

        buf.write_short(map.v);
        buf.write_boolean(map.s.re);
        buf.write_boolean(map.s.nc);
        buf.write_short(map.s.pq);
        buf.write_float(map.s.gd as f32);
        buf.write_boolean(map.s.fl);

        buf.write_utf(&map.m.rxn);
        buf.write_utf(&map.m.rxa);
        buf.write_uint(map.m.rxid);
        buf.write_short(map.m.rxdb as i16);
        buf.write_utf(&map.m.n);
        buf.write_utf(&map.m.a);
        buf.write_uint(map.m.vu.unwrap_or(0));
        buf.write_uint(map.m.vd.unwrap_or(0));

        buf.write_short(map.m.cr.len() as i16);

        for cr in &map.m.cr {
            buf.write_utf(cr);
        }

        buf.write_utf(&map.m.mo);
        buf.write_int(map.m.dbid);
        buf.write_boolean(map.m.is_pub);
        buf.write_int(map.m.dbv);

        buf.write_short(map.physics.ppm);

        buf.write_short(map.physics.bro.len() as i16);

        for bro in &map.physics.bro {
            buf.write_short(*bro);
        }

        buf.write_short(map.physics.shapes.len() as i16);

        for shape in &map.physics.shapes {
            match shape.kind.as_str() {
                "bx" => {
                    buf.write_short(1);
                    buf.write_double(shape.w.unwrap_or(10.0));
                    buf.write_double(shape.h.unwrap_or(40.0));
                    buf.write_double(shape.c[0]);
                    buf.write_double(shape.c[1]);
                    buf.write_double(shape.a.unwrap_or(0.0));
                    buf.write_boolean(shape.sk.unwrap_or(false));
                }
                "ci" => {
                    buf.write_short(2);
                    buf.write_double(shape.r.unwrap_or(25.0));
                    buf.write_double(shape.c[0]);
                    buf.write_double(shape.c[1]);
                    buf.write_boolean(shape.sk.unwrap_or(false));
                }
                "po" => {
                    buf.write_short(3);
                    buf.write_double(shape.s.unwrap_or(1.0));
                    buf.write_double(shape.a.unwrap_or(0.0));
                    buf.write_double(shape.c[0]);
                    buf.write_double(shape.c[1]);

                    let v = shape.v.as_deref().unwrap_or(&[]);

                    buf.write_short(v.len() as i16);

                    for vertex in v {
                        buf.write_double(vertex[0]);
                        buf.write_double(vertex[1]);
                    }
                }
                _ => {}
            }
        }

        buf.write_short(map.physics.fixtures.len() as i16);

        for fixture in &map.physics.fixtures {
            buf.write_short(fixture.sh);
            buf.write_utf(&fixture.n);
            buf.write_double(fixture.fr.unwrap_or(f64::MAX));

            match fixture.fp {
                None => buf.write_short(0),
                Some(false) => buf.write_short(1),
                Some(true) => buf.write_short(2),
            }

            buf.write_double(fixture.re.unwrap_or(f64::MAX));
            buf.write_double(fixture.de.unwrap_or(f64::MAX));
            buf.write_uint(fixture.f);
            buf.write_boolean(fixture.d);
            buf.write_boolean(fixture.np);
            buf.write_boolean(fixture.ng);
            buf.write_boolean(fixture.ig.unwrap_or(false));
        }

        buf.write_short(map.physics.bodies.len() as i16);

        for body in &map.physics.bodies {
            buf.write_utf(&body.s.kind);
            buf.write_utf(&body.s.n);
            buf.write_double(body.p[0]);
            buf.write_double(body.p[1]);
            buf.write_double(body.a);
            buf.write_double(body.s.fric);
            buf.write_boolean(body.s.fricp);
            buf.write_double(body.s.re);
            buf.write_double(body.s.de);
            buf.write_double(body.lv[0]);
            buf.write_double(body.lv[1]);
            buf.write_double(body.av);
            buf.write_double(body.s.ld);
            buf.write_double(body.s.ad);
            buf.write_boolean(body.s.fr);
            buf.write_boolean(body.s.bu);

            buf.write_double(body.cf.x);
            buf.write_double(body.cf.y);
            buf.write_double(body.cf.ct);
            buf.write_boolean(body.cf.w);

            buf.write_short(body.s.f_c);
            buf.write_boolean(body.s.f_1);
            buf.write_boolean(body.s.f_2);
            buf.write_boolean(body.s.f_3);
            buf.write_boolean(body.s.f_4);
            buf.write_boolean(body.s.f_p);

            buf.write_boolean(body.fz.on);

            if body.fz.on {
                buf.write_double(body.fz.x);
                buf.write_double(body.fz.y);
                buf.write_boolean(body.fz.d);
                buf.write_boolean(body.fz.p);
                buf.write_boolean(body.fz.a);
                buf.write_short(body.fz.t);
                buf.write_double(body.fz.cf);
            }

            buf.write_short(body.fx.len() as i16);

            for fx in &body.fx {
                buf.write_short(*fx);
            }
        }

        buf.write_short(map.spawns.len() as i16);

        for spawn in &map.spawns {
            buf.write_double(spawn.x);
            buf.write_double(spawn.y);
            buf.write_double(spawn.xv);
            buf.write_double(spawn.yv);
            buf.write_short(spawn.priority);
            buf.write_boolean(spawn.r);
            buf.write_boolean(spawn.f);
            buf.write_boolean(spawn.b);
            buf.write_boolean(spawn.gr.unwrap_or(false));
            buf.write_boolean(spawn.ye.unwrap_or(false));
            buf.write_utf(&spawn.n);
        }

        buf.write_short(map.cap_zones.len() as i16);

        for cap_zone in &map.cap_zones {
            buf.write_utf(&cap_zone.n);
            buf.write_double(cap_zone.l);
            buf.write_short(cap_zone.i);
            buf.write_short(cap_zone.ty);
        }

        buf.write_short(map.physics.joints.len() as i16);

        for joint in &map.physics.joints {
            let wrote_type = match joint.kind.as_str() {
                "rv" => {
                    buf.write_short(1);

                    buf.write_double(joint.d.as_ref().and_then(|d| d.la).unwrap_or(0.0));
                    buf.write_double(joint.d.as_ref().and_then(|d| d.ua).unwrap_or(0.0));
                    buf.write_double(joint.d.as_ref().and_then(|d| d.mmt).unwrap_or(0.0));
                    buf.write_double(joint.d.as_ref().and_then(|d| d.ms).unwrap_or(0.0));
                    buf.write_boolean(joint.d.as_ref().and_then(|d| d.el).unwrap_or(false));
                    buf.write_boolean(joint.d.as_ref().and_then(|d| d.em).unwrap_or(false));

                    let aa = joint.aa.unwrap_or([0.0, 0.0]);

                    buf.write_double(aa[0]);
                    buf.write_double(aa[1]);

                    true
                }
                "d" => {
                    buf.write_short(2);

                    buf.write_double(joint.d.as_ref().and_then(|d| d.fh).unwrap_or(0.0));
                    buf.write_double(joint.d.as_ref().and_then(|d| d.dr).unwrap_or(0.0));

                    let aa = joint.aa.unwrap_or([0.0, 0.0]);
                    let ab = joint.ab.unwrap_or([0.0, 0.0]);

                    buf.write_double(aa[0]);
                    buf.write_double(aa[1]);
                    buf.write_double(ab[0]);
                    buf.write_double(ab[1]);

                    true
                }
                "lpj" => {
                    buf.write_short(3);
                    buf.write_double(joint.pax.unwrap_or(0.0));
                    buf.write_double(joint.pay.unwrap_or(0.0));
                    buf.write_double(joint.pa.unwrap_or(0.0));
                    buf.write_double(joint.pf.unwrap_or(0.0));
                    buf.write_double(joint.pl.unwrap_or(0.0));
                    buf.write_double(joint.pu.unwrap_or(0.0));
                    buf.write_double(joint.plen.unwrap_or(0.0));
                    buf.write_double(joint.pms.unwrap_or(0.0));

                    true
                }
                "lsj" => {
                    buf.write_short(4);
                    buf.write_double(joint.sax.unwrap_or(0.0));
                    buf.write_double(joint.say.unwrap_or(0.0));
                    buf.write_double(joint.sf.unwrap_or(0.0));
                    buf.write_double(joint.slen.unwrap_or(0.0));

                    true
                }
                "g" => {
                    buf.write_short(5);
                    buf.write_utf(joint.n.as_deref().unwrap_or(""));
                    buf.write_short(joint.ja.unwrap_or(-1));
                    buf.write_short(joint.jb.unwrap_or(-1));
                    buf.write_double(joint.r.unwrap_or(1.0));

                    true
                }
                _ => {
                    buf.write_short(2);

                    buf.write_double(joint.d.as_ref().and_then(|d| d.fh).unwrap_or(0.0));
                    buf.write_double(joint.d.as_ref().and_then(|d| d.dr).unwrap_or(0.0));

                    let aa = joint.aa.unwrap_or([0.0, 0.0]);
                    let ab = joint.ab.unwrap_or([0.0, 0.0]);

                    buf.write_double(aa[0]);
                    buf.write_double(aa[1]);
                    buf.write_double(ab[0]);
                    buf.write_double(ab[1]);

                    true
                }
            };

            if wrote_type && joint.kind != "g" {
                buf.write_short(joint.ba.unwrap_or(-1));
                buf.write_short(joint.bb.unwrap_or(-1));

                buf.write_boolean(joint.d.as_ref().is_some_and(|d| d.cc));
                buf.write_double(joint.d.as_ref().and_then(|d| d.bf).unwrap_or(0.0));
                buf.write_boolean(joint.d.as_ref().and_then(|d| d.dl).unwrap_or(true));
            }
        }

        let base64_str = BASE64.encode(&buf.data);
        let compressed = compress_to_encoded_uri_component(&base64_str);

        compressed
    }

    pub fn decode_from_b64_string(encoded_string: &str) -> Result<Bonk2Map, String> {
        let decompressed_u16 = decompress_from_encoded_uri_component(encoded_string)
            .ok_or("Failed to decompress map data")?;
        let base64_str = String::from_utf16(&decompressed_u16)
            .map_err(|_| "Invalid UTF-16 in decompressed data")?;
        let bytes = BASE64.decode(base64_str).map_err(|_| "Invalid Base64")?;
        let mut buf = ByteBuffer::new(bytes);
        let mut map = Bonk2Map::default();

        map.v = buf.read_short();

        if map.v > MAP_VERSION {
            return Err("Future map version, please refresh page".to_string());
        }

        map.s.re = buf.read_boolean();
        map.s.nc = buf.read_boolean();

        if map.v >= 3 {
            map.s.pq = buf.read_short();
        }
        if map.v >= 4 && map.v <= 12 {
            map.s.gd = buf.read_short() as f64;
        } else if map.v >= 13 {
            map.s.gd = buf.read_float() as f64;
        }
        if map.v >= 9 {
            map.s.fl = buf.read_boolean();
        }

        map.m.rxn = buf.read_utf();
        map.m.rxa = buf.read_utf();
        map.m.rxid = buf.read_uint();
        map.m.rxdb = buf.read_short() as i32;
        map.m.n = buf.read_utf();
        map.m.a = buf.read_utf();

        if map.v >= 10 {
            map.m.vu = Some(buf.read_uint());
            map.m.vd = Some(buf.read_uint());
        }

        if map.v >= 4 {
            let cr_count = buf.read_short().max(0) as usize;
            map.m.cr.reserve(cr_count);

            for _ in 0..cr_count {
                map.m.cr.push(buf.read_utf());
            }
        }

        if map.v >= 5 {
            map.m.mo = buf.read_utf();
            map.m.dbid = buf.read_int();
        }
        if map.v >= 7 {
            map.m.is_pub = buf.read_boolean();
        }
        if map.v >= 8 {
            map.m.dbv = buf.read_int();
        }

        map.physics.ppm = buf.read_short();

        let bro_count = buf.read_short().max(0) as usize;
        map.physics.bro.reserve(bro_count);

        for _ in 0..bro_count {
            map.physics.bro.push(buf.read_short());
        }

        let shapes_count = buf.read_short().max(0) as usize;
        map.physics.shapes.reserve(shapes_count);

        for _ in 0..shapes_count {
            let shape_type = buf.read_short();

            if shape_type == 1 {
                let mut s = Shape::new_box();

                s.w = Some(buf.read_double());
                s.h = Some(buf.read_double());
                s.c = [buf.read_double(), buf.read_double()];
                s.a = Some(buf.read_double());
                s.sk = Some(buf.read_boolean());

                map.physics.shapes.push(s);
            } else if shape_type == 2 {
                let mut s = Shape::new_circle();

                s.r = Some(buf.read_double());
                s.c = [buf.read_double(), buf.read_double()];
                s.sk = Some(buf.read_boolean());

                map.physics.shapes.push(s);
            } else if shape_type == 3 {
                let mut s = Shape::new_poly();

                s.s = Some(buf.read_double());
                s.a = Some(buf.read_double());
                s.c = [buf.read_double(), buf.read_double()];

                let vertices_count = buf.read_short().max(0) as usize;
                let mut v_arr = Vec::with_capacity(vertices_count);

                for _ in 0..vertices_count {
                    v_arr.push([buf.read_double(), buf.read_double()]);
                }

                s.v = Some(v_arr);

                map.physics.shapes.push(s);
            } else {
                map.physics.shapes.push(Shape::new_box());
            }
        }

        let fixtures_count = buf.read_short().max(0) as usize;
        map.physics.fixtures.reserve(fixtures_count);
        for _ in 0..fixtures_count {
            let sh = buf.read_short();
            let mut f = Fixture::new(sh);
            f.n = buf.read_utf();

            let fr = buf.read_double();

            f.fr = if fr == f64::MAX { None } else { Some(fr) };

            let fp_val = buf.read_short();

            f.fp = match fp_val {
                0 => None,
                1 => Some(false),
                2 => Some(true),
                _ => None,
            };

            let re = buf.read_double();

            f.re = if re == f64::MAX { None } else { Some(re) };

            let de = buf.read_double();

            f.de = if de == f64::MAX { None } else { Some(de) };

            f.f = buf.read_uint();
            f.d = buf.read_boolean();
            f.np = buf.read_boolean();

            if map.v >= 11 {
                f.ng = buf.read_boolean();
            }
            if map.v >= 12 {
                f.ig = Some(buf.read_boolean());
            }

            map.physics.fixtures.push(f);
        }

        let bodies_count = buf.read_short().max(0) as usize;
        map.physics.bodies.reserve(bodies_count);

        for _ in 0..bodies_count {
            let mut b = Body::default();

            b.s.kind = buf.read_utf();
            b.s.n = buf.read_utf();
            b.p = [buf.read_double(), buf.read_double()];
            b.a = buf.read_double();
            b.s.fric = buf.read_double();
            b.s.fricp = buf.read_boolean();
            b.s.re = buf.read_double();
            b.s.de = buf.read_double();
            b.lv = [buf.read_double(), buf.read_double()];
            b.av = buf.read_double();
            b.s.ld = buf.read_double();
            b.s.ad = buf.read_double();
            b.s.fr = buf.read_boolean();
            b.s.bu = buf.read_boolean();

            b.cf.x = buf.read_double();
            b.cf.y = buf.read_double();
            b.cf.ct = buf.read_double();
            b.cf.w = buf.read_boolean();

            b.s.f_c = buf.read_short();
            b.s.f_1 = buf.read_boolean();
            b.s.f_2 = buf.read_boolean();
            b.s.f_3 = buf.read_boolean();
            b.s.f_4 = buf.read_boolean();

            if map.v >= 2 {
                b.s.f_p = buf.read_boolean();
            }
            if map.v >= 14 {
                b.fz.on = buf.read_boolean();

                if b.fz.on {
                    b.fz.x = buf.read_double();
                    b.fz.y = buf.read_double();
                    b.fz.d = buf.read_boolean();
                    b.fz.p = buf.read_boolean();
                    b.fz.a = buf.read_boolean();

                    if map.v >= 15 {
                        b.fz.t = buf.read_short();
                        b.fz.cf = buf.read_double();
                    }
                }
            }

            let fx_count = buf.read_short().max(0) as usize;
            b.fx.reserve(fx_count);

            for _ in 0..fx_count {
                b.fx.push(buf.read_short());
            }

            map.physics.bodies.push(b);
        }

        let spawns_count = buf.read_short().max(0) as usize;
        map.spawns.reserve(spawns_count);
        for _ in 0..spawns_count {
            let mut s = Spawn::default();

            s.x = buf.read_double();
            s.y = buf.read_double();
            s.xv = buf.read_double();
            s.yv = buf.read_double();
            s.priority = buf.read_short();
            s.r = buf.read_boolean();
            s.f = buf.read_boolean();
            s.b = buf.read_boolean();
            s.gr = Some(buf.read_boolean());
            s.ye = Some(buf.read_boolean());
            s.n = buf.read_utf();

            map.spawns.push(s);
        }

        let cap_zones_count = buf.read_short().max(0) as usize;
        map.cap_zones.reserve(cap_zones_count);

        for _ in 0..cap_zones_count {
            let mut cz = CapZone::default();

            cz.n = buf.read_utf();
            cz.l = buf.read_double();
            cz.i = buf.read_short();

            if map.v >= 6 {
                cz.ty = buf.read_short();
            }

            map.cap_zones.push(cz);
        }

        let joints_count = buf.read_short().max(0) as usize;
        map.physics.joints.reserve(joints_count);

        for _ in 0..joints_count {
            let joint_type = buf.read_short();
            let mut j = Joint::default();

            if joint_type == 1 {
                j = Joint::new_revolute(None, None);
                let d = j.d.get_or_insert_with(JointProperties::default);
                d.la = Some(buf.read_double());
                d.ua = Some(buf.read_double());
                d.mmt = Some(buf.read_double());
                d.ms = Some(buf.read_double());
                d.el = Some(buf.read_boolean());
                d.em = Some(buf.read_boolean());
                j.aa = Some([buf.read_double(), buf.read_double()]);
            } else if joint_type == 2 {
                j = Joint::new_distance(None, None);
                let d = j.d.get_or_insert_with(JointProperties::default);
                d.fh = Some(buf.read_double());
                d.dr = Some(buf.read_double());
                j.aa = Some([buf.read_double(), buf.read_double()]);
                j.ab = Some([buf.read_double(), buf.read_double()]);
            } else if joint_type == 3 {
                j = Joint::new_lpj(None, None);
                j.pax = Some(buf.read_double());
                j.pay = Some(buf.read_double());
                j.pa = Some(buf.read_double());
                j.pf = Some(buf.read_double());
                j.pl = Some(buf.read_double());
                j.pu = Some(buf.read_double());
                j.plen = Some(buf.read_double());
                j.pms = Some(buf.read_double());
            } else if joint_type == 4 {
                j = Joint::new_lsj(None, None);
                j.sax = Some(buf.read_double());
                j.say = Some(buf.read_double());
                j.sf = Some(buf.read_double());
                j.slen = Some(buf.read_double());
            } else if joint_type == 5 {
                j = Joint::new_gear();
                j.n = Some(buf.read_utf());
                j.ja = Some(buf.read_short());
                j.jb = Some(buf.read_short());
                j.r = Some(buf.read_double());
            }

            if joint_type != 5 {
                j.ba = Some(buf.read_short());
                j.bb = Some(buf.read_short());
                let d = j.d.get_or_insert_with(JointProperties::default);
                d.cc = buf.read_boolean();
                d.bf = Some(buf.read_double());
                d.dl = Some(buf.read_boolean());
            }
            map.physics.joints.push(j);
        }

        Ok(map)
    }

    pub fn from_old_format(map_data: &Value) -> Result<Bonk2Map, String> {
        let mut map = Self::get_blank_map();

        if let Some(author) = map_data.get("author").and_then(Value::as_str) {
            map.m.a = author.to_string();
        }
        if let Some(mapname) = map_data.get("mapname").and_then(Value::as_str) {
            map.m.n = mapname.to_string();
        }

        map.s.fl = map_data
            .get("discAllForce")
            .and_then(Value::as_i64)
            .map(|v| v == 20)
            .unwrap_or(false);
        map.s.a1 = Some(true);
        map.s.a2 = Some(false);
        map.s.a3 = Some(false);

        if let Some(respawn) = map_data.get("respawn").and_then(Value::as_bool) {
            map.s.re = respawn;
        }
        if let Some(no_collide) = map_data.get("noCollide").and_then(Value::as_bool) {
            map.s.nc = no_collide;
        }
        if let Some(ppm) = map_data.get("ppm").and_then(Value::as_i64) {
            map.physics.ppm = ppm as i16;
        }

        let spawn_names = map_data.get("spawnNames").and_then(Value::as_array);

        if let Some(spawn_array) = map_data.get("spawnArray").and_then(Value::as_array) {
            map.spawns.reserve(spawn_array.len());
            for (i, spawn) in spawn_array.iter().enumerate() {
                map.spawns.push(Spawn {
                    x: spawn.get("x").and_then(Value::as_f64).unwrap_or(400.0),
                    y: spawn.get("y").and_then(Value::as_f64).unwrap_or(300.0),
                    xv: spawn.get("xv").and_then(Value::as_f64).unwrap_or(0.0),
                    yv: spawn.get("yv").and_then(Value::as_f64).unwrap_or(0.0),
                    priority: spawn.get("priority").and_then(Value::as_i64).unwrap_or(5) as i16,
                    r: spawn.get("red").and_then(Value::as_bool).unwrap_or(true),
                    f: spawn.get("ffa").and_then(Value::as_bool).unwrap_or(true),
                    b: spawn.get("blue").and_then(Value::as_bool).unwrap_or(true),
                    gr: None,
                    ye: None,
                    n: spawn_names
                        .and_then(|names| names.get(i))
                        .and_then(Value::as_str)
                        .unwrap_or("Spawn")
                        .to_string(),
                });
            }
        }

        let platform_names = map_data.get("platformNames").and_then(Value::as_array);

        if let Some(platforms) = map_data.get("platformArray").and_then(Value::as_array) {
            map.physics.bodies.reserve(platforms.len());
            for (i, body_data) in platforms.iter().enumerate() {
                let mut body = Self::get_new_body();

                body.s.kind = if body_data
                    .get("isDynamic")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    "d".to_string()
                } else {
                    "s".to_string()
                };
                body.s.n = platform_names
                    .and_then(|names| names.get(i))
                    .and_then(Value::as_str)
                    .unwrap_or("Unnamed")
                    .to_string();
                body.p = [
                    body_data.get("x").and_then(Value::as_f64).unwrap_or(0.0),
                    body_data.get("y").and_then(Value::as_f64).unwrap_or(0.0),
                ];
                body.lv = [
                    body_data.get("xv").and_then(Value::as_f64).unwrap_or(0.0),
                    body_data.get("yv").and_then(Value::as_f64).unwrap_or(0.0),
                ];
                body.a = body_data
                    .get("angle")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                body.av = body_data
                    .get("angularVelocity")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                body.s.ld = body_data
                    .get("linearDamping")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                body.s.ad = body_data
                    .get("angularDamping")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                body.s.de = body_data
                    .get("density")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.3);
                body.s.fric = body_data
                    .get("friction")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.3);
                body.s.re = body_data
                    .get("restitution")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.8);

                if let Some(shape_array) = body_data.get("shapeArray").and_then(Value::as_array) {
                    body.fx.reserve(shape_array.len());
                    for shape_data in shape_array {
                        let width = shape_data
                            .get("width")
                            .and_then(Value::as_f64)
                            .unwrap_or(0.0)
                            .abs();
                        let height = shape_data
                            .get("height")
                            .and_then(Value::as_f64)
                            .unwrap_or(0.0)
                            .abs();
                        let shape_id = shape_data
                            .get("shapeID")
                            .and_then(Value::as_i64)
                            .unwrap_or(2);
                        let mut shape = match shape_id {
                            1 => {
                                let mut s = Self::get_new_circle_shape();

                                s.r = Some(width);

                                s
                            }
                            2 | 4 => {
                                let mut s = Self::get_new_box_shape();

                                s.w = Some(width * 2.0);
                                s.h = Some(height * 2.0);
                                s.sk = Some(shape_id == 4);

                                s
                            }
                            5 => {
                                let mut s = Self::get_new_poly_shape();

                                s.v = Some(vec![
                                    [width, 0.0],
                                    [-0.5 * width, height],
                                    [-0.5 * width, -height],
                                ]);

                                s
                            }
                            6 => {
                                let mut s = Self::get_new_poly_shape();
                                let min_dimension = width.min(height);
                                let corner_inset =
                                    ((map.physics.ppm as f64) * 2.0).min(min_dimension * 0.4);
                                let inner_x = width - corner_inset;
                                let inner_y = height - corner_inset;

                                s.v = Some(vec![
                                    [-width, -inner_y],
                                    [-inner_x, -height],
                                    [inner_x, -height],
                                    [width, -inner_y],
                                    [width, inner_y],
                                    [inner_x, height],
                                    [-inner_x, height],
                                    [-width, inner_y],
                                ]);

                                s
                            }
                            _ => Self::get_new_box_shape(),
                        };

                        shape.c = [
                            shape_data
                                .get("localX")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0),
                            shape_data
                                .get("localY")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0),
                        ];
                        shape.a = Some(
                            shape_data
                                .get("localAngle")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0),
                        );

                        map.physics.shapes.push(shape);

                        let mut fixture = Self::get_new_fixture(None);

                        fixture.n = String::new();
                        fixture.fr = None;
                        fixture.fp = None;
                        fixture.re = None;
                        fixture.de = None;
                        fixture.sn = Some(false);
                        fixture.f = shape_data
                            .get("color")
                            .and_then(Value::as_u64)
                            .unwrap_or(0x4F7CAC) as u32;
                        fixture.d = shape_data
                            .get("death")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        fixture.np = shape_data
                            .get("noPhysics")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        fixture.sh = (map.physics.shapes.len() - 1) as i16;

                        map.physics.fixtures.push(fixture);
                        body.fx.push((map.physics.fixtures.len() - 1) as i16);
                    }
                }

                map.physics.bodies.push(body);
            }
        }

        map.physics.bro.reserve(map.physics.bodies.len());
        for i in (0..map.physics.bodies.len()).rev() {
            map.physics.bro.push(i as i16);
        }

        let cap_zone_names = map_data.get("capZoneNames").and_then(Value::as_array);

        if let Some(cap_zone_array) = map_data.get("capZoneArray").and_then(Value::as_array) {
            map.cap_zones.reserve(cap_zone_array.len());
            for (i, cap_zone) in cap_zone_array.iter().enumerate() {
                let mut shape = Self::get_new_circle_shape();

                shape.r = Some(
                    cap_zone
                        .get("radius")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0)
                        * map.physics.ppm as f64,
                );

                let mut body = Self::get_new_body();

                body.n = Some("Cap Zone Body".to_string());
                body.fx = vec![map.physics.fixtures.len() as i16];
                body.p = [
                    cap_zone.get("x").and_then(Value::as_f64).unwrap_or(0.0),
                    cap_zone.get("y").and_then(Value::as_f64).unwrap_or(0.0),
                ];
                body.s.f_1 = false;
                body.s.f_2 = false;
                body.s.f_3 = false;
                body.s.f_4 = false;

                let fixture = Self::get_new_fixture(Some(map.physics.shapes.len() as i16));

                map.physics.shapes.push(shape);
                map.physics.bodies.push(body);
                map.physics.fixtures.push(fixture);
                map.cap_zones.push(CapZone {
                    n: cap_zone_names
                        .and_then(|names| names.get(i))
                        .and_then(Value::as_str)
                        .unwrap_or("Cap Zone")
                        .to_string(),
                    l: cap_zone
                        .get("captureLimit")
                        .and_then(Value::as_f64)
                        .unwrap_or(10.0),
                    i: (map.physics.fixtures.len() - 1) as i16,
                    ty: 1,
                });
                map.physics
                    .bro
                    .insert(0, (map.physics.bodies.len() - 1) as i16);
            }
        }

        Ok(map)
    }

    pub fn merge_into_new_map(map_data: Bonk2Map) -> Bonk2Map {
        let mut map = Self::get_blank_map();

        map.v = map_data.v;
        map.s = map_data.s;
        map.m = map_data.m;
        map.physics = map_data.physics;
        map.spawns = map_data.spawns;
        map.cap_zones = map_data.cap_zones;

        if map.physics.ppm < 5 {
            map.physics.ppm = 5;
        }
        if map.physics.ppm > 30 {
            map.physics.ppm = 30;
        }

        map
    }
}

fn to_py_value_error(err: String) -> PyErr {
    PyValueError::new_err(err)
}

#[pyfunction(name = "get_blank_map")]
pub fn py_get_blank_map() -> Bonk2Map {
    MapDataHandler::get_blank_map()
}

#[pyfunction(name = "get_new_body")]
pub fn py_get_new_body() -> Body {
    MapDataHandler::get_new_body()
}

#[pyfunction(name = "get_new_fixture", signature = (shape_id=None))]
pub fn py_get_new_fixture(shape_id: Option<i16>) -> Fixture {
    MapDataHandler::get_new_fixture(shape_id)
}

#[pyfunction(name = "get_new_box_shape")]
pub fn py_get_new_box_shape() -> Shape {
    MapDataHandler::get_new_box_shape()
}

#[pyfunction(name = "get_new_circle_shape")]
pub fn py_get_new_circle_shape() -> Shape {
    MapDataHandler::get_new_circle_shape()
}

#[pyfunction(name = "get_new_poly_shape")]
pub fn py_get_new_poly_shape() -> Shape {
    MapDataHandler::get_new_poly_shape()
}

#[pyfunction(name = "get_new_chain_shape")]
pub fn py_get_new_chain_shape() -> Shape {
    MapDataHandler::get_new_chain_shape()
}

#[pyfunction(name = "get_new_revolute_joint", signature = (a_id=None, b_id=None))]
pub fn py_get_new_revolute_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
    MapDataHandler::get_new_revolute_joint(a_id, b_id)
}

#[pyfunction(name = "get_new_distance_joint", signature = (a_id=None, b_id=None))]
pub fn py_get_new_distance_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
    MapDataHandler::get_new_distance_joint(a_id, b_id)
}

#[pyfunction(name = "get_new_lpj_joint", signature = (a_id=None, b_id=None))]
pub fn py_get_new_lpj_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
    MapDataHandler::get_new_lpj_joint(a_id, b_id)
}

#[pyfunction(name = "get_new_lsj_joint", signature = (a_id=None, b_id=None))]
pub fn py_get_new_lsj_joint(a_id: Option<i16>, b_id: Option<i16>) -> Joint {
    MapDataHandler::get_new_lsj_joint(a_id, b_id)
}

#[pyfunction(name = "get_new_gear_joint")]
pub fn py_get_new_gear_joint() -> Joint {
    MapDataHandler::get_new_gear_joint()
}

#[pyfunction(name = "get_new_cap_zone")]
pub fn py_get_new_cap_zone() -> CapZone {
    MapDataHandler::get_new_cap_zone()
}

#[pyfunction(name = "get_new_spawn")]
pub fn py_get_new_spawn() -> Spawn {
    MapDataHandler::get_new_spawn()
}

#[pyfunction(name = "pson_encode")]
pub fn py_pson_encode(map: Bonk2Map) -> PyResult<Vec<u8>> {
    MapDataHandler::pson_encode(&map).map_err(to_py_value_error)
}

#[pyfunction(name = "pson_decode")]
pub fn py_pson_decode(map_data: Vec<u8>) -> PyResult<Bonk2Map> {
    MapDataHandler::pson_decode(&map_data).map_err(to_py_value_error)
}

#[pyfunction(name = "decode_from_old_b64_string")]
pub fn py_decode_from_old_b64_string(old_string: &str) -> PyResult<Bonk2Map> {
    MapDataHandler::decode_from_old_b64_string(old_string).map_err(to_py_value_error)
}

#[pyfunction(name = "decode_from_b64_string")]
pub fn py_decode_from_b64_string(encoded_string: &str) -> PyResult<Bonk2Map> {
    MapDataHandler::decode_from_b64_string(encoded_string).map_err(to_py_value_error)
}

#[pyfunction(name = "encode_to_b64_string")]
pub fn py_encode_to_b64_string(mut map: Bonk2Map) -> String {
    MapDataHandler::encode_to_b64_string(&mut map)
}
