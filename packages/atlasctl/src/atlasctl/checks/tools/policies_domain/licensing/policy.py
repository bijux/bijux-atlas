from __future__ import annotations

from pathlib import Path


_MIT_MARKER = "MIT License"
_COPYRIGHT_MARKER = "Copyright (c) 2026 Bijan Mousavi"
_FORBIDDEN_LICENSE_PHRASES = (
    "inherits repository licensing policy",
    "No separate package license grant is introduced here",
    "Apache License",
    "GPL",
)


def check_license_file_mit(repo_root: Path) -> tuple[int, list[str]]:
    package_root = repo_root / "packages/atlasctl"
    license_file = package_root / "LICENSE"
    errors: list[str] = []
    if (package_root / "LICENSE.md").exists():
        errors.append("forbidden license file: packages/atlasctl/LICENSE.md (use plain LICENSE)")
    if not license_file.exists():
        errors.append("missing license file: packages/atlasctl/LICENSE")
        return 1, errors
    text = license_file.read_text(encoding="utf-8")
    if _MIT_MARKER not in text:
        errors.append("packages/atlasctl/LICENSE does not contain MIT license text marker")
    if _COPYRIGHT_MARKER not in text:
        errors.append("packages/atlasctl/LICENSE missing required copyright line")
    return (0 if not errors else 1), errors


def check_license_statements_consistent(repo_root: Path) -> tuple[int, list[str]]:
    package_root = repo_root / "packages/atlasctl"
    targets = [
        package_root / "README.md",
        package_root / "docs/license.md",
        package_root / "docs/contracts.md",
    ]
    errors: list[str] = []
    for target in targets:
        if not target.exists():
            errors.append(f"missing expected license statement file: {target.relative_to(repo_root)}")
            continue
        text = target.read_text(encoding="utf-8")
        if "MIT" not in text:
            errors.append(f"{target.relative_to(repo_root)} must reference MIT licensing")
        for phrase in _FORBIDDEN_LICENSE_PHRASES:
            if phrase in text:
                errors.append(f"{target.relative_to(repo_root)} contains conflicting license statement: {phrase}")
    return (0 if not errors else 1), errors


def check_spdx_policy(repo_root: Path) -> tuple[int, list[str]]:
    """SPDX policy: headers are optional, but if present they must be MIT."""
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    errors: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        lines = path.read_text(encoding="utf-8").splitlines()[:5]
        for line in lines:
            marker = "SPDX-License-Identifier:"
            if marker in line:
                value = line.split(marker, 1)[1].strip()
                if value != "MIT":
                    errors.append(
                        f"{path.relative_to(repo_root)} has non-MIT SPDX identifier: {value}"
                    )
    return (0 if not errors else 1), errors
