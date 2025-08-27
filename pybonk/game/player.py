from typing import Union, TYPE_CHECKING

import socketio

from pybonk.game.teams import Team
from pybonk.avatar.avatar import Avatar
from pybonk.game.exceptions import GameConnectionError

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot import BonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.game import Game


class Player:
    """
    Class that holds bonk.io game players' info.

    :param bot: bot class that uses in the same game with player.
    :param game: the game which player is playing in.
    :param is_bot: indicates whether player is bot or not.
    :param peer_id: player's game peer id.
    :param username: player's username.
    :param is_guest: indicates whether player is guest or not.
    :param level: player's level.
    :param is_ready: indicates whether player is ready or not.
    :param is_tabbed: indicates whether player is tabbed or not.
    :param team: Teams' class that indicates player's team.
    :param short_id: player's id in the game.
    :param avatar: player's avatar.
    """

    def __init__(
        self,
        bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]",
        game: "Game",
        is_bot: bool,
        is_host: bool,
        peer_id: str,
        username: str,
        is_guest: bool,
        level: int,
        is_ready: bool,
        is_tabbed: bool,
        team: Team,
        short_id: int,
        avatar: Avatar
    ) -> None:
        self.bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]" = bot
        self.is_bot: bool = is_bot
        self.is_host = is_host
        self.game: "Game" = game
        self.username: str = username
        self.is_guest: bool = is_guest
        self.level: int = level
        self.is_ready: bool = is_ready
        self.is_tabbed: bool = is_tabbed
        self.ping: Union[int, None] = None
        self.team: Team = team
        self.balanced_by: int = 0
        self.short_id: int = short_id
        self.avatar: Avatar = avatar
        self.__socket_client: socketio.AsyncClient = self.game.socket_client
        self.__peer_id: str = peer_id

    async def kick(self) -> None:
        """Kick player from game."""

        if not self.game.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot kick player, bot is not a host", self.game)
            )
        else:
            await self.__socket_client.emit(
                9,
                {
                    "banshortid": self.short_id,
                    "kickonly": True
                }
            )

    async def ban(self) -> None:
        """Ban player from game."""

        if not self.game.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot ban player, bot is not a host", self.game)
            )

        await self.__socket_client.emit(
            9,
            {
                "banshortid": self.short_id,
                "kickonly": False
            }
        )

    async def move_to_team(self, team: Team) -> None:
        """
        Move player to another team.

        :param team: Teams class that indicates player's team.
        """

        if not self.game.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot move player to team, bot is not a host", self.game)
            )
        else:
            await self.__socket_client.emit(
                26,
                {
                    "targetID": self.short_id,
                    "targetTeam": team.value
                }
            )

    async def balance(self, percents: int) -> None:
        """
        Nerf/buff player.

        :param percents: the percent you want to balance player by (in range [-100, 100]).
        """

        if not self.game.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot balance player, bot is not a host", self.game)
            )
        else:
            if not (percents in range(-100, 101)):
                raise ValueError("Can't balance player: percents param is not in range [-100, 100]")

            await self.__socket_client.emit(
                29,
                {
                    "sid": self.short_id,
                    "bal": percents
                }
            )

    async def give_host(self) -> None:
        """Give the host permissions to player."""

        if not self.game.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot give host, bot is not a host", self.game)
            )
        else:
            await self.__socket_client.emit(
                34,
                {
                    "id": self.short_id
                }
            )

    async def send_friend_request(self) -> None:
        """Send friend request to the player."""

        await self.__socket_client.emit(
            35,
            {
                "id": self.short_id
            }
        )
