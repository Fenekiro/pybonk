from enum import IntEnum


class GameInput(IntEnum):
    NONE = 0
    LEFT = 1
    RIGHT = 2
    UP = 4
    DOWN = 8
    HEAVY = 16
    SPECIAL = 32
