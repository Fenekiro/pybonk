from dataclasses import dataclass
from typing import List, TYPE_CHECKING

from pybonk.game.player_input import PlayerInput
from pybonk.game.game import Match, Player

if TYPE_CHECKING:
    from pybonk.game.game import Game


@dataclass
class PlayerMove:
    input_keys: List[PlayerInput]
    game: "Game"
    match: Match
    player: Player
    frame: int
    sequence_number: int
