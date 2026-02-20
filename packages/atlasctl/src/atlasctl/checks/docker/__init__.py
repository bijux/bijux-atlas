from __future__ import annotations

from ..repo.legacy_native import check_docker_image_size, check_docker_layout, check_docker_policy, check_no_latest_tags
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("docker/layout", "docker", 1200, check_docker_layout),
    CheckDef("docker/policy", "docker", 1200, check_docker_policy),
    CheckDef("docker/no-latest-tags", "docker", 1200, check_no_latest_tags),
    CheckDef("docker/image-size", "docker", 1200, check_docker_image_size),
)
