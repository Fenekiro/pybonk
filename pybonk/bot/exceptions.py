class BonkLoginError(Exception):
    """Raised when bonk login is failed."""

    def __init__(self, message: str) -> None:
        self.message: str = message
