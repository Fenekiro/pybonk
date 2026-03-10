use pyo3::{pyclass, pymethods, Bound, BoundObject, PyErr, PyResult, Python};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::PyDict;
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bonk2Map {
    pub v: i16, // Map Version
    pub m: MapMetadata,
    pub s: MapProperties,
    pub physics: MapPhysics,
    pub spawns: Vec<Spawn>,
    #[serde(rename = "capZones")]
    pub cap_zones: Vec<CapZone>,
}

impl Default for Bonk2Map {
    fn default() -> Self {
        Self {
            v: 1,
            s: MapProperties::default(),
            physics: MapPhysics::default(),
            spawns: Vec::new(),
            cap_zones: Vec::new(),
            m: MapMetadata::default(),
        }
    }
}

#[pymethods]
impl Bonk2Map {
    #[new]
    pub fn new() -> Self {
        Bonk2Map::default()
    }

    #[staticmethod]
    pub fn from_dict(dict: Bound<'_, pyo3::types::PyAny>) -> PyResult<Self> {
        if !dict.is_instance_of::<PyDict>() {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected dict",
            ));
        }

        depythonize(&dict).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let value: Value = serde_json::to_value(self)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let obj = pythonize(py, &value)?;
        let bound_obj = obj.into_bound();
        let dict = bound_obj
            .downcast_into::<PyDict>()
            .map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>("Result was not a dict"))?;

        Ok(dict)
    }

    pub fn to_json_string(&self) -> PyResult<String> {
        serde_json::to_string(self).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub a: String,
    pub n: String,
    pub dbv: i32,
    pub dbid: i32,
    pub rxa: String,
    pub rxn: String,
    pub rxdb: i32,
    pub rxid: u32,
    #[serde(rename = "pub")]
    pub is_pub: bool,
    pub cr: Vec<String>,
    pub date: String,
    pub authid: i32,
    pub mo: String,
    pub vd: Option<u32>,
    pub vu: Option<u32>,
}

impl Default for MapMetadata {
    fn default() -> Self {
        Self {
            a: "noauthor".to_string(),
            n: "noname".to_string(),
            dbv: 2,
            dbid: -1,
            authid: -1,
            date: "".to_string(),
            rxid: 0,
            rxn: "".to_string(),
            rxa: "".to_string(),
            rxdb: 1,
            cr: Vec::new(),
            is_pub: false,
            mo: "".to_string(),
            vd: None,
            vu: None,
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProperties {
    pub gd: f64,
    pub nc: bool,
    pub re: bool,
    pub fl: bool,
    pub pq: i16,
    pub a1: Option<bool>,
    pub a2: Option<bool>,
    pub a3: Option<bool>,
}

impl Default for MapProperties {
    fn default() -> Self {
        Self {
            re: false,
            nc: false,
            pq: 1,
            gd: 25.0,
            fl: false,
            a1: None,
            a2: None,
            a3: None,
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPhysics {
    pub bodies: Vec<Body>,
    pub fixtures: Vec<Fixture>,
    pub joints: Vec<Joint>,
    pub shapes: Vec<Shape>,
    pub bro: Vec<i16>, // Индексы тел (Body indices)
    pub ppm: i16,
}

impl Default for MapPhysics {
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
            fixtures: Vec::new(),
            bodies: Vec::new(),
            bro: Vec::new(),
            joints: Vec::new(),
            ppm: 12,
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub x: f64,
    pub y: f64,
    pub xv: f64,
    pub yv: f64,
    pub priority: i16,
    pub r: bool,
    pub f: bool,
    pub b: bool,
    pub gr: Option<bool>,
    pub ye: Option<bool>,
    pub n: String,
}

impl Default for Spawn {
    fn default() -> Self {
        Self {
            x: 400.0,
            y: 300.0,
            xv: 0.0,
            yv: 0.0,
            priority: 5,
            r: true,
            f: true,
            b: true,
            gr: Some(false),
            ye: Some(false),
            n: "Spawn".to_string(),
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapZone {
    pub n: String,
    pub ty: i16,
    pub l: f64,
    pub i: i16, // Shape ID | -1 for none
}

impl Default for CapZone {
    fn default() -> Self {
        Self {
            n: "Cap Zone".to_string(),
            ty: 1,
            l: 10.0,
            i: -1,
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub n: Option<String>,
    pub a: f64,
    pub av: f64,
    pub fx: Vec<i16>,
    pub p: [f64; 2],
    pub lv: [f64; 2],
    pub cf: BodyForce,
    pub fz: BodyForceZoneProperties,
    pub s: BodyShape,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            n: None,
            p: [0.0, 0.0],
            a: 0.0,
            lv: [0.0, 0.0],
            av: 0.0,
            cf: BodyForce {
                x: 0.0,
                y: 0.0,
                w: true,
                ct: 0.0,
            },
            fx: Vec::new(),
            fz: BodyForceZoneProperties {
                on: false,
                x: 0.0,
                y: 0.0,
                d: true,
                p: true,
                a: true,
                t: 0,
                cf: 0.0,
            },
            s: BodyShape::default(),
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyForce {
    pub x: f64,
    pub y: f64,
    pub w: bool,
    pub ct: f64,
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyForceZoneProperties {
    pub on: bool,
    pub x: f64,
    pub y: f64,
    pub t: i16,
    pub d: bool,
    pub p: bool,
    pub a: bool,
    pub cf: f64,
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyShape {
    #[serde(rename = "type")]
    pub kind: String,
    pub n: String,
    pub fricp: bool,
    pub ld: f64,
    pub ad: f64,
    pub de: f64,
    pub fric: f64,
    pub re: f64,
    pub fr: bool,
    pub bu: bool,
    pub f_1: bool,
    pub f_2: bool,
    pub f_3: bool,
    pub f_4: bool,
    pub f_c: i16,
    pub f_p: bool,
}

impl Default for BodyShape {
    fn default() -> Self {
        Self {
            kind: "s".to_string(),
            n: "Unnamed".to_string(),
            fric: 0.3,
            fricp: false,
            re: 0.8,
            de: 0.3,
            ld: 0.0,
            ad: 0.0,
            fr: false,
            bu: false,
            f_c: 1,
            f_p: true,
            f_1: true,
            f_2: true,
            f_3: true,
            f_4: true,
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub sh: i16,
    pub n: String,
    pub fr: Option<f64>,
    pub fp: Option<bool>,
    pub re: Option<f64>,
    pub de: Option<f64>,
    pub f: u32,
    pub d: bool,
    pub np: bool,
    pub ng: bool,
    pub ig: Option<bool>,
    pub sn: Option<bool>,
    pub fs: Option<String>,
    pub zp: Option<f64>,
}

impl Fixture {
    pub fn new(shape_id: i16) -> Self {
        Self {
            sh: shape_id,
            n: "Def Fix".to_string(),
            fr: Some(0.3),
            fp: None,
            re: Some(0.8),
            de: Some(0.3),
            f: 0x4F7CAC,
            d: false,
            np: false,
            ng: false,
            ig: None,
            sn: None,
            fs: None,
            zp: None,
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    #[serde(rename = "type")]
    pub kind: String,
    pub a: Option<f64>,
    pub h: Option<f64>,
    pub w: Option<f64>,
    pub sk: Option<bool>,
    pub r: Option<f64>,
    pub v: Option<Vec<[f64; 2]>>,
    pub s: Option<f64>,
    pub l: Option<bool>,
    pub c: [f64; 2],
}

impl Shape {
    pub fn new_box() -> Self {
        Self {
            kind: "bx".to_string(),
            w: Some(10.0),
            h: Some(40.0),
            c: [0.0, 0.0],
            a: Some(0.0),
            sk: Some(false),
            r: None,
            v: None,
            s: None,
            l: None,
        }
    }

    pub fn new_circle() -> Self {
        Self {
            kind: "ci".to_string(),
            r: Some(25.0),
            c: [0.0, 0.0],
            sk: Some(false),
            a: None,
            h: None,
            w: None,
            v: None,
            s: None,
            l: None,
        }
    }

    pub fn new_poly() -> Self {
        Self {
            kind: "po".to_string(),
            v: Some(Vec::new()),
            s: Some(1.0),
            a: Some(0.0),
            c: [0.0, 0.0],
            sk: None,
            r: None,
            h: None,
            w: None,
            l: None,
        }
    }

    pub fn new_chain() -> Self {
        Self {
            kind: "ch".to_string(),
            v: Some(Vec::new()),
            s: Some(1.0),
            a: Some(0.0),
            c: [0.0, 0.0],
            l: Some(false),
            sk: Some(false),
            r: None,
            h: None,
            w: None,
        }
    }
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JointProperties {
    pub em: Option<bool>,
    pub el: Option<bool>,
    pub bf: Option<f64>,
    pub la: Option<f64>,
    pub ua: Option<f64>,
    pub mmt: Option<f64>,
    pub ms: Option<f64>,
    pub dr: Option<f64>,
    pub fh: Option<f64>,
    pub cc: bool,
    pub dl: Option<bool>,
    pub lt: Option<f64>,
    pub ut: Option<f64>,
    pub mmf: Option<f64>,
    pub cd: Option<bool>,
}

#[pyclass(get_all, set_all, from_py_object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    #[serde(rename = "type")]
    pub kind: String,
    pub n: Option<String>,
    pub ba: Option<i16>,
    pub bb: Option<i16>,
    pub aa: Option<[f64; 2]>,
    pub ab: Option<[f64; 2]>,
    pub d: Option<JointProperties>,
    pub pa: Option<f64>,
    pub pax: Option<f64>,
    pub pay: Option<f64>,
    pub sax: Option<f64>,
    pub say: Option<f64>,
    pub pf: Option<f64>,
    pub pl: Option<f64>,
    pub plen: Option<f64>,
    pub pms: Option<f64>,
    pub pu: Option<f64>,
    pub sf: Option<f64>,
    pub slen: Option<f64>,
    pub ja: Option<i16>,
    pub jb: Option<i16>,
    pub r: Option<f64>,
    pub axa: Option<[f64; 2]>,
}

impl Joint {
    pub fn new_revolute(ba: Option<i16>, bb: Option<i16>) -> Self {
        let mut props = JointProperties::default();

        props.la = Some(0.0);
        props.ua = Some(0.0);
        props.mmt = Some(0.0);
        props.ms = Some(0.0);
        props.el = Some(false);
        props.em = Some(false);
        props.cc = false;
        props.bf = Some(0.0);
        props.dl = Some(true);

        Self {
            kind: "rv".to_string(),
            ba,
            bb,
            aa: Some([0.0, 0.0]),
            d: Some(props),
            ..Default::default()
        }
    }

    pub fn new_distance(ba: Option<i16>, bb: Option<i16>) -> Self {
        let mut props = JointProperties::default();

        props.fh = Some(0.0);
        props.dr = Some(0.0);
        props.cc = false;
        props.bf = Some(0.0);
        props.dl = Some(true);

        Self {
            kind: "d".to_string(),
            ba,
            bb,
            aa: Some([0.0, 0.0]),
            ab: Some([0.0, 0.0]),
            d: Some(props),
            ..Default::default()
        }
    }

    pub fn new_lpj(ba: Option<i16>, bb: Option<i16>) -> Self {
        let mut props = JointProperties::default();

        props.cc = false;
        props.bf = Some(0.0);
        props.dl = Some(true);

        Self {
            kind: "lpj".to_string(),
            ba,
            bb,
            pax: Some(0.0),
            pay: Some(0.0),
            pa: Some(0.0),
            pf: Some(0.0),
            pl: Some(0.0),
            pu: Some(0.0),
            plen: Some(0.0),
            pms: Some(0.0),
            d: Some(props),
            ..Default::default()
        }
    }

    pub fn new_lsj(ba: Option<i16>, bb: Option<i16>) -> Self {
        let mut props = JointProperties::default();

        props.cc = false;
        props.bf = Some(0.0);
        props.dl = Some(true);

        Self {
            kind: "lsj".to_string(),
            ba,
            bb,
            sax: Some(0.0),
            say: Some(0.0),
            sf: Some(0.0),
            slen: Some(0.0),
            d: Some(props),
            ..Default::default()
        }
    }

    pub fn new_gear() -> Self {
        Self {
            kind: "g".to_string(),
            n: Some("Gear Joint".to_string()),
            ja: Some(-1),
            jb: Some(-1),
            r: Some(1.0),
            ..Default::default()
        }
    }
}

impl Default for Joint {
    fn default() -> Self {
        Self {
            kind: "".to_string(),
            n: None,
            ba: None,
            bb: None,
            aa: None,
            ab: None,
            d: None,
            pa: None,
            pax: None,
            pay: None,
            sax: None,
            say: None,
            pf: None,
            pl: None,
            plen: None,
            pms: None,
            pu: None,
            sf: None,
            slen: None,
            ja: None,
            jb: None,
            r: None,
            axa: None,
        }
    }
}
