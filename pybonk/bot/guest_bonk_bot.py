from typing import Union, List

from pybonk.avatar.avatar import Avatar
from pybonk.bot.bonk_bot import BonkBot


class GuestBonkBot(BonkBot):
    """
    Bot class for bonk.io guest account.

    :param username: guest account name.
    :param xp: the amount of xp on account (0 by default).
    """

    def __init__(
        self,
        username: str,
        xp: int,
        avatars: Union[List[str], None],
        main_avatar: Union[Avatar, None]
    ) -> None:
        super().__init__(username, xp, avatars, main_avatar)
