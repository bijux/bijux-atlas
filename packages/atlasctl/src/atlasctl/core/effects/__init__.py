"""Effect boundary primitives for process/filesystem/network policies."""

from .policy import all_command_effects, command_effects, command_group, group_allowed_effects

__all__ = [
    "all_command_effects",
    "command_effects",
    "command_group",
    "group_allowed_effects",
]
