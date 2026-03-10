from dataclasses import dataclass
from typing import Union, TYPE_CHECKING

from pybonk.game.player import Player

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot_base import BonkBotBase
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.game import Game


@dataclass
class Message:
    game: "Game"
    bot: "Union[BonkBotBase, GuestBonkBot, AccountBonkBot]"
    author: Player
    content: str
