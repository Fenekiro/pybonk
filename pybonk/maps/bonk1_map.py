from typing import TYPE_CHECKING
from dataclasses import dataclass

if TYPE_CHECKING:
    from pybonk.bot import AccountBonkBot


@dataclass
class Bonk1Map:
    bot: "AccountBonkBot"
    map_id: int
    encoded_data: str
    name: str
    author_name: str
    creation_date: str
    modified_date: str
    votes_up: int
    votes_down: int
