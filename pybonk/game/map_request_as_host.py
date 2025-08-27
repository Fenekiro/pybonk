from typing import Union, TYPE_CHECKING

from pybonk.parsers.parsers import decode_bonk_map_metadata
from pybonk.game.exceptions import GameConnectionError
from pybonk.game.player import Player
from pybonk.maps.bonk2_map import Bonk2Map

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot import BonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.game import Game


class MapRequestAsHost:
    """
    Class that holds info about requested maps from host side.

    :param game: game where map was sent.
    :param player_requested: Player that requested the map.
    :param level_data: base64 encoded map data string.
    """

    def __init__(
        self,
        game: "Game",
        bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]",
        player_requested: Player,
        level_data: str
    ) -> None:
        self.game: "Game" = game
        self.bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]" = bot
        self.player_requested: Player = player_requested
        self.level_data: str = level_data

    @property
    def bonk_map(self) -> Bonk2Map:
        map_decoded_data = decode_bonk_map_metadata(self.level_data)

        return Bonk2Map(
            self.bot,
            map_decoded_data["m"]["dbid"],
            self.level_data,
            map_decoded_data["m"]["n"],
            map_decoded_data["m"]["a"],
            map_decoded_data["m"]["date"],
            map_decoded_data["m"]["vu"],
            map_decoded_data["m"]["vd"]
        )

    async def load(self) -> None:
        """Sets game map from map request."""

        if not self.game.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set map, bot is not a host", self.game)
            )
        else:
            await self.game.socket_client.emit(
                23,
                {
                    "m": self.level_data
                }
            )
            self.game._bonk_map = self.bonk_map
