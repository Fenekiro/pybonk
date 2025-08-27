from dataclasses import dataclass
from typing import Union, TYPE_CHECKING

from pybonk.game.player import Player

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot import BonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.game import Game


@dataclass
class MapRequestAsClient:
    game: "Game"
    bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]"
    map_name: str
    author: str
    player_requested: Player
