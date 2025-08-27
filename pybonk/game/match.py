import asyncio
import time
from typing import Union, List, TYPE_CHECKING

from pybonk.game.game_inputs import GameInput
from pybonk.maps.bonk1_map import Bonk1Map
from pybonk.maps.bonk2_map import Bonk2Map
from pybonk.maps.own_map import OwnMap

if TYPE_CHECKING:
    from pybonk.bot.bonk_bot import BonkBot
    from pybonk.bot.guest_bonk_bot import GuestBonkBot
    from pybonk.bot.account_bonk_bot import AccountBonkBot
    from pybonk.game.game import Game


class Match:
    def __init__(
        self,
        bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]",
        game: "Game",
        bonk_map: Union[OwnMap, Bonk2Map, Bonk1Map],
        offset=0
    ) -> None:
        self.bot: "Union[BonkBot, GuestBonkBot, AccountBonkBot]" = bot
        self.game: "Game" = game
        self.bonk_map: Union[OwnMap, Bonk2Map, Bonk1Map] = bonk_map
        self.inputs: List[GameInput] = []
        self.__offset: int = offset
        self.__start = time.time()

    @property
    def current_frame(self) -> int:
        return int((time.time() - self.__start) * 30) + self.__offset

    async def move(self, time_in_ms: int, *input_keys: GameInput) -> None:
        """
        Presses input key(s) in the game.

        :param time_in_ms: how long should key be pressed (in ms).
        :param input_keys: what keys should be pressed.
        """

        self.inputs.extend(input_keys)

        new_input = 0

        for input_key in self.inputs:
            new_input |= input_key.value

        await self.__send_move_packet(new_input)
        await asyncio.sleep(time_in_ms / 1000)

        for unpressed_key in input_keys:
            new_input ^= unpressed_key.value
            self.inputs.remove(unpressed_key)

        await self.__send_move_packet(new_input)

    async def __send_move_packet(self, bits: int) -> None:
        await self.game.socket_client.emit(
            4,
            {
                "i": bits,
                "f": self.current_frame,
                "c": self.game.bot_move_count
            }
        )

        self.game.bot_move_count += 1
