from dataclasses import dataclass
from functools import cached_property
from typing import TYPE_CHECKING

from pybonk.parsers import decode_bonk_map

if TYPE_CHECKING:
    from pybonk.bot import AccountBonkBot


@dataclass
class Bonk2Map:
    bot: "AccountBonkBot"
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
