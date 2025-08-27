import json
from typing import List, Union
from urllib.parse import unquote_plus
import requests
import re
import asyncio
import nest_asyncio
import aiohttp
from functools import cached_property

from ..bonk_online import BonkOnline
from ..settings import PROTOCOL_VERSION, links
from pybonk.maps.bonk1_map import Bonk1Map
from pybonk.maps.bonk2_map import Bonk2Map
from pybonk.room_list.room import Room
from ..parsers import decode_avatar
from pybonk.game.game import Game
from pybonk.types.game_modes import Classic
from pybonk.types.servers import Server, Warsaw
from pybonk.avatar.avatar import Avatar
from ..bot.bot_event_handler import BotEventHandler

nest_asyncio.apply()


class BonkBot(BotEventHandler):
    """
    Base class for AccountBonkBot and GuestBonkBot.

    :param username: bot username.
    :param xp: amount of xp on bot's account.
    """

    def __init__(
        self,
        username: str,
        xp: int,
        avatars: List[str],
        main_avatar: Avatar
    ) -> None:
        super().__init__()
        self.username: str = username
        self.xp: int = xp
        self._raw_avatars: List[str] = avatars
        self.main_avatar: Union[Avatar, None] = main_avatar
        self.games: List[Game] = []
        self.aiohttp_session: Union[aiohttp.ClientSession, None] = None

    @property
    def is_guest(self) -> bool:
        return self.xp == 0

    @property
    def level(self) -> int:
        """Returns account level from its XP."""

        if self.is_guest:
            return 0

        return int((self.xp / 100) ** 0.5 + 1)

    @cached_property
    def avatars(self) -> List[Avatar]:
        """Returns account avatars from encoded base64 strings."""

        avatars = []

        if not (self._raw_avatars is None):
            for raw_avatar in self._raw_avatars:
                avatars.append(decode_avatar(raw_avatar))

            return [decode_avatar(avatar) for avatar in self._raw_avatars]

        return [
            Avatar().load_from_json({"layers": [], "bc": 4492031}),
            Avatar().load_from_json({"layers": [], "bc": 4492031}),
            Avatar().load_from_json({"layers": [], "bc": 4492031}),
            Avatar().load_from_json({"layers": [], "bc": 4492031}),
            Avatar().load_from_json({"layers": [], "bc": 4492031})
        ]

    @property
    def main_avatar(self) -> Avatar:
        return self._main_avatar

    @main_avatar.setter
    def main_avatar(self, avatar: Union[Avatar, None]) -> None:
        """
        Changes bot's account session avatar.

        :param avatar: avatar to change (Avatar class instance). None argument sets default bonk.io avatar.
        """

        if not isinstance(avatar, Avatar) and not (avatar is None):
            raise TypeError("Avatar must be of type Avatar")

        if avatar is None:
            self._main_avatar = Avatar().load_from_json({"layers": [], "bc": 4492031})
        else:
            self._main_avatar = avatar

    def run(self) -> None:
        asyncio.run(self._run())

    async def _run(self) -> None:
        """Prevents room connections from stopping and "starts" the bot."""

        self.aiohttp_session = aiohttp.ClientSession()

        for attr in dir(self):
            if attr.startswith("on"):
                self.event_emitter.on(attr.replace("on_", ""), getattr(self, attr))

        self.event_emitter.emit("ready")

        while not self.aiohttp_session.closed:
            await asyncio.sleep(0.1)

    async def stop(self) -> None:
        """Stops the bot."""

        await self.aiohttp_session.close()

    def event(self, function) -> None:
        """
        Wrapper for async functions to make them handle bonk events.

        :param function: coroutine function to be called.
        """

        if not asyncio.iscoroutinefunction(function):
            raise TypeError("Event handler function is not a coroutine")

        attribute_name = "_" + function.__name__
        handler = getattr(self, attribute_name)

        func_arg_count = function.__code__.co_argcount
        actual_arg_count = handler.__code__.co_argcount - 1

        if func_arg_count != actual_arg_count:
            raise TypeError(
                f"Handler expected to get {actual_arg_count} arguments, but got {func_arg_count}"
            )

        if attribute_name == "on_error":
            self._errors_handled = True

        setattr(self, function.__name__, function)

    async def join_game_from_link(self, link: str, password="") -> Game:
        """
        Joins a game from join link.

        :param password: password that is required to join the game.
        :param link: link to join game.
        """

        return Game(
            self,
            None,
            "",
            False,
            Classic,
            False,
            True,
            False,
            game_join_params=[link, password]
        )

    async def create_game(
        self,
        name="",
        max_players=6,
        unlisted=False,
        password="",
        min_level=0,
        max_level=999,
        server: Server = Warsaw
    ) -> Game:
        """
        Host a bonk.io game.

        :param name: Name of the room. It can only be a string. The default is "Test room".
        :param max_players: The amount of players that can join the game. The amount should be in range [1, 8] (def=6).
        :param unlisted: Indicates whether the game is hidden or not. True or False. Default is False.
        :param password: The password that is required from other players to join the game. Default is "" (no password).
        :param min_level: The minimal level that is required from other players to join the game. Default is 0.
        :param max_level: The maximal level that is required from other players to join the game. Default is 999.
        :param server: The server to join the game. Default is Servers.Warsaw().

        Example usage::

            bot = bonk_account_login("name", "pass")

            async def main():
                game = await bot.create_game()
                await bot.run()

            asyncio.run(main())
        """

        if max_players < 1 or max_players > 8:
            raise TypeError("Max players must be between 1 and 8")
        elif min_level > self.level:
            raise TypeError("Minimal cannot be greater than the account level")
        elif max_level < self.level:
            raise TypeError("Maximum level cannot be lower than the account level")

        return Game(
            self,
            server,
            name,
            True,
            Classic,
            True,
            False,
            False,
            game_create_params=[name, max_players, unlisted, password, min_level, max_level, server]
        )

    async def fetch_online(self) -> BonkOnline:
        """Returns current bonk online players."""

        async with self.aiohttp_session.get("https://bonk2.io/scripts/combinedplayercount.txt") as resp:
            data = json.loads(await resp.text())["bonk"]

        return BonkOnline(
            data["quick_classic"],
            data["quick_arrows"],
            data["quick_grapple"],
            data["custom"],
            data["quick_simple"],
            data["total"]
        )

    async def fetch_b2_maps(self, request: str, by_name=True, by_author=True) -> List[Bonk2Map]:
        """
        Returns list of bonk2 maps.

        :param request: Input string along which the search is performed.
        :param by_name: True if you want to search map by its name. Default is True.
        :param by_author: True if you want to search map by its author. Default is True.
        """

        async with self.aiohttp_session.post(
            url=links["map_get_b2"],
            data={
                "searchauthor": str(by_author).lower(),
                "searchmapname": str(by_name).lower(),
                "searchsort": "best",
                "searchstring": request,
                "startingfrom": 0
            }
        ) as resp:
            data = await resp.json()

        if data.get("e") == "invalid_options":
            raise TypeError("Invalid options for map searching")

        return [
            Bonk2Map(
                self,
                bonk_map["id"],
                bonk_map["leveldata"],
                bonk_map["name"],
                bonk_map["authorname"],
                bonk_map["publisheddate"],
                bonk_map["vu"],
                bonk_map["vd"]
            )
            for bonk_map in data["maps"]
        ]

    # why tf do you store bonk 1 maps data like that chaz mapid0=1597734&mapname0=hammer+vs+SUS+ptb+&creationdate...
    async def fetch_b1_maps(self, request: str, by_name=True, by_author=True) -> List[Bonk1Map]:
        """
        Returns list of bonk1 maps.

        :param request: Input string along which the search is performed.
        :param by_name: True if you want to search map by its name. Default is True.
        :param by_author: True if you want to search map by its author. Default is True.
        """

        async with self.aiohttp_session.post(
            url=links["map_get_b1"],
            data={
                "searchsort": "ctr",
                "searchauthor": str(by_author).lower(),
                "searchmapname": str(by_name).lower(),
                "startingfrom": 0,
                "searchstring": request
            }
        ) as resp:
            data = unquote_plus((await resp.json())["maps"])

        pattern = re.compile(r'mapid\d*=(\d*)&mapname\d*=([^-]*)&creationdate\d*=([^&]*)&modifieddate\d*=([^&]*)&thumbsup\d*=(\d*)&thumbsdown\d*=(\d*)&score\d*=\d*&authorname\d*=([^&]*)&leveldata\d*=([^&]*)')
        parsed_data = pattern.findall(data)

        return [
            Bonk1Map(
                self,
                int(bonk_map[0]),
                bonk_map[7],
                bonk_map[1],
                bonk_map[6],
                bonk_map[2],
                bonk_map[3],
                int(bonk_map[4]),
                int(bonk_map[5])
            ) for bonk_map in parsed_data
        ]

    async def fetch_rooms(self) -> List[Room]:
        """Returns list of rooms in the bonk.io room list."""

        async with self.aiohttp_session.post(
            url=links["rooms"],
            data={
                "version": PROTOCOL_VERSION,
                "gl": "n",
                "token": ""
            }
        ) as resp:
            data = await resp.json()

        return [
            Room(
                self,
                room["id"],
                room["roomname"],
                room["players"],
                room["maxplayers"],
                room["password"] == 1,
                room["mode_mo"],
                room["minlevel"],
                room["maxlevel"]
            ) for room in data["rooms"]
        ]
