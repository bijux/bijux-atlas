from __future__ import annotations

from ..repo.native import check_docker_image_size, check_docker_layout, check_docker_policy, check_no_latest_tags
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docker.layout", "docker", "validate docker layout contracts", 1200, check_docker_layout, fix_hint="Align docker files to layout contract."),
    CheckDef("docker.policy", "docker", "validate docker policy contracts", 1200, check_docker_policy, fix_hint="Resolve policy violations in docker contracts."),
    CheckDef("docker.no_latest_tags", "docker", "forbid floating latest image tags", 1200, check_no_latest_tags, fix_hint="Pin image tags to immutable versions."),
    CheckDef("docker.image_size", "docker", "enforce docker image size budget", 1200, check_docker_image_size, fix_hint="Reduce image size or adjust documented budget."),
)
