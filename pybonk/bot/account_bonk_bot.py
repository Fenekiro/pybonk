import asyncio
from functools import cached_property
from typing import Union, List
import aiohttp

from pybonk.avatar import Avatar
from pybonk.bot.bonk_bot_base import BonkBotBase
from pybonk.bot.exceptions import BonkLoginError
from pybonk.friend_list.friend_list import FriendList
from pybonk.friend_list.legacy_friend import LegacyFriend
from pybonk.maps.partial_bonk2_map import PartialBonk2Map
from pybonk.maps.partial_own_bonk_map import PartialOwnBonkMap
from pybonk.settings import endpoints


class AccountBonkBot(BonkBotBase):
    """
    Bot class for bonk.io account.
    """

    def __init__(self) -> None:
        super().__init__()

        self._token: Union[None, str] = None
        self._id: Union[None, int] = None
        self._raw_legacy_friends: Union[None, str] = None

    @property
    def token(self) -> str:
        if self._token is None:
            raise Exception("Bot is not logged in")

        return self._token

    @property
    def id(self) -> int:
        if self._id is None:
            raise Exception("Bot is not logged in")

        return self._id

    @cached_property
    def legacy_friends(self) -> List[LegacyFriend]:
        """Returns list of bonk 1 account friends."""

        if self._raw_legacy_friends is None:
            raise Exception("Bot is not logged in")

        return [LegacyFriend(self, friend) for friend in self._raw_legacy_friends.split("#")]

    async def login(self, username: str, password: str) -> None:
        """Prevents room connections from stopping and "starts" the bot."""

        self.aiohttp_session = aiohttp.ClientSession()

        data = await self.aiohttp_session.post(
            url=endpoints["login"],
            data={
                "username": username,
                "password": password,
                "remember": "false"
            }
        )

        json_data = await data.json()

        if json_data.get("e") == "username_fail":
            raise BonkLoginError(f"Invalid username {username}")
        elif json_data.get("e") == "password":
            raise BonkLoginError(f"Invalid password for account {username}")

        avatars = [
            json_data["avatar1"],
            json_data["avatar2"],
            json_data["avatar3"],
            json_data["avatar4"],
            json_data["avatar5"]
        ]

        self._token = json_data["token"]
        self._id = json_data["dbid"]
        self._raw_legacy_friends = json_data["legacyFriends"]
        self.username = username
        self.xp = json_data["xp"]
        self.avatars = avatars
        self.main_avatar = Avatar.from_encoded(avatars[json_data["activeAvatarNumber"] - 1])
        self.games = []

        for attr in dir(self):
            if attr.startswith("on"):
                self._event_emitter.on(attr.replace("on_", ""), getattr(self, attr))

        await self._event_emitter.emit_async("ready")

        while not self.aiohttp_session.closed:
            await asyncio.sleep(0.1)

    async def fetch_own_maps(self) -> List[PartialOwnBonkMap]:
        """Returns list of maps created on the account."""

        async with self.aiohttp_session.post(
            url=endpoints["map_get_own"],
            data={
                "token": self.token,
                "startingfrom": "0"
            }
        ) as resp:
            data = await resp.json()

        return [
            PartialOwnBonkMap(
                self,
                bonk_map["id"],
                bonk_map["leveldata"],
                bonk_map["name"],
                bonk_map["creationdate"],
                bonk_map["published"] == 1,
                bonk_map["vu"],
                bonk_map["vd"]
            )
            for bonk_map in data["maps"]
        ]

    async def fetch_favorite_maps(self) -> List[PartialBonk2Map]:
        async with self.aiohttp_session.post(
            url=endpoints["map_get_fav"],
            data={
                "token": self.token,
                "startingfrom": "0"
            }
        ) as response:
            data = await response.json()

        return [
            PartialBonk2Map(
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

    async def fetch_friend_list(self) -> FriendList:
        """Returns account friend list that contains friends and friend requests."""

        async with self.aiohttp_session.post(
            url=endpoints["friends"],
            data={
                "token": self.token,
                "task": "getfriends"
            }
        ) as resp:
            data = await resp.json()

        return FriendList(self, data)
