from typing import List, Optional, Sequence, Union

BytesLike = Union[bytes, bytearray, Sequence[int]]


class AvatarLayer:
    id: int
    scale: float
    angle: float
    x: float
    y: float
    flip_x: bool
    flip_y: bool
    color: int

    def __init__(
        self,
        id: int = ...,
        scale: float = ...,
        angle: float = ...,
        x: float = ...,
        y: float = ...,
        flip_x: bool = ...,
        flip_y: bool = ...,
        color: int = ...,
    ) -> None: ...


class Avatar:
    layers: List[AvatarLayer]
    base_color: int

    def __init__(self) -> None: ...

    @staticmethod
    def from_encoded(encoded_string: str) -> "Avatar": ...

    @staticmethod
    def from_json_string(avatar_json: str) -> "Avatar": ...

    def load_from_json_string(self, avatar_json: str) -> None: ...
    def to_json_string(self) -> str: ...
    def to_dict(self) -> dict: ...


class MapMetadata:
    a: str
    n: str
    dbv: int
    dbid: int
    rxa: str
    rxn: str
    rxdb: int
    rxid: int
    is_pub: bool
    cr: List[str]
    date: str
    authid: int
    mo: str
    vd: Optional[int]
    vu: Optional[int]


class MapProperties:
    gd: float
    nc: bool
    re: bool
    fl: bool
    pq: int
    a1: Optional[bool]
    a2: Optional[bool]
    a3: Optional[bool]


class BodyForce:
    x: float
    y: float
    w: bool
    ct: float


class BodyForceZoneProperties:
    on: bool
    x: float
    y: float
    t: int
    d: bool
    p: bool
    a: bool
    cf: float


class BodyShape:
    kind: str
    n: str
    fricp: bool
    ld: float
    ad: float
    de: float
    fric: float
    re: float
    fr: bool
    bu: bool
    f_1: bool
    f_2: bool
    f_3: bool
    f_4: bool
    f_c: int
    f_p: bool


class Body:
    n: Optional[str]
    a: float
    av: float
    fx: List[int]
    p: Sequence[float]
    lv: Sequence[float]
    cf: BodyForce
    fz: BodyForceZoneProperties
    s: BodyShape


class Shape:
    kind: str
    a: Optional[float]
    h: Optional[float]
    w: Optional[float]
    sk: Optional[bool]
    r: Optional[float]
    v: Optional[List[Sequence[float]]]
    s: Optional[float]
    l: Optional[bool]
    c: Sequence[float]


class Fixture:
    sh: int
    n: str
    fr: Optional[float]
    fp: Optional[bool]
    re: Optional[float]
    de: Optional[float]
    f: int
    d: bool
    np: bool
    ng: bool
    ig: Optional[bool]
    sn: Optional[bool]
    fs: Optional[str]
    zp: Optional[float]


class JointProperties:
    em: Optional[bool]
    el: Optional[bool]
    bf: Optional[float]
    la: Optional[float]
    ua: Optional[float]
    mmt: Optional[float]
    ms: Optional[float]
    dr: Optional[float]
    fh: Optional[float]
    cc: bool
    dl: Optional[bool]
    lt: Optional[float]
    ut: Optional[float]
    mmf: Optional[float]
    cd: Optional[bool]


class Joint:
    kind: str
    n: Optional[str]
    ba: Optional[int]
    bb: Optional[int]
    aa: Optional[Sequence[float]]
    ab: Optional[Sequence[float]]
    d: Optional[JointProperties]
    pa: Optional[float]
    pax: Optional[float]
    pay: Optional[float]
    sax: Optional[float]
    say: Optional[float]
    pf: Optional[float]
    pl: Optional[float]
    plen: Optional[float]
    pms: Optional[float]
    pu: Optional[float]
    sf: Optional[float]
    slen: Optional[float]
    ja: Optional[int]
    jb: Optional[int]
    r: Optional[float]
    axa: Optional[Sequence[float]]


class MapPhysics:
    bodies: List[Body]
    fixtures: List[Fixture]
    joints: List[Joint]
    shapes: List[Shape]
    bro: List[int]
    ppm: int


class Spawn:
    x: float
    y: float
    xv: float
    yv: float
    priority: int
    r: bool
    f: bool
    b: bool
    gr: Optional[bool]
    ye: Optional[bool]
    n: str


class CapZone:
    n: str
    ty: int
    l: float
    i: int


class Bonk2Map:
    v: int
    m: MapMetadata
    s: MapProperties
    physics: MapPhysics
    spawns: List[Spawn]
    cap_zones: List[CapZone]

    def __init__(self) -> None: ...

    @staticmethod
    def from_dict(data: dict) -> Bonk2Map: ...

    def to_dict(self) -> dict: ...


def decode_avatar(encoded_avatar: str) -> Avatar: ...

def get_blank_map() -> Bonk2Map: ...
def get_new_body() -> Body: ...
def get_new_fixture(shape_id: Optional[int] = ...) -> Fixture: ...
def get_new_box_shape() -> Shape: ...
def get_new_circle_shape() -> Shape: ...
def get_new_poly_shape() -> Shape: ...
def get_new_chain_shape() -> Shape: ...
def get_new_revolute_joint(a_id: Optional[int] = ..., b_id: Optional[int] = ...) -> Joint: ...
def get_new_distance_joint(a_id: Optional[int] = ..., b_id: Optional[int] = ...) -> Joint: ...
def get_new_lpj_joint(a_id: Optional[int] = ..., b_id: Optional[int] = ...) -> Joint: ...
def get_new_lsj_joint(a_id: Optional[int] = ..., b_id: Optional[int] = ...) -> Joint: ...
def get_new_gear_joint() -> Joint: ...
def get_new_cap_zone() -> CapZone: ...
def get_new_spawn() -> Spawn: ...
def pson_encode(map: Bonk2Map) -> bytes: ...
def pson_decode(map_data: BytesLike) -> Bonk2Map: ...
def decode_from_old_b64_string(old_string: str) -> Bonk2Map: ...
def decode_from_b64_string(encoded_string: str) -> Bonk2Map: ...
def encode_to_b64_string(map: Bonk2Map) -> str: ...
