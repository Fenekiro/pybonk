from dataclasses import dataclass
from functools import cached_property
from typing import TYPE_CHECKING, Union

from pybonk.parsers import decode_bonk_map

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.bonk_bot import BonkBot


@dataclass
class Bonk2Map:
    bot: "Union[AccountBonkBot, GuestBonkBot, BonkBot]"
    map_id: int
    encoded_data: str
    name: str
    author_name: str
    creation_date: str
    votes_up: int
    votes_down: int

    @cached_property
    def decoded_data(self) -> dict:
        """Decodes map data to dict."""

        return decode_bonk_map(self.encoded_data)
