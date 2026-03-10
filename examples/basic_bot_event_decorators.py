import asyncio

from pybonk import GuestBonkBot, Game, Message


bot = GuestBonkBot()


@bot.event
async def on_ready() -> None:
    print("Connected")


@bot.event
async def on_game_connect(game: Game):
    print(f"Connected game {game.room_name}")
    print(game.join_link)


@bot.event
async def on_error(error):
    print(error)


@bot.event
async def on_message(message: Message):
    if not message.author.is_bot and message.content == "!ping":
        await message.game.send_message("Pong!")


@bot.event
async def on_game_disconnect(game: Game):
    print(f"Disconnected from game {game.room_name}")


asyncio.run(bot.login("ABOBA"))
