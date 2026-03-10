from enum import IntEnum
from typing import List


class PlayerInput(IntEnum):
    NONE = 0
    LEFT = 1
    RIGHT = 2
    UP = 4
    DOWN = 8
    HEAVY = 16
    SPECIAL = 32


def move_direction_from_number(number: int) -> List[PlayerInput]:
    """
    Parses the move bits to get input keys pressed for move.

    :param number: move bits.
    """

    heavy = 15 < number < 32
    special = 31 < number < 46
    heavy_and_special = number > 45
    movement_clear_number = number
    inputs = []

    if heavy:
        movement_clear_number -= 16
        inputs.append(PlayerInput.HEAVY)
    elif special:
        movement_clear_number -= 32
        inputs.append(PlayerInput.SPECIAL)
    elif heavy_and_special:
        movement_clear_number -= 48
        inputs.append(PlayerInput.HEAVY)
        inputs.append(PlayerInput.SPECIAL)

    move_directions = {
        0: [PlayerInput.NONE],
        1: [PlayerInput.LEFT],
        2: [PlayerInput.RIGHT],
        3: [PlayerInput.LEFT, PlayerInput.RIGHT],
        4: [PlayerInput.UP],
        5: [PlayerInput.UP, PlayerInput.LEFT],
        6: [PlayerInput.UP, PlayerInput.RIGHT],
        7: [PlayerInput.UP, PlayerInput.LEFT, PlayerInput.RIGHT],
        8: [PlayerInput.DOWN],
        9: [PlayerInput.DOWN, PlayerInput.LEFT],
        10: [PlayerInput.DOWN, PlayerInput.RIGHT],
        11: [PlayerInput.DOWN, PlayerInput.LEFT, PlayerInput.RIGHT],
        12: [PlayerInput.UP, PlayerInput.DOWN],
        13: [PlayerInput.UP, PlayerInput.DOWN, PlayerInput.LEFT],
        14: [PlayerInput.UP, PlayerInput.DOWN, PlayerInput.RIGHT],
        15: [PlayerInput.UP, PlayerInput.DOWN, PlayerInput.LEFT, PlayerInput.RIGHT]
    }

    inputs.extend(move_directions[movement_clear_number])

    return inputs
