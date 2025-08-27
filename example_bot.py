from pybonk.bot.login import bonk_guest_login
from pybonk.bot.guest_bonk_bot import GuestBonkBot
from pybonk.game.game import Game
from pybonk.game.message import Message


bot = bonk_guest_login("ABOBA")
bot.main_avatar = bot.avatars[1]


@bot.event
async def on_ready():
    for i in range(50):
        await bot.create_game(name=f"Cool room {i}")


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


bot.run()
