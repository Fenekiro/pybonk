import asyncio

from pybonk import GuestBonkBot, Game, Message, MapRequestHostPov


class MyBot(GuestBonkBot):
    async def on_ready(self) -> None:
        print("Connected")

        await self.join_game_from_link("https://bonk.io/899827lwvhk")

    async def on_game_connect(self, game: Game) -> None:
        print(f"Connected to game {game.room_name}: {game.join_link}")

    async def on_error(self, error: Exception) -> None:
        print(error)

    async def on_message(self, message: Message) -> None:
        if not message.author.is_bot and message.content == "!ping":
            await message.game.send_message("Pong!")

    async def on_map_request_host(self, map_request: MapRequestHostPov) -> None:
        await map_request.load()

    async def on_game_disconnect(self, game: Game) -> None:
        print(f"Disconnected from game {game.room_name}")


bot = MyBot()

asyncio.run(bot.login("ABOBA"))
