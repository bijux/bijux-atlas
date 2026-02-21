from __future__ import annotations

import json
import os
import re
from collections import defaultdict
from datetime import date, timedelta
from pathlib import Path


def _k8s_checks_layout(repo_root: Path) -> tuple[int, str]:
    checks_dir = repo_root / "ops" / "k8s" / "tests" / "checks"
    errors: list[str] = []
    max_files = 10

    for area in sorted(p for p in checks_dir.iterdir() if p.is_dir() and p.name != "_lib"):
        direct_tests = sorted(area.glob("test_*.sh"))
        has_submodules = any(p.is_dir() for p in area.iterdir())
        if len(direct_tests) > max_files and not has_submodules:
            errors.append(
                f"{area.relative_to(repo_root)} has {len(direct_tests)} test files; max {max_files} without submodules"
            )

    cfg = checks_dir / "config"
    for path in sorted(cfg.glob("test_*.sh")):
        if "config" not in path.name and "envfrom" not in path.name:
            errors.append(f"{path.relative_to(repo_root)} is under config/ but does not look config-related")

    manifest = json.loads((repo_root / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    for test in manifest.get("tests", []):
        groups = {g for g in test.get("groups", []) if isinstance(g, str)}
        if "obs" in groups:
            errors.append(f"{test.get('script')}: ambiguous group `obs` forbidden; use `observability`")

    if errors:
        return 1, "k8s checks layout lint failed\n" + "\n".join(f"- {e}" for e in errors)
    return 0, "k8s checks layout lint passed"


def _k8s_flakes(repo_root: Path) -> tuple[int, str]:
    report = repo_root / "artifacts/ops/k8s/flake-report.json"
    if not report.exists():
        return 0, "flake report missing; skipping"

    payload = json.loads(report.read_text(encoding="utf-8"))
    count = int(payload.get("flake_count", 0))
    if count == 0:
        return 0, "no flakes detected"

    lines = [f"flake detected: {count}"]
    for flake in payload.get("flakes", []):
        lines.append(f"- {flake.get('script')} owner={flake.get('owner')} attempts={flake.get('attempts')}")

    issue_path = repo_root / "artifacts/ops/k8s/flake-issue.md"
    issue_path.parent.mkdir(parents=True, exist_ok=True)
    ttl_days = int(os.environ.get("ATLAS_FLAKE_TTL_DAYS", "14"))
    quarantine_until = (date.today() + timedelta(days=ttl_days)).isoformat()
    body = [
        "# K8s E2E Flake Detected",
        "",
        f"Count: {count}",
        "",
        f"Quarantine TTL: {ttl_days} days (until `{quarantine_until}`)",
        "",
        "## Flakes",
    ]
    for flake in payload.get("flakes", []):
        body.append(f"- `{flake.get('script')}` owner={flake.get('owner')} attempts={flake.get('attempts')}")
    body.append("\nAction: quarantine with TTL in `ops/k8s/tests/manifest.json`.")
    body.append(f"Set `quarantine_until` to `{quarantine_until}` or later for each flaky test.")
    issue_path.write_text("\n".join(body) + "\n", encoding="utf-8")

    if os.environ.get("CI", "").lower() in {"1", "true", "yes"}:
        return 1, "\n".join(lines)
    return 0, "\n".join(lines + ["flake policy warning (non-CI)"])


def _k8s_test_contract(repo_root: Path) -> tuple[int, str]:
    manifest = json.loads((repo_root / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    ownership = json.loads((repo_root / "ops/k8s/tests/ownership.json").read_text(encoding="utf-8"))
    tests = manifest["tests"]
    owners = ownership["owners"]
    errors: list[str] = []

    all_scripts = {t["script"] for t in tests}
    scripts_by_name: dict[str, list[str]] = {}
    for script in all_scripts:
        scripts_by_name.setdefault(Path(script).name, []).append(script)

    for test in tests:
        if "owner" not in test:
            errors.append(f"missing owner in manifest: {test['script']}")
        if "timeout_seconds" not in test:
            errors.append(f"missing timeout_seconds in manifest: {test['script']}")
        groups = test.get("groups")
        if not isinstance(groups, list) or not groups:
            errors.append(f"missing/non-list groups in manifest: {test['script']}")
        efm = test.get("expected_failure_modes")
        if not isinstance(efm, list) or not efm:
            errors.append(f"missing/non-list expected_failure_modes in manifest: {test['script']}")
        if groups != sorted(groups or []):
            errors.append(f"manifest groups must be sorted for deterministic ordering: {test['script']}")

        script_path = repo_root / "ops/k8s/tests" / test["script"]
        if script_path.exists():
            body = script_path.read_text(encoding="utf-8")
            emitted = {
                m.lower()
                for m in re.findall(r"failure_mode\\s*[:=]\\s*([a-z0-9_]+)", body, flags=re.IGNORECASE)
            }
            declared = {m.lower() for m in test.get("expected_failure_modes", []) if isinstance(m, str)}
            undeclared = sorted(emitted - declared)
            if undeclared:
                errors.append(f"script emits undeclared failure_mode(s) {undeclared} for {test['script']}")

    claimed = set()
    for owner, scripts in owners.items():
        for script in scripts:
            resolved = script
            if script not in all_scripts and "/" not in script:
                matches = scripts_by_name.get(script, [])
                if len(matches) == 1:
                    resolved = matches[0]
                elif len(matches) > 1:
                    errors.append(f"ownership map test '{script}' is ambiguous for owner '{owner}': {matches}")
                    continue
            claimed.add(resolved)
            if resolved not in all_scripts:
                errors.append(f"ownership map has unknown test '{resolved}' for owner '{owner}'")

    for script in sorted(all_scripts):
        if script not in claimed:
            errors.append(f"manifest test not in ownership map: {script}")

    for test in tests:
        if test["owner"] not in owners:
            errors.append(f"manifest owner '{test['owner']}' not declared in ownership map for {test['script']}")

    if errors:
        return 1, "\n".join(errors)
    return 0, "k8s test contract check passed"


def _k8s_test_lib(repo_root: Path) -> tuple[int, str]:
    lib_dir = repo_root / "ops/k8s/tests/checks/_lib"
    files = sorted(p for p in lib_dir.glob("*.sh") if p.is_file())
    if len(files) > 10:
        return 1, f"k8s test lib contract failed: {lib_dir.relative_to(repo_root)} has {len(files)} files (max 10)"
    for path in files:
        text = path.read_text(encoding="utf-8")
        if "k8s-test-common.sh" not in text:
            return (
                1,
                f"k8s test lib contract failed: {path.relative_to(repo_root)} must source canonical k8s-test-common.sh",
            )
    return 0, "k8s test lib contract passed"


def _k8s_surface_generate(repo_root: Path) -> tuple[int, str]:
    manifest = json.loads((repo_root / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    suites = json.loads((repo_root / "ops/k8s/tests/suites.json").read_text(encoding="utf-8"))
    tests = sorted(manifest.get("tests", []), key=lambda x: x["script"])
    by_group: dict[str, list[str]] = defaultdict(list)
    for test in tests:
        for group in sorted(test.get("groups", [])):
            by_group[group].append(test["script"])

    out_index = repo_root / "ops/k8s/tests/INDEX.md"
    out_doc = repo_root / "docs/_generated/k8s-test-surface.md"

    lines = ["# K8s Tests Index", "", "Generated from `ops/k8s/tests/manifest.json`.", "", "## Groups"]
    for group in sorted(by_group):
        lines.append(f"- `{group}` ({len(by_group[group])})")
    lines.extend(["", "## Tests"])
    for test in tests:
        lines.append(f"- `{test['script']}` groups={','.join(test.get('groups', []))} owner={test.get('owner','unknown')}")
    out_index.write_text("\n".join(lines) + "\n", encoding="utf-8")

    suite_map = {s["id"]: sorted(s.get("groups", [])) for s in suites.get("suites", [])}
    doc = ["# K8s Test Surface", "", "Generated from `ops/k8s/tests/manifest.json` and `ops/k8s/tests/suites.json`.", "", "## Suites"]
    for sid in sorted(suite_map):
        doc.append(f"- `{sid}` groups={','.join(suite_map[sid]) if suite_map[sid] else '*'}")
    doc.extend(["", "## Group -> Tests"])
    for group in sorted(by_group):
        doc.append(f"### `{group}`")
        for script in sorted(by_group[group]):
            doc.append(f"- `{script}`")
        doc.append("")
    out_doc.write_text("\n".join(doc).rstrip() + "\n", encoding="utf-8")
    return 0, f"generated {out_index.relative_to(repo_root)} and {out_doc.relative_to(repo_root)}"


