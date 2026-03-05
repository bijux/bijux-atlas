"""HTTP transport helpers."""

from .config import ClientConfig


class HttpTransport:
    """Minimal transport abstraction for upcoming HTTP adapter migration."""

    def __init__(self, config: ClientConfig) -> None:
        self.config = config
