import asyncio
from typing import Union

from pybonk.bot.bonk_bot import BonkBot
from pybonk.bot.guest_bonk_bot import GuestBonkBot
from pybonk.bot.account_bonk_bot import AccountBonkBot
from pybonk.game.game import Player, Game
from pybonk.settings import endpoints


class FriendRequest:
    """
    Class that holds info about friend request from player.

    :param game: game where friend request was sent.
    :param player: player that sent friend request.
    """

    def __init__(
        self,
        game: Game,
        bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]",
        player: Player
    ) -> None:
        self.game: Game = game
        self.bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]" = bot
        self.player: Player = player

    async def accept(self) -> None:
        """
        Accept player's friend request. If bot or player that sent request is guest it just sends friend accept
        notification in the chat and doesn't add to friend list.
        """

        bot = self.bot
        tasks = []

        if not bot.is_guest and not self.player.is_guest:
            tasks.append(
                bot.aiohttp_session.post(
                    url=endpoints["friends"],
                    data={
                        "token": bot.token,
                        "task": "send",
                        "theirname": self.player.username
                    }
                )
            )

        tasks.append(
            self.game.socket_client.emit(
                35,
                {
                    "id": self.player.short_id
                }
            )
        )

        await asyncio.gather(*tasks)
