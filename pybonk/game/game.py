import asyncio
import random
from random import shuffle
from string import ascii_lowercase
import socketio
import re
from typing import List, Union, TYPE_CHECKING

from pybonk.avatar.avatar import Avatar
from pybonk.maps.bonk1_map import Bonk1Map
from pybonk.maps.bonk2_map import Bonk2Map
from pybonk.maps.own_map import OwnMap
from pybonk.game.map_request_as_client import MapRequestAsClient
from pybonk.game.map_request_as_host import MapRequestAsHost
from pybonk.game.match import Match
from pybonk.game.message import Message
from pybonk.game.player import Player
from pybonk.game.player_move import PlayerMove
from pybonk.settings import PROTOCOL_VERSION, endpoints
from pybonk.types.game_modes import *
from pybonk.types.servers import *
from pybonk.game.teams import *
from pybonk.game.exceptions import GameConnectionError
from pybonk.parsers.parsers import (
    team_from_number,
    mode_from_short_name,
    move_direction_from_number,
    decode_bonk_map_metadata
)

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot import BonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.friend_request import FriendRequest


class Game:
    """
    Class for handling real-time game info and events.

    :param bot: bot class that uses the account.
    :param room_name: name of the room.
    :param is_host: indicates whether bot is host or not.
    :param mode: mode that is currently played.
    :param is_created_by_bot: indicates whether room is created by bot or not. Needed to define which method should be
            called in .__connect() method.
    :param is_joined_from_link: indicates whether room is joined from room link or not. Needed to define which method
            should be called in .__connect() method.
    :param is_joined_from_link: indicates whether room is joined from friend list or not. Needed to define which method
            should be called in .__connect() method.
    :param game_create_params: params that are needed for game creation.
    :param game_join_params: params that are needed to join the game.
    """

    def __init__(
        self,
        bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]",
        server: Union[Server, None],
        room_name: str,
        is_host: bool,
        mode: GameMode,
        is_created_by_bot: bool,
        is_joined_from_link: bool,
        is_joined_from_friend_list: bool,
        game_create_params: Union[list, None] = None,
        game_join_params: Union[list, None] = None
    ) -> None:
        self.bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]" = bot
        self.server: Union[Server, None] = server
        self.room_name: str = room_name
        self.room_password = ""
        self.players: List[Player] = []
        self.messages: List[Message] = []
        self.match: Union[Match, None] = None
        self.bot_move_count = 0
        self.bot_ping: Union[int, None] = None
        self.is_host = is_host
        self.host: Union[Player, None] = None
        self.is_bot_ready = False
        self.is_tabbed = False
        self.is_banned = False
        self.in_lobby = True
        self.mode: GameMode = mode
        self.teams = False
        self.team_lock = False
        self.rounds = 3
        self.bonk_map: Union[OwnMap, Bonk2Map, Bonk1Map, None] = Bonk2Map(
            self.bot,
            1068447,
            "ILAcJAhBFBjBzCTlMiAJgZQEoAYCS8AsgBoCaAHhgJ4BGAzAHIA22AtgJwB2EkXiIAKJkQ0YAAlYnAF4AtAKqyAbrIDuwAMKx6QwQCYAZsDER8wACLrByMeZBnxiGOLQo0GlJ5SqN1r-6R5SFUTG2ABAJRrAAtIyIAxYEYvazNPaABxYzjkcwB6AFZ8AwAXeWwNXBEoaHwASzTUYFgAeRybDMJ2z3kNevTxcwj2okgG7pRE2AApT0FIPNCkaBcJiecNJahgA1xPUZC17vlzczr0ohbGo5uQROgAFjmFrduIfMEM6g49LgAFIa8aCCNpuHZ7N6ePIARkgAGclOgAA7SVRPCBBQEoFYcSGeP4cB4Aewo0nEACcAIYANSQ93RyHmizxuTyAGsvj9-ljgDAQZ40LsWcg8gA2TDiADs1ME0BKWQxkB5y3EuOFED+AGpiaSKTS6cYGUgma9IfkKMxQGxxNRqQAZPy84Gg5CCiHq4B5DjwxEotGBJXDCA4j0gP55SkAU2YAHVqaoAFZKdR3Q3PZke-JFUrlSrVJ38sFC0N5TWi6nRaAUAx25KK5XB1Wh4DhnVkqm0iD09Omt5+CuU6AkB4tPRAwuu8Gh1yyAzTaKgZiYKj1oOiJuhsR22fiUDSfDoRrdqIvUPWT7fX4AiJ8l1NYvqtCQanSOq0eCUvKSiKYtfGDcemIUaxvGSYpsAx6MqeHrWOIDyCNEsB6MwDwANLjnevBTs2ORssA7o4fI+FeGIf52GQrgTOGbLRJq0LYIIdrYAIkBGvgsAZBQ0ToBokC4AwCySnkwkicJKYGJqomiRwX5SSJqjmDJD7Gpo9DwmQJDQPAZCcLg6AZBwJQSXJwngBIeRKCZeTGSZ4DWfQarIB4wBIqotAZPEBDiNMSKyHo0S4DZclmeIFlWUFUl2QYDmeHYwAUG+9B4LQei4NpbAUEoZB6CUskmeJkkmTJQn5cA1BEAmaRLGIGgkNgSiwLxb4GQmlL0GFJlQHlcntZZnXmPg5VOKgW5SMwXCUuIWk6Zl2W5SVckFVZxVWeo5WVaISAiBoGXMLIXDTRlWU5d1UlLUVp2iWtFVVUgsC8jG0xwiljDkhFokhR1cnvSJ4A3dQQhIGk41SPEbKUnGACu6DmIJq28j9wkrfl+R0EQm0QHhGgxug1BkDG2C+TGBS4KQX1SXcl0ib1VktgAKjGcLbBAGiINMsj0POsgZMwzC0Ad1CI3kZkkOTolC+ABQlJqTPAI6wB2iAINsGDEPUtD5iMFTYkI4VcnI4t5gkAU5LBkg6jY7j+OE7IxO4IwoV9XJlMLVJNOdUQXDkpDw3vGI7Oc9E3O8-z0DQhLICi07UkR0okaSuIgMQKCyuq1DMOyNreTqJAQsG2dlSgOgRHZBA4CW3jBNEyTjDu87EFZ3XFN2jaDw5ypwAB1zPN8wdDwR8AUfhXrkXAEZdopvL2BK1woPg+n5jMFnOd58vGiaqABiy6EkBmDtVtV7bJNEHTcOlbnI-Scv5gZJKShmTv6P75XNt2yf8TL7ry3X3aop-A-QJNBsAPq-Y+dNpifwvt-V2V1zB0wePfDGvIiLP2ttXUmdMtYwPkl-C62CxLmG0P-JBkBtrAJfugk+2BIGr3wdnRe4cAFQHuqgw+b86bUhoZfES+dYGQ2kMmEhiBWGgIwZnOhK9uFI2vtCagfwS47yxuQtBR8MHoC4dA1a5hJSqBliQokQCQGULptEDReCtGalUIwdEO8mYiOMUvCRuD9bXwsqAFMO8c5sEYEiKQjA4SyCHp1XkjcxYiQsINIgEQTBTU7hxLiPF4S21kIFKRwsQCO2HlZKKMUoh3GkErNyHlcCyE4HQEg6McKeiNE5dJsU8h0yINIJpLSClVPacKGqeSMbDT4qgWA6hUI6XiK1PQuMGDq1kO0-If4QD5CRGwUpSy2CUgJK09ZHTNlvFcPLScFhNowGAAqeIbSCh1Hxq9Dm0xETEzZPgAy0J0DTLyLMiweQFnLNKasjg6zmnNK2QCtYOyAhiEThBU2wB0AJhSpldAJBphtK8HYQcRwtQBC1IszFSyiR5FwH89Z4Ejh5GoJ8zFCZs5eHDB0vIzAKTRGwKKB48hIyqD8L2YAnZhFiHQsAJmcU6xPEsNkSA55EB0ydBBMQMZeT3QVFMyABiMjqHugrYAMZTTowKF4VQwRbj3DZXMzme1eKzzYNSPmdRIDSA4vEegtA2SQCmZ4BSeEwRgpuAog12AZrQlkJgSABRbYUGevQBA2hIDcT0DUoAA",
            "RGB 1v1",
            "roseFog",
            "2023-12-06 18:38:37",
            472,
            83
        )
        self.requested_maps: List[MapRequestAsHost] = []
        self.join_link = ""
        self.socket_client = socketio.AsyncClient(ssl_verify=False)
        self.__is_created_by_bot: bool = is_created_by_bot
        self.__is_joined_from_link: bool = is_joined_from_link
        self.__is_joined_from_friend_list: bool = is_joined_from_friend_list
        self.__game_create_params: Union[list, None] = game_create_params
        self.__game_join_params: Union[list, None] = game_join_params
        self.__is_connected = False

        asyncio.run(self.__connect())

    async def __connect(self) -> None:
        """Method that establishes connection with game."""

        self.bot.games.append(self)

        if self.__is_created_by_bot:
            await self.__create(*self.__game_create_params)
        elif self.__is_joined_from_link:
            await self.__join_from_room_link(*self.__game_join_params)
        elif self.__is_joined_from_friend_list:
            await self.__join_from_friend_list(*self.__game_join_params)
        else:
            await self.__join(*self.__game_join_params)

    async def set_bot_team(self, team: Team) -> None:
        """
        Changes current bot team.

        :param team: team that is bot moving in.
        """

        await self.socket_client.emit(6, {"targetTeam": team.value})

    async def set_team_lock(self, flag: bool) -> None:
        """
        Lock free team switching.

        :param flag: on -> True (locked teams) | off -> False (free team switching).
        """

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set teamlock, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(7, {"teamLock": flag})

            self.team_lock = True

    async def send_message(self, message: str) -> None:
        """
        Send message from bot in the game.

        :param message: message content.
        """

        await self.socket_client.emit(10, {"message": message})

    async def set_bot_ready(self, flag: bool) -> None:
        """
        Turn on/off bot ready mark in the game.

        :param flag: on -> True (bot is ready) | off -> False (bot is not ready).
        """

        await self.socket_client.emit(16, {"ready": flag})

        self.is_bot_ready = flag

    async def reset_all_ready(self) -> None:
        """Resets ready mark for all players in the game."""

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot reset ready, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(17)
            self.is_bot_ready = False

            for player in self.players:
                player.is_ready = False

    async def set_mode(self, mode: GameMode) -> None:
        """
        Change game mode.

        :param mode: one of the Modes class types.
        """

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set mode, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(20, {"ga": mode.engine, "mo": mode.short_name})

            self.mode = mode

    async def set_rounds(self, rounds: int) -> None:
        """
        Change rounds to win.

        :param rounds: rounds that player has to reach to win the game.
        """

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set rounds, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(21, {"w": rounds})

            self.rounds = rounds

    async def set_map(self, bonk_map: Union[OwnMap, Bonk2Map, Bonk1Map]) -> None:
        """
        Change game map.

        :param bonk_map: the map that is wanted to be played in the game.
        """

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set map, bot is not a host", self)
            )
        else:
            if not (
                isinstance(bonk_map, OwnMap) or
                isinstance(bonk_map, Bonk2Map) or
                isinstance(bonk_map, Bonk1Map)
            ):
                raise TypeError("Input param is not a map")

            await self.socket_client.emit(23, {"m": bonk_map.encoded_data})

            self.bonk_map = bonk_map

    async def request_map(self, bonk_map: Union[OwnMap, Bonk2Map, Bonk1Map]) -> None:
        """
        Suggest map in the chat.

        :param bonk_map: the map that is wanted to be suggested.
        """

        if not (
            isinstance(bonk_map, OwnMap) or
            isinstance(bonk_map, Bonk2Map) or
            isinstance(bonk_map, Bonk1Map)
        ):
            raise TypeError("Input param is not a map")

        await self.socket_client.emit(
            27,
            {
                "m": bonk_map.encoded_data,
                "mapauthor": bonk_map.author_name,
                "mapname": bonk_map.name
            }
        )

    async def set_teams(self, flag: bool) -> None:
        """
        Turn on/off extended (red, blue, green and yellow) teams.

        :param flag: on -> True (extended teams) | off -> False (only FFA).
        """

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set teams, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(32, {"t": flag})

            self.teams = flag

    async def record(self) -> None:
        """Record the last 15 seconds of round."""

        await self.socket_client.emit(33)

    async def gain_xp(self) -> None:
        """Get 100 xp. Limit: 18000, 2000 xp are available every 20 minutes."""

        await self.socket_client.emit(38)

    async def set_bot_tab_status(self, flag: bool) -> None:
        """
        Toggle bot tab status.

        :param flag: indicates whether bot should be tabbed or not.
        """

        await self.socket_client.emit(44, {"out": flag})

        self.is_tabbed = flag

    async def close(self) -> None:
        """Removes the game from room list."""

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot close game, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(50)
            await self.leave()

    async def change_room_name(self, new_room_name: str) -> None:
        """
        Change room name.

        :param new_room_name: new room name.
        """

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set room name, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(52, {"newName": new_room_name})

            self.room_name = new_room_name

    async def change_room_password(self, new_password: str = "") -> None:
        """
        Change room password.

        :param new_password: new room password. If not provided, room pass is cleared.
        """

        if not self.is_host:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Cannot set room password, bot is not a host", self)
            )
        else:
            await self.socket_client.emit(53, {"newPass": new_password})

            self.room_password = new_password

    async def leave(self) -> None:
        """Disconnect from the game."""

        await self.socket_client.disconnect()

        self.__is_connected = False
        self.bot.games.remove(self)

        self.bot.event_emitter.emit("game_disconnect", self)

    @staticmethod
    def __get_peer_id() -> str:
        """Generates new peer_id that is needed for game connection."""

        alph = list(ascii_lowercase + "0123456789")
        shuffle(alph)
        return "".join(alph[:10]) + "000000"

    def __get_player_from_short_id(self, short_id: int) -> "Player":
        """Finds player in the room by their short id."""

        for player in self.players:
            if player.short_id == short_id:
                return player

        raise Exception(f"Player {short_id} not found")

    async def __create(
        self,
        name="",
        max_players=6,
        unlisted=False,
        password="",
        min_level=0,
        max_level=999,
        server: Server = Warsaw
    ) -> None:
        """
        Sends packets to create a room.

        :param name: the name of the room.
        :param max_players: the maximum number of players that can join the room.
        :param unlisted: indicates whether the room is listed in room list.
        :param password: room password.
        :param min_level: the minimum level required from other players to join; can't be greater than bot's level.
        :param max_level: the maximum level required from other players to join; can't be lower than bot's level.
        """

        socket_address = f"https://{server.api_name}.bonk.io/socket.io"
        base_room_name = f"{self.bot.username}'s game"

        if name == "":
            self.room_name = base_room_name
        else:
            self.room_name = name

        @self.socket_client.event
        async def connect() -> None:
            self._is_host = True
            new_peer_id = self.__get_peer_id()

            if not self.bot.is_guest:
                await self.socket_client.emit(
                    12,
                    {
                        "peerID": new_peer_id,
                        "roomName": base_room_name if name == "" else name,
                        "maxPlayers": max_players,
                        "password": password,
                        "dbid": self.bot.id,
                        "guest": False,
                        "minLevel": min_level,
                        "maxLevel": max_level,
                        "latitude": server.latitude,
                        "longitude": server.latitude,
                        "country": server.country,
                        "version": PROTOCOL_VERSION,
                        "hidden": int(unlisted),
                        "quick": False,
                        "mode": "custom",
                        "token": self.bot.token,
                        "avatar": self.bot.main_avatar.to_json()
                    }
                )
            else:
                await self.socket_client.emit(
                    12,
                    {
                        "peerID": new_peer_id,
                        "roomName": base_room_name if name == "" else name,
                        "maxPlayers": max_players,
                        "password": password,
                        "dbid": random.randint(10_000_000, 14_000_000),
                        "guest": True,
                        "minLevel": min_level,
                        "maxLevel": max_level,
                        "latitude": server.latitude,
                        "longitude": server.longitude,
                        "country": server.country,
                        "version": PROTOCOL_VERSION,
                        "hidden": int(unlisted),
                        "quick": False,
                        "mode": "custom",
                        "guestName": self.bot.username,
                        "avatar": self.bot.main_avatar.to_json()
                    }
                )

            bot_player = Player(
                self.bot,
                self,
                True,
                True,
                new_peer_id,
                self.bot.username,
                self.bot.is_guest,
                self.bot.level,
                False,
                False,
                Team.FFA,
                0,
                self.bot.main_avatar
            )

            self.players.append(bot_player)
            self.host = bot_player

        await self.__load_socket_events()

        await self.socket_client.connect(socket_address)
        await self.__keep_alive()

    async def __join_from_friend_list(self, room_id: int, password="") -> None:
        """
        Sends join packets to join a room from friend list.

        :param room_id: the room id.
        """

        async with self.bot.aiohttp_session.post(
            url=endpoints["get_room_address"],
            data={
                "id": room_id
            }
        ) as resp:
            room_data = await resp.json()

        error = room_data.get("e")

        if error:
            self.bot.event_emitter.emit("error", GameConnectionError(error, self))
            return

        @self.socket_client.event
        async def connect() -> None:
            if not self.bot.is_guest:
                await self.socket_client.emit(
                    13,
                    {
                        "joinID": room_data["address"],
                        "roomPassword": password,
                        "guest": False,
                        "dbid": 2,
                        "version": PROTOCOL_VERSION,
                        "peerID": self.__get_peer_id(),
                        "bypass": "",
                        "token": self.bot.token,
                        "avatar": self.bot.main_avatar.to_json()
                    }
                )
            else:
                await self.socket_client.emit(
                    13,
                    {
                        "joinID": room_data["address"],
                        "roomPassword": password,
                        "guest": True,
                        "dbid": 2,
                        "version": PROTOCOL_VERSION,
                        "peerID": self.__get_peer_id(),
                        "bypass": "",
                        "guestName": self.bot.username,
                        "avatar": self.bot.main_avatar.to_json()
                    }
                )

        await self.__load_socket_events()

        await self.socket_client.connect(f"https://{room_data['server']}.bonk.io/socket.io")
        await self.__keep_alive()

    async def __join_from_room_link(self, link: str, password="") -> None:
        """
        Sends join packets to join a room from link.

        :param link: room link; looks like `https://bonk.io/607883jdyrv`.
        """

        pattern = re.compile(
            r'{"address":"(.*?)","roomname":"(.*?)","server":"(.*?)","passbypass":"(.*?)","r":"success"}'
        )

        async with self.bot.aiohttp_session.get(link) as resp:
            data = await resp.text()

        try:
            room_data = pattern.findall(data)[0]
            self.room_name = room_data[1]

            @self.socket_client.event
            async def connect() -> None:
                if not self.bot.is_guest:
                    await self.socket_client.emit(
                        13,
                        {
                            "joinID": room_data[0],
                            "roomPassword": password,
                            "guest": False,
                            "dbid": 2,
                            "version": PROTOCOL_VERSION,
                            "peerID": self.__get_peer_id(),
                            "bypass": room_data[3],
                            "token": self.bot.token,
                            "avatar": self.bot.main_avatar.to_json()
                        }
                    )
                else:
                    await self.socket_client.emit(
                        13,
                        {
                            "joinID": room_data[0],
                            "roomPassword": password,
                            "guest": True,
                            "dbid": 2,
                            "version": PROTOCOL_VERSION,
                            "peerID": self.__get_peer_id(),
                            "bypass": room_data[3],
                            "guestName": self.bot.username,
                            "avatar": self.bot.main_avatar.to_json()
                        }
                    )

            await self.__load_socket_events()

            await self.socket_client.connect(f"https://{room_data[2]}.bonk.io/socket.io")
            await self.__keep_alive()
        except IndexError:
            self.bot.event_emitter.emit(
                "error",
                GameConnectionError("Room is not found", self)
            )
            await self.leave()

    async def __join(self, room_id: int, password="") -> None:
        """
        Sends packets to join a game within Room class.

        :param password: password to enter the room (if required).
        """

        async with self.bot.aiohttp_session.post(
            url=endpoints["get_room_address"],
            data={
                "id": room_id
            }
        ) as resp:
            room_data = await resp.json()

        error = room_data.get("e")

        if error:
            self.bot.event_emitter.emit("error", GameConnectionError(error, self))
            return

        @self.socket_client.event
        async def connect() -> None:
            if not self.bot.is_guest:
                await self.socket_client.emit(
                    13,
                    {
                        "joinID": room_data["address"],
                        "roomPassword": password,
                        "guest": False,
                        "dbid": 2,
                        "version": PROTOCOL_VERSION,
                        "peerID": self.__get_peer_id(),
                        "bypass": "",
                        "token": self.bot.token,
                        "avatar": self.bot.main_avatar.to_json()
                    }
                )
            else:
                await self.socket_client.emit(
                    13,
                    {
                        "joinID": room_data["address"],
                        "roomPassword": password,
                        "guest": True,
                        "dbid": 2,
                        "version": PROTOCOL_VERSION,
                        "peerID": self.__get_peer_id(),
                        "bypass": "",
                        "guestName": self.bot.username,
                        "avatar": self.bot.main_avatar.to_json()
                    }
                )

        await self.__load_socket_events()

        await self.socket_client.connect(f"https://{room_data['server']}.bonk.io/socket.io")
        await self.__keep_alive()

    async def __keep_alive(self) -> None:
        """Sends timesync packet every 5 seconds to prevent bonk server from kicking bot."""

        ping_id = 1

        while self.__is_connected:
            await self.socket_client.emit(
                18,
                {
                    "jsonrpc": "2.0",
                    "id": ping_id,
                    "method": "timesync",
                }
            )
            await asyncio.sleep(5)

            ping_id += 1

    async def __load_socket_events(self) -> None:
        """Game event listener."""

        @self.socket_client.on(1)
        async def on_ping(ping_data: dict, ping_id: int) -> None:
            for ping in ping_data.keys():
                player = self.__get_player_from_short_id(int(ping))
                player_ping = ping_data[ping]
                player.ping = player_ping

                if player.is_bot:
                    self.bot_ping = player_ping

                self.bot.event_emitter.emit("player_ping", player, player_ping)

            if not self.is_tabbed:
                await self.socket_client.emit(1, {"id": ping_id})

        @self.socket_client.on(3)
        async def players_on_bot_join(
            bot_short_id: int,
            host_short_id: int,
            players: list,
            timestamp: int,
            team_lock: bool,
            room_id: int,
            bypass: str,
            w8
        ) -> None:
            for player in players:
                if player:
                    player_short_id = players.index(player)
                    self.players.append(
                        Player(
                            self.bot,
                            self,
                            player_short_id == len(players) - 1,
                            player_short_id == host_short_id,
                            player["peerID"],
                            player["userName"],
                            player["guest"],
                            player["level"],
                            player["ready"],
                            player["tabbed"],
                            team_from_number(player["team"]),
                            player_short_id,
                            Avatar().load_from_json(player["avatar"])
                        )
                    )

            self.join_link = f"https://bonk.io/{room_id:06}{bypass}"
            self._team_lock = team_lock

            for x in self.players:
                if x.team.value > 1:
                    self._teams = True

            self.__is_connected = True
            self.bot.event_emitter.emit("game_connect", self)

        @self.socket_client.on(4)
        async def on_player_join(
            short_id: int,
            peer_id: str,
            username: str,
            is_guest: bool,
            level: int,
            joined_with_bypass: int,
            avatar: dict
        ) -> None:
            joined_player = Player(
                self.bot,
                self,
                False,
                False,
                peer_id,
                username,
                is_guest,
                level,
                False,
                False,
                Team.SPECTATOR if self.team_lock or self.teams else Team.FFA,
                short_id,
                Avatar().load_from_json(avatar)
            )

            self.players.append(joined_player)

            if self.is_host:
                bad_map_data = {
                    "v": 13,
                    "s": {
                        "re": False,
                        "nc": False,
                        "pq": 1,
                        "gd": 25,
                        "fl": False
                    },
                    "physics": {
                        "shapes": [],
                        "fixtures": [],
                        "bodies": [],
                        "bro": [],
                        "joints": [],
                        "ppm": 12
                    },
                    "spawns": [],
                    "capZones": [],
                    "m": {
                        "a": self.bonk_map.author_name,
                        "n": self.bonk_map.name,
                        "dbv": 2,
                        "dbid": self.bonk_map.map_id,
                        "authid": -1,
                        "date": self.bonk_map.creation_date,
                        "rxid": 0,
                        "rxn": "",
                        "rxa": "",
                        "rxdb": 1,
                        "cr": [
                            "💀"
                        ],
                        "pub": self.bonk_map.is_published if isinstance(self.bonk_map, OwnMap) else True,
                        "mo": "",
                        "vu": self.bonk_map.votes_up,
                        "vd": self.bonk_map.votes_down
                    }
                }

                await self.socket_client.emit(
                    11,
                    {
                        "sid": short_id,
                        "gs": {
                            "map": self.bonk_map.decoded_data if isinstance(self.bonk_map, OwnMap) or isinstance(self.bonk_map, Bonk2Map) else bad_map_data,
                            "gt": 2,
                            "wl": self.rounds,
                            "q": False,
                            "tl": self.team_lock,
                            "tea": self.teams,
                            "ga": self.mode.engine,
                            "mo": self.mode.short_name,
                            "bal": []
                        }
                    }
                )
            self.bot.event_emitter.emit("player_join", joined_player)

        @self.socket_client.on(5)
        async def on_player_leave(player_short_id: int, w) -> None:
            left_player = self.__get_player_from_short_id(player_short_id)
            self.players.remove(left_player)

            self.bot.event_emitter.emit("player_leave", left_player)

        @self.socket_client.on(6)
        async def on_host_leave(old_host_id: int, new_host_id: int, w) -> None:
            if new_host_id != -1:
                old_host = self.__get_player_from_short_id(old_host_id)
                new_host = self.__get_player_from_short_id(new_host_id)

                if old_host.is_bot and not new_host.is_bot:
                    self._is_host = False
                elif not old_host.is_bot and new_host.is_bot:
                    self._is_host = True

                new_host.is_host = True
                self.bot.event_emitter.emit("host_leave", old_host, new_host)
            else:
                self.bot.event_emitter.emit("game_close", self)

        @self.socket_client.on(7)
        async def on_player_move(player_short_id: int, move_data: dict) -> None:
            try:
                player = self.__get_player_from_short_id(player_short_id)
                move_direction = move_direction_from_number(move_data["i"])

                player_move = PlayerMove(
                    move_direction,
                    self,
                    self.match,
                    player,
                    move_data["f"],
                    move_data["c"]
                )
                self.bot.event_emitter.emit("player_move", player_move)
            except KeyError:
                pass

        @self.socket_client.on(8)
        async def on_player_ready(player_short_id: int, flag: bool) -> None:
            player = self.__get_player_from_short_id(player_short_id)
            player.is_ready = flag

            if flag:
                self.bot.event_emitter.emit("player_ready", player)

        @self.socket_client.on(13)
        async def on_match_abort() -> None:
            self._in_lobby = True
            self.bot.event_emitter.emit("match_abort", self)

        @self.socket_client.on(15)
        async def on_match_start(timestamp: int, map_data: str, additional_data: dict) -> None:
            self._in_lobby = False
            new_match = Match(self.bot, self, self.bonk_map)

            self._match = new_match
            self.bot.event_emitter.emit("match_start", new_match)

        @self.socket_client.on(16)
        async def on_error(error) -> None:
            if error != "rate_limit_pong":
                self.bot.event_emitter.emit("error", GameConnectionError(error, self))

            if error in [
                "invalid_params",
                "password_wrong",
                "room_full",
                "players_xp_too_high",
                "players_xp_too_low",
                "guests_not_allowed",
                "already_in_this_room",
                "room_not_found",
                "avatar_data_invalid"
            ]:
                await self.leave()

        @self.socket_client.on(18)
        async def on_player_team_change(player_short_id: int, team_number: int) -> None:
            player = self.__get_player_from_short_id(player_short_id)
            team = team_from_number(team_number)
            player.team = team

            self.bot.event_emitter.emit("player_team_change", player, team)

        @self.socket_client.on(19)
        async def on_team_lock(flag: bool) -> None:
            self._team_lock = flag

            if flag:
                self.bot.event_emitter.emit("team_lock", self)
            else:
                self.bot.event_emitter.emit("team_unlock", self)

        @self.socket_client.on(20)
        async def on_message(player_short_id: int, message: str) -> None:
            author = self.__get_player_from_short_id(player_short_id)
            _message = Message(self, self.bot, author, message)

            self.messages.append(_message)

            self.bot.event_emitter.emit("message", _message)

        @self.socket_client.on(21)
        async def on_lobby_load(data: dict) -> None:
            self._mode = mode_from_short_name(data["mo"])
            self._team_lock = data["tl"]
            self._rounds = data["wl"]

        @self.socket_client.on(24)
        async def on_player_kick(player_short_id: int, kick_only: bool) -> None:
            player = self.__get_player_from_short_id(player_short_id)

            if kick_only:
                if player.is_bot:
                    self.bot.event_emitter.emit("bot_kick", self)
                    await self.leave()
                else:
                    self.bot.event_emitter.emit("player_kick", player)
            else:
                if player.is_bot:
                    self.bot.event_emitter.emit("bot_ban", self)
                    await self.leave()
                    self._is_banned = True
                else:
                    self.bot.event_emitter.emit("player_ban", player)

        @self.socket_client.on(26)
        async def on_mode_change(ga, mode_short_name: str) -> None:
            self._mode = mode_from_short_name(mode_short_name)

            self.bot.event_emitter.emit("mode_change", self, self.mode)

        @self.socket_client.on(27)
        async def on_rounds_change(rounds: int) -> None:
            self._rounds = rounds
            self.bot.event_emitter.emit("rounds_change", self, rounds)

        @self.socket_client.on(29)
        async def on_map_change(map_encoded_data: str) -> None:
            map_decoded_data = decode_bonk_map_metadata(map_encoded_data)

            new_map = Bonk2Map(
                self.bot,
                map_decoded_data["m"]["dbid"],
                map_encoded_data,
                map_decoded_data["m"]["n"],
                map_decoded_data["m"]["a"],
                map_decoded_data["m"]["date"],
                map_decoded_data["m"]["vu"],
                map_decoded_data["m"]["vd"]
            )

            self._bonk_map = new_map
            self.bot.event_emitter.emit("map_change", self, new_map)

        @self.socket_client.on(32)
        async def on_afk_warn() -> None:
            self.bot.event_emitter.emit("afk_warn", self)

        @self.socket_client.on(33)
        async def on_map_request_host(level_data: str, player_short_id: int) -> None:
            player = self.__get_player_from_short_id(player_short_id)
            map_request = MapRequestAsHost(self, self.bot, player, level_data)

            self.requested_maps.append(map_request)
            self.bot.event_emitter.emit("map_request_host", map_request)

        @self.socket_client.on(34)
        async def on_map_request_client(map_name: str, author: str, player_short_id: int) -> None:
            player = self.__get_player_from_short_id(player_short_id)
            map_request = MapRequestAsClient(self, self.bot, map_name, author, player)

            self.bot.event_emitter.emit("map_request_client", map_request)

        @self.socket_client.on(36)
        async def on_player_balance(player_short_id: int, percents: int) -> None:
            player = self.__get_player_from_short_id(player_short_id)
            player.balanced_by = percents

            self.bot.event_emitter.emit("player_balance", player, percents)

        @self.socket_client.on(39)
        async def on_teams_toggle(flag: bool) -> None:
            self._teams = flag

            if flag:
                self.bot.event_emitter.emit("teams_on", self)
            else:
                self.bot.event_emitter.emit("teams_off", self)

        @self.socket_client.on(40)
        async def on_replay(player_short_id: int) -> None:
            player = self.__get_player_from_short_id(player_short_id)

            self.bot.event_emitter.emit("replay", player)

        @self.socket_client.on(41)
        async def on_host_change(data: dict) -> None:
            old_host = self.__get_player_from_short_id(data["oldHost"])
            new_host = self.__get_player_from_short_id(data["newHost"])

            if old_host.is_bot and not new_host.is_bot:
                self._is_host = False
            elif not old_host.is_bot and new_host.is_bot:
                self._is_host = True

            old_host.is_host = False
            new_host.is_host = True
            self.host = new_host

            self.bot.event_emitter.emit("host_change", old_host, new_host)

        @self.socket_client.on(42)
        async def on_friend_request(player_short_id: int) -> None:
            player = self.__get_player_from_short_id(player_short_id)
            friend_request = FriendRequest(self, self.bot, player)

            self.bot.event_emitter.emit("friend_request", friend_request)

        @self.socket_client.on(43)
        async def on_match_countdown(starts_in_seconds: int) -> None:
            self.bot.event_emitter.emit("match_countdown", self, starts_in_seconds)

        @self.socket_client.on(44)
        async def on_match_countdown_abort() -> None:
            self.bot.event_emitter.emit("match_countdown_abort", self)

        @self.socket_client.on(45)
        async def on_player_level_up(data: dict) -> None:
            player = self.__get_player_from_short_id(data["sid"])
            new_level = data["lv"]

            player.level = new_level

            self.bot.event_emitter.emit("player_level_up", player, new_level)

        @self.socket_client.on(46)
        async def on_xp_gain(data: dict) -> None:
            new_xp = data["newXP"]
            new_token = data.get("newToken")

            self.bot.xp = new_xp

            if new_token:
                self.bot.token = new_token

            self.bot.event_emitter.emit("xp_gain", self, new_xp)

        @self.socket_client.on(48)
        async def on_match_info(data: dict) -> None:
            self._match = Match(self.bot, self, self.bonk_map, data["fc"])
            self._in_lobby = False

        @self.socket_client.on(49)
        async def on_join_link_receive(join_link_number: int, bypass: str) -> None:
            self.join_link = f"https://bonk.io/{join_link_number:06}{bypass}"
            self.__is_connected = True

            self.bot.event_emitter.emit("game_connect", self)

        @self.socket_client.on(52)
        async def on_player_tab(player_short_id: int, status: bool) -> None:
            player = self.__get_player_from_short_id(player_short_id)
            player.is_tabbed = status

            if status:
                self.bot.event_emitter.emit("player_tab", player)
            else:
                self.bot.event_emitter.emit("player_tab_reset", player)

        @self.socket_client.on(58)
        async def on_new_room_name(new_room_name: str) -> None:
            self.room_name = new_room_name

            self.bot.event_emitter.emit("new_room_name", self, new_room_name)

        @self.socket_client.on(59)
        async def on_new_room_password(flag: int) -> None:
            if bool(flag):
                self.bot.event_emitter.emit("new_room_password", self)
            else:
                self.bot.event_emitter.emit("room_password_clear", self)
