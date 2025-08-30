from typing import TYPE_CHECKING, Union
from dataclasses import dataclass

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.bonk_bot import BonkBot


@dataclass
class Bonk1Map:
    bot: "Union[AccountBonkBot, GuestBonkBot, BonkBot]"
    map_id: int
    encoded_data: str
    name: str
    author_name: str
    creation_date: str
    modified_date: str
    votes_up: int
    votes_down: int
