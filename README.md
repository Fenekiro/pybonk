# bonk_bot
User-friendly async python framework for writing bots in bonk.io.
Supported python versions: 3.8+
## Features
- API is using async and await for handling several connections and requests at once
- Different bonk.io servers support
- Event-based (discord.py-like events)
## Installing
Package is currently unavailable on PyPI
## Bot example
### Guest bot

```py
from pybonk import GuestBonkBot, Game, Message

bot = GuestBonkBot()


@bot.event
async def on_ready():
    bot.main_avatar = bot.avatars[0]

    await bot.create_game(name="Cool room", max_players=4)


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


bot.login("Fenekiro")
```
### Account bot

```py
from pybonk import AccountBonkBot, Game, Message

bot = AccountBonkBot()


@bot.event
async def on_ready():
    bot.main_avatar = bot.avatars[0]

    await bot.create_game(name="Cool room", max_players=4)


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


bot.login("Fenekiro", "123")
```
### Defining bot through class inheritance

```py
from pybonk import GuestBonkBot, Game, Message


class MyBot(GuestBonkBot):
    async def on_ready(self):
        bot.main_avatar = bot.avatars[0]

        await bot.create_game(name="Cool room", max_players=4)

    async def on_game_connect(self, game: Game):
        print(f"Connected game {game.room_name}")
        print(game.join_link)

    async def on_error(self, error):
        print(error)

    async def on_message(self, message: Message):
        if not message.author.is_bot and message.content == "!ping":
            await message.game.send_message("Pong!")

    async def on_game_disconnect(self, game: Game):
        print(f"Disconnected from game {game.room_name}")


bot = MyBot()

bot.login("Fenekiro")
```
## Documentation
Coming soon
