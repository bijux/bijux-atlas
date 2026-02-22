from __future__ import annotations

from ....core.base import CheckCategory, CheckDef
from .check_registry_change_gates import (
    check_registry_change_requires_docs_update,
    check_registry_change_requires_golden_update,
    check_registry_change_requires_owner_update,
)
from .check_registry_integrity import check_registry_integrity


CHECKS = (
    CheckDef(
        "checks.registry_integrity",
        "checks",
        "validate checks registry TOML/JSON integrity and drift",
        2500,
        check_registry_integrity,
        category=CheckCategory.CONTRACT,
        fix_hint="Run `./bin/atlasctl gen checks-registry` and commit REGISTRY.toml/REGISTRY.generated.json.",
        slow=True,
        owners=("platform",),
        tags=("checks", "registry"),
    ),
    CheckDef(
        "checks.registry_change_owners_gate",
        "checks",
        "require ownership metadata update when checks registry changes",
        500,
        check_registry_change_requires_owner_update,
        category=CheckCategory.POLICY,
        fix_hint="Update makefiles/ownership.json when REGISTRY.toml changes.",
        owners=("platform",),
        tags=("checks", "registry", "ci"),
    ),
    CheckDef(
        "checks.registry_change_docs_gate",
        "checks",
        "require docs update when checks registry changes",
        500,
        check_registry_change_requires_docs_update,
        category=CheckCategory.POLICY,
        fix_hint="Update docs/checks/registry.md when REGISTRY.toml changes.",
        owners=("platform",),
        tags=("checks", "registry", "ci"),
    ),
    CheckDef(
        "checks.registry_change_goldens_gate",
        "checks",
        "require checks list/tree/owners goldens update when registry changes",
        500,
        check_registry_change_requires_golden_update,
        category=CheckCategory.POLICY,
        fix_hint="Refresh check list/tree/owners goldens when REGISTRY.toml changes.",
        owners=("platform",),
        tags=("checks", "registry", "ci"),
    ),
)
