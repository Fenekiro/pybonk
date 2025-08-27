from dataclasses import dataclass
from typing import List, TYPE_CHECKING

from pybonk.game.game_inputs import GameInput
from pybonk.game.game import Match, Player

if TYPE_CHECKING:
    from pybonk.game.game import Game


@dataclass
class PlayerMove:
    input_keys: List[GameInput]
    game: "Game"
    match: Match
    player: Player
    frame: int
    sequence_number: int
