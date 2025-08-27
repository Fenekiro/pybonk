from dataclasses import dataclass


@dataclass
class Layer:
    id: int
    scale: float
    angle: float
    x: float
    y: float
    flip_x: bool
    flip_y: bool
    color: int
