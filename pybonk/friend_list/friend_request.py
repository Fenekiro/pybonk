from pybonk.settings import links
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot


class FriendRequest:
    """
    Class for holding account's friend requests.

    :param bot: bot class that uses account.
    :param user_id: account database ID.
    :param username: account username.
    :param date: the date of sending friend request.
    """

    def __init__(
        self,
        bot: "AccountBonkBot",
        user_id: int,
        username: str,
        date: str
    ) -> None:
        self.bot: "AccountBonkBot" = bot
        self.user_id: int = user_id
        self.username: str = username
        self.date: str = date

    async def accept(self) -> None:
        """Accept friend request."""

        await self.bot.aiohttp_session.post(
            url=links["friends"],
            data={
                "token": self.bot.token,
                "task": "accept",
                "theirid": self.user_id
            }
        )

    async def delete(self) -> None:
        """Decline friend request."""

        await self.bot.aiohttp_session.post(
            url=links["friends"],
            data={
                "token": self.bot.token,
                "task": "deleterequest",
                "theirid": self.user_id
            }
        )
