from __future__ import annotations

from . import native_loader as _native_loader

__all__ = [name for name in dir(_native_loader) if name.startswith(("check_", "generate_"))]

for _name in __all__:
    globals()[_name] = getattr(_native_loader, _name)
