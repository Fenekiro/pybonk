from typing import Union, TYPE_CHECKING

if TYPE_CHECKING:
    from pybonk.game.game import Game


class GameConnectionError(Exception):
    """Raised when game connection has some error."""

    def __init__(self, message: str, game: "Union[Game, None]") -> None:
        self.message: str = message
        self.game: Union[Game, None] = game
