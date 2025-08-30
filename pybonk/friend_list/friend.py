from typing import TYPE_CHECKING, Union

from pybonk.settings import endpoints

if TYPE_CHECKING:
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.game import Game
    from pybonk.types.game_modes import Classic


class Friend:
    """
    Class for holding account's friends.

    :param bot: bot class that uses the account.
    :param user_id: friend's account database ID.
    :param username: friend's account username.
    :param room_id: the room ID where friend is playing.
    """

    def __init__(
        self,
        bot: "AccountBonkBot",
        user_id: int,
        username: str,
        room_id: Union[int, None]
    ) -> None:
        self.bot: "AccountBonkBot" = bot
        self.user_id: int = user_id
        self.username: str = username
        self.room_id: Union[int, None] = room_id

    async def unfriend(self) -> None:
        """Remove friend from account friend list."""

        await self.bot.aiohttp_session.post(
            url=endpoints["friends"],
            data={
                "token": self.bot.token,
                "task": "unfriend",
                "theirid": self.user_id
            }
        )

    async def join_game(self, password="") -> "Game":
        """
        Establish connection with room where friend is playing.

        :param password: password that is required to join the game.

        Example usage::

            bot = bonk_account_login("name", "pass")

            friend_list = bot.get_friend_list()
            friend = [friend for friend in friend_list.get_friends() if friend.username == "test" and friend.room_id][0]

            async def main():
                game = friend.join_game()
                await game.send_message("Hello!")

                await bot.run()

            asyncio.run(main())
        """

        return Game(
            self.bot,
            None,
            "Unknown",
            False,
            Classic,
            False,
            False,
            True,
            game_join_params=[self.room_id, password]
        )
