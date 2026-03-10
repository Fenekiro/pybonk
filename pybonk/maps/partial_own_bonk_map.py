from functools import cached_property
from typing import TYPE_CHECKING
from dataclasses import dataclass

from pybonk.settings import endpoints
from pybonk.maps.data_handler import Bonk2Map, decode_from_b64_string

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot


@dataclass
class PartialOwnBonkMap:
    bot: "AccountBonkBot"
    map_id: int
    encoded_data: str
    name: str
    creation_date: str
    is_published: bool
    votes_up: int
    votes_down: int

    @cached_property
    def decoded_data(self) -> Bonk2Map:
        """Decodes full map data"""

        return decode_from_b64_string(self.encoded_data)

    async def delete(self) -> None:
        """Deletes bot's account own map."""

        await self.bot.aiohttp_session.post(
            url=endpoints["map_delete"],
            data={
                "token": self.bot.token,
                "mapid": self.map_id,
            }
        )
