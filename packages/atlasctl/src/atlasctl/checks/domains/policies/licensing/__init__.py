from __future__ import annotations

from ....core.base import CheckCategory, CheckDef
from .policy import check_license_file_mit, check_license_statements_consistent, check_spdx_policy

CHECKS: tuple[CheckDef, ...] = (
    CheckDef(
        "license.file_mit",
        "license",
        "ensure package license file exists and is MIT",
        150,
        check_license_file_mit,
        category=CheckCategory.CONTRACT,
        fix_hint="Create packages/atlasctl/LICENSE with canonical MIT text and copyright.",
    ),
    CheckDef(
        "license.statements_consistent",
        "license",
        "ensure package docs contain only MIT-compatible license statements",
        150,
        check_license_statements_consistent,
        category=CheckCategory.POLICY,
        fix_hint="Update README/docs license references to MIT and remove conflicting statements.",
    ),
    CheckDef(
        "license.spdx_policy",
        "license",
        "enforce SPDX policy for python source headers",
        150,
        check_spdx_policy,
        category=CheckCategory.POLICY,
        fix_hint="If SPDX header is present, set it to SPDX-License-Identifier: MIT.",
    ),
)
