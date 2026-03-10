from typing import Union, TYPE_CHECKING

from pybonk.maps.data_handler import Bonk2Map, decode_from_b64_string
from pybonk.game.exceptions import GameConnectionError
from pybonk.game.player import Player

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot_base import BonkBotBase
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.game import Game


class MapRequestHostPov:
    """
    Class that holds info about requested maps from host side.

    :param game: game where map was sent.
    :param player_requested: Player that requested the map.
    :param level_data: base64 encoded map data string.
    """

    def __init__(
        self,
        game: "Game",
        bot: "Union[BonkBotBase, GuestBonkBot, AccountBonkBot]",
        player_requested: Player,
        level_data: str
    ) -> None:
        self.game: "Game" = game
        self.bot: "Union[BonkBotBase, GuestBonkBot, AccountBonkBot]" = bot
        self.player_requested: Player = player_requested
        self.level_data: str = level_data

    @property
    def bonk_map(self) -> Bonk2Map:
        return decode_from_b64_string(self.level_data)

    async def load(self) -> None:
        """Sets game map from map request."""

        if not self.game.is_host:
            await self.bot.event_emitter.emit_async(
                "error",
                GameConnectionError("Cannot set map, bot is not a host", self.game)
            )
        else:
            await self.game.socket_client.emit(23, {"m": self.level_data})

            self.game._bonk_map = self.bonk_map
