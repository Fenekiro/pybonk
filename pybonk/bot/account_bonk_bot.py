from functools import cached_property
from typing import Union, List

from pybonk.avatar.avatar import Avatar
from pybonk.bot.bonk_bot import BonkBot
from pybonk.friend_list.friend_list import FriendList
from pybonk.friend_list.legacy_friend import LegacyFriend
from pybonk.maps.bonk2_map import Bonk2Map
from pybonk.maps.own_map import OwnMap
from pybonk.settings import links


class AccountBonkBot(BonkBot):
    """
    Bot class for bonk.io account.

    :param token: session token that is received when logging into bonk.io account and required for some bonk api calls.
    :param user_id: account database ID.
    :param username: account name.
    :param xp: the amount of xp on account.
    :param legacy_friends: bonk 1 (flash version) friends of the account.
    """

    def __init__(
        self,
        token: str,
        user_id: int,
        username: str,
        xp: int,
        avatars: Union[List[str], None],
        main_avatar: Union[Avatar, None],
        legacy_friends: str
    ) -> None:
        super().__init__(username, xp, avatars, main_avatar)
        self.token: str = token
        self.id: int = user_id
        self._raw_legacy_friends: str = legacy_friends

    @cached_property
    def legacy_friends(self) -> List[LegacyFriend]:
        """Returns list of bonk 1 account friends."""

        return [LegacyFriend(self, friend) for friend in self._raw_legacy_friends.split("#")]

    async def fetch_own_maps(self) -> List[OwnMap]:
        """Returns list of maps created on the account."""

        async with self.aiohttp_session.post(
            url=links["map_get_own"],
            data={
                "token": self.token,
                "startingfrom": "0"
            }
        ) as resp:
            data = await resp.json()

        return [
            OwnMap(
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

    async def fetch_favorite_maps(self) -> List[Bonk2Map]:
        async with self.aiohttp_session.post(
            url=links["map_get_fav"],
            data={
                "token": self.token,
                "startingfrom": "0"
            }
        ) as response:
            data = await response.json()

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

    async def fetch_friend_list(self) -> FriendList:
        """Returns account friend list that contains friends and friend requests."""

        async with self.aiohttp_session.post(
            url=links["friends"],
            data={
                "token": self.token,
                "task": "getfriends"
            }
        ) as resp:
            data = await resp.json()

        return FriendList(self, data)
