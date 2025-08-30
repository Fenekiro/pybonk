from typing import TYPE_CHECKING, Union

from pybonk.game.game import Game
from pybonk.types.game_modes import GameMode
from pybonk.parsers import mode_from_short_name

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot import BonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot


class Room:
    """
    Class for rooms from bonk.io room list.

    :param bot: bot class that loaded the room.
    :param room_id: database ID of room.
    :param name: name of the room.
    :param players: current amount of players that play in this room.
    :param max_players: maximal amount of players that can play in this room at the same time.
    :param has_password: whether room has password or not.
    :param mode: the mode that is currently played in the room.
    :param min_level: the minimal level that is required to join the room.
    :param max_level: the maximal level along with you can join the room.
    """

    def __init__(
        self,
        bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]",
        room_id: int,
        name: str,
        players: int,
        max_players: int,
        has_password: bool,
        mode: str,
        min_level: int,
        max_level: int
    ) -> None:
        self.bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]" = bot
        self.id: int = room_id
        self.name: str = name
        self.players: int = players
        self.max_players: int = max_players
        self.has_password: bool = has_password
        self._mode: str = mode
        self.min_level: int = min_level
        self.max_level: int = max_level

    @property
    def mode(self) -> GameMode:
        return mode_from_short_name(self._mode)

    async def join(self, password="") -> Game:
        """
        Joins game from room list.

        :param password: password to join room.

        Example usage::

            bot = bonk_account_login("name", "pass")

            async def main():
                room = [room for room in await bot.get_rooms() if room.name == "test room"][0]
                game = await room.join()

                await bot.run()

            asyncio.run(main())
        """

        return Game(
            self.bot,
            None,
            self.name,
            False,
            self.mode,
            False,
            False,
            False,
            game_join_params=[self.id, password]
        )
