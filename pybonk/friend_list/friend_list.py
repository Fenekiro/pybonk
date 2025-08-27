from functools import cached_property
from typing import List, TYPE_CHECKING

from pybonk.friend_list.friend import Friend
from pybonk.friend_list.friend_request import FriendRequest

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot


class FriendList:
    """
    Class for routing Friend and FriendRequest classes. Account's full friend list.

    :param bot: bot class that uses the account.
    :param raw_data: json data about account's friends and friend requests.
    """

    def __init__(self, bot: "AccountBonkBot", raw_data: dict) -> None:
        self.bot: "AccountBonkBot" = bot
        self._raw_data: dict = raw_data

    @cached_property
    def friends(self) -> List[Friend]:
        """Get friends from account friend list."""

        return [
            Friend(
                self.bot,
                friend["id"],
                friend["name"],
                friend["roomid"]
            ) for friend in self._raw_data["friends"]
        ]

    @cached_property
    def friend_requests(self) -> List[FriendRequest]:
        """Get friend requests from account friend list."""

        return [
            FriendRequest(
                self.bot,
                request["id"],
                request["name"],
                request["date"]
            ) for request in self._raw_data["requests"]
        ]
