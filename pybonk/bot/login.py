import re
import requests

from pybonk.avatar.avatar import Avatar
from pybonk.bot.account_bonk_bot import AccountBonkBot
from pybonk.bot.guest_bonk_bot import GuestBonkBot
from pybonk.parsers import decode_avatar
from pybonk.settings import links


def bonk_account_login(username: str, password: str) -> AccountBonkBot:
    """
    Creates bot on bonk.io account.

    :param username: bonk.io account username. Be aware that some usernames like "___" or "%_e" aren't supported.
    :param password: bonk.io account password.

    Example usage::

        bot = bonk_account_login("name", "pass")
        print(bot.username)
    """

    data = requests.post(
        links["login"],
        {
            "username": username,
            "password": password,
            "remember": "false"
        }
    ).json()

    if data.get("e") == "username_fail":
        raise BonkLoginError(f"Invalid username {username}")
    elif data.get("e") == "password":
        raise BonkLoginError(f"Invalid password for account {username}")

    avatars = [data["avatar1"], data["avatar2"], data["avatar3"], data["avatar4"], data["avatar5"]]

    bot = AccountBonkBot(
        data["token"],
        data["id"],
        data["username"],
        data["xp"],
        avatars,
        decode_avatar(avatars[data["activeAvatarNumber"] - 1]),
        data["legacyFriends"]
    )

    return bot


def bonk_guest_login(username: str) -> GuestBonkBot:
    """
    Creates bot without bonk.io account (some methods like get_friend_list() are unavailable).
    Note that guest account can change its nickname if someone in the game has the same nickname.

    Example usage::

        bot = bonk_account_login("name")
        print(bot.username)

    :param username: guest username.
    """

    pattern = re.compile(r"""[:;,.`~!@"'?#$%^&*()+=/|><\-]""")

    if not (len(username) in range(2, 16)) or pattern.findall(username) or not username.isascii():
        raise BonkLoginError("Username must be between 2 and 16 characters and contain only numbers, letters and _")

    bot = GuestBonkBot(
        username,
        0,
        None,
        Avatar().load_from_json({"layers": [], "bc": 4492031})
    )

    return bot


class BonkLoginError(Exception):
    """Raised when bonk login is failed."""

    def __init__(self, message: str) -> None:
        self.message = message
