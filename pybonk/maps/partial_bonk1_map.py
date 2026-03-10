from functools import cached_property
from typing import TYPE_CHECKING, Union
from dataclasses import dataclass

from pybonk.maps.data_handler import Bonk2Map, decode_from_old_b64_string

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.bonk_bot_base import BonkBotBase


@dataclass
class PartialBonk1Map:
    bot: "Union[AccountBonkBot, GuestBonkBot, BonkBotBase]"
    map_id: int
    encoded_data: str
    name: str
    author_name: str
    creation_date: str
    modified_date: str
    votes_up: int
    votes_down: int

    @cached_property
    def decoded_data(self) -> Bonk2Map:
        """Decodes full map data"""

        return decode_from_old_b64_string(self.encoded_data)

