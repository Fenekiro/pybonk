from typing import List, TYPE_CHECKING

from pybonk.avatar.avatar_layer import Layer

if TYPE_CHECKING:
    from pybonk.parsers import decode_avatar


class Avatar:
    def __init__(self) -> None:
        self.layers: List[Layer] = []
        self.base_color: int = 0

    def load_from_encoded_string(self, encoded_string: str) -> "Avatar":
        avatar = decode_avatar(encoded_string)

        self.layers = avatar.layers
        self.base_color = avatar.base_color

        return self

    def load_from_json(self, avatar_dict: dict) -> "Avatar":
        self.base_color = avatar_dict["bc"]
        self.layers = [Layer(*layer.values()) for layer in avatar_dict["layers"]]

        return self

    def to_json(self) -> dict:
        layers = [layer.__dict__ for layer in self.layers]

        return {"layers": layers, "bc": self.base_color}

