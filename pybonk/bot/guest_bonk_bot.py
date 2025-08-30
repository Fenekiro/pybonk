import asyncio
import re

import aiohttp

from pybonk.avatar.avatar import Avatar
from pybonk.bot.bonk_bot import BonkBot
from pybonk.bot.exceptions import BonkLoginError


class GuestBonkBot(BonkBot):
    """
    Bot class for bonk.io guest account.
    """

    def __init__(self) -> None:
        super().__init__()

    def login(self, username: str) -> None:
        asyncio.run(self._login(username))

    async def _login(self, username: str) -> None:
        pattern = re.compile(r"""[:;,.`~!@"'?#$%^&*()+=/|><\-]""")

        if not (len(username) in range(2, 16)) or pattern.findall(username) or not username.isascii():
            raise BonkLoginError("Username must be between 2 and 16 characters and contain only numbers, letters and _")

        self.aiohttp_session = aiohttp.ClientSession()
        self.username = username
        self.xp = 0
        self.avatars = ["", "", "", "", ""]
        self.main_avatar = self.avatars[0]
        self.games = []

        for attr in dir(self):
            if attr.startswith("on"):
                self.event_emitter.on(attr.replace("on_", ""), getattr(self, attr))

        self.event_emitter.emit("ready")

        while not self.aiohttp_session.closed:
            await asyncio.sleep(0.1)

