from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot

from pybonk.settings import endpoints


class LegacyFriend:
    """
    Class for holding bonk1 legacy friends.

    :param bot: bot class that uses account.
    :param username: legacy friend's username.
    """

    def __init__(self, bot: "AccountBonkBot", username: str) -> None:
        self.bot: "AccountBonkBot" = bot
        self.username: str = username

    async def send_friend_request(self) -> None:
        """Sends friend request to user from legacy friend info."""

        await self.bot.aiohttp_session.post(
            url=endpoints["friends"],
            data={
                "token": self.bot.token,
                "task": "send",
                "theirname": self.username
            }
        )
