from typing import NamedTuple


class GameMode(NamedTuple):
    engine: str  # ga
    short_name: str


Classic: GameMode = GameMode("b", "b")
Simple: GameMode = GameMode("b", "bs")
Arrows: GameMode = GameMode("b", "ar")
DeathArrows: GameMode = GameMode("b", "ard")
Grapple: GameMode = GameMode("b", "sp")
VTOL: GameMode = GameMode("b", "v")
Football = GameMode("f", "f")
