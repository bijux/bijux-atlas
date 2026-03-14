"""Python Atlas SDK public exports."""

from .client import AtlasClient
from .config import ClientConfig
from .errors import (
    AtlasApiError,
    AtlasClientError,
    AtlasConfigError,
    AtlasRetryExhaustedError,
)
from .query import QueryRequest
from .version import __version__

try:
    from . import _native as native
except ImportError:  # pragma: no cover - optional native bridge during source-only workflows
    native = None

__all__ = [
    "AtlasApiError",
    "AtlasClient",
    "AtlasClientError",
    "AtlasConfigError",
    "AtlasRetryExhaustedError",
    "ClientConfig",
    "QueryRequest",
    "__version__",
    "native",
]
