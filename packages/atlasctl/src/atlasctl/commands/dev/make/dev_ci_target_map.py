from __future__ import annotations

import json
import re
from pathlib import Path

from ....core.context import RunContext
from ....core.fs import ensure_evidence_path

TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:.*?)(?:\s+##\s*(.*))?$")

ALIAS_OF: dict[str, str] = {
    "test-all": "test",
    "fmt-all": "fmt",
    "lint-all": "lint",
    "audit-all": "audit",
    "coverage-all": "coverage",
    "test-contracts": "test",
}


def _parse_makefile_targets(path: Path) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line or line.startswith("\t") or line.startswith("#") or line.startswith("."):
            continue
        match = TARGET_RE.match(line)
        if not match:
            continue
        rows.append(
            {
                "name": match.group(1).strip(),
                "description": (match.group(2) or "").strip(),
            }
        )
    return rows


def _classify_target(target: str, source_file: str) -> str:
    if target.startswith("internal/") or target.startswith("_"):
        return "internal"
    if target.startswith("ci-"):
        return "ci-only"
    if target in {"fmt", "lint", "test", "audit", "coverage", "check", "ci"}:
        return "public"
    return "legacy"


def _map_to_intent(target: str) -> str | None:
    if target == "fmt":
        return "atlasctl dev fmt"
    if target == "lint":
        return "atlasctl dev lint"
    if target == "check":
        return "atlasctl dev check"
    if target == "test":
        return "atlasctl dev test"
    if target == "audit":
        return "atlasctl dev audit"
    if target == "coverage":
        return "atlasctl dev coverage"
    if target in {"fmt-all"}:
        return "atlasctl dev fmt --all"
    if target in {"lint-all"}:
        return "atlasctl dev lint --all"
    if target in {"check-all"}:
        return "atlasctl dev check --all"
    if target in {"test-all"}:
        return "atlasctl dev test --all"
    if target in {"test-contracts"}:
        return "atlasctl dev test --contracts"
    if target in {"audit-all"}:
        return "atlasctl dev audit --all"
    if target in {"coverage-all"}:
        return "atlasctl dev coverage --all"
    if target in {"internal/dev/check"}:
        return "atlasctl dev check"
    if target == "ci":
        return "atlasctl ci run --json --out-dir artifacts/reports/atlasctl/suite-ci"
    if target == "internal/ci/run":
        return "atlasctl ci run --json"
    if target == "ci-fast":
        return "atlasctl dev ci fast"
    if target == "ci-contracts":
        return "atlasctl dev ci contracts"
    if target == "ci-docs":
        return "atlasctl dev ci docs"
    if target == "ci-ops":
        return "atlasctl dev ci ops"
    if target in {"ci-init-iso-dirs", "ci-init-tmp", "ci-dependency-lock-refresh", "ci-release-compat-matrix-verify", "ci-release-build-artifacts", "ci-release-notes-render", "ci-release-publish-gh", "ci-cosign-sign", "ci-cosign-verify", "ci-chart-package-release", "ci-reproducible-verify", "ci-security-advisory-render", "governance-check", "ci-workflows-make-only"}:
        return f"atlasctl make run {target}"
    if target.startswith("ci-"):
        return f"atlasctl make run {target}"
    if target in {"test"}:
        return "atlasctl dev test"
    if target.startswith("internal/") or target.startswith("_") or target:
        return f"atlasctl make run {target}"
    return None


def _duplicate_mapping_errors(rows: list[dict[str, str]]) -> list[str]:
    intent_to_targets: dict[str, list[str]] = {}
    for row in rows:
        intent_to_targets.setdefault(row["atlasctl"], []).append(row["target"])
    errors: list[str] = []
    for intent, targets in sorted(intent_to_targets.items()):
        if len(targets) <= 1:
            continue
        primaries = [target for target in targets if target not in ALIAS_OF]
        if len(primaries) != 1:
            errors.append(
                f"duplicate atlasctl mapping without explicit aliases: intent={intent} targets={','.join(sorted(targets))}"
            )
            continue
        canonical = primaries[0]
        invalid = [target for target in targets if target != canonical and ALIAS_OF.get(target) != canonical]
        if invalid or ALIAS_OF.get(canonical):
            errors.append(
                f"duplicate atlasctl mapping without explicit aliases: intent={intent} targets={','.join(sorted(targets))}"
            )
    return errors


def build_dev_ci_target_payload(repo_root: Path) -> dict[str, object]:
    sources = [repo_root / "makefiles" / "dev.mk"]
    dumps: list[dict[str, object]] = []
    mapping_rows: list[dict[str, str]] = []
    unmapped: list[str] = []
    for source in sources:
        targets = _parse_makefile_targets(source)
        source_rel = source.relative_to(repo_root).as_posix()
        dumps.append({"file": source_rel, "targets": targets})
        for row in targets:
            target = row["name"]
            intent = _map_to_intent(target)
            if not intent:
                unmapped.append(target)
                continue
            mapped: dict[str, str] = {
                "target": target,
                "source": source_rel,
                "description": row["description"],
                "classification": _classify_target(target, source_rel),
                "atlasctl": intent,
            }
            alias_of = ALIAS_OF.get(target)
            if alias_of:
                mapped["alias_of"] = alias_of
            mapping_rows.append(mapped)
    duplicates = _duplicate_mapping_errors(mapping_rows)
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if (unmapped or duplicates) else "ok",
        "sources": [item["file"] for item in dumps],
        "dumps": dumps,
        "target_map": mapping_rows,
        "errors": {
            "unmapped": sorted(unmapped),
            "duplicate_without_alias": duplicates,
        },
    }


def run_dev_ci_target_map(ctx: RunContext, out_dir_arg: str, check: bool, as_json: bool) -> int:
    out_dir = Path(out_dir_arg)
    if not out_dir.is_absolute():
        out_dir = (ctx.repo_root / out_dir).resolve()
    payload = build_dev_ci_target_payload(ctx.repo_root)
    dumps = payload["dumps"]
    dev_dump = next(item for item in dumps if str(item["file"]).endswith("dev.mk"))
    dev_path = ensure_evidence_path(ctx, out_dir / "dev-targets.json")
    map_path = ensure_evidence_path(ctx, out_dir / "ci-target-map.json")
    dev_path.write_text(json.dumps(dev_dump, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    map_path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": payload["status"],
                "target_map": payload["target_map"],
                "errors": payload["errors"],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    result = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": payload["status"],
        "artifacts": {
            "dev_targets": str(dev_path.relative_to(ctx.repo_root)),
            "target_map": str(map_path.relative_to(ctx.repo_root)),
        },
        "errors": payload["errors"],
    }
    if as_json:
        print(json.dumps(result, sort_keys=True))
    else:
        print(
            "make dev-ci-target-map: "
            f"status={result['status']} "
            f"unmapped={len(result['errors']['unmapped'])} "
            f"duplicate_without_alias={len(result['errors']['duplicate_without_alias'])}"
        )
    if check and payload["status"] != "ok":
        return 1
    return 0
