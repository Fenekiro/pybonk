from typing import List

from pybonk.room_list.room import Room
from pybonk.types.game_modes import GameMode


class RoomList:
    def __init__(self, rooms: List[Room]) -> None:
        self._rooms: List[Room] = rooms

    def __index__(self, index: int) -> Room:
        return self._rooms[index]

    def find_room_by_id(self, room_id: int) -> Room:
        for room in self._rooms:
            if room.id == room_id:
                return room

        raise Exception("RoomNotFoundError")

    def find_room_by_name(self, room_name: str) -> Room:
        for room in self._rooms:
            if room.name == room_name:
                return room

        raise Exception("RoomNotFoundError")

    def filter_by_game_modes(self, *modes: GameMode) -> List[Room]:
        return list(filter(lambda room: room.mode in modes, self._rooms))
