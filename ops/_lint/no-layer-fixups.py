#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import fnmatch
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
E2E = ROOT / "ops/e2e"
RELAXATIONS = ROOT / "configs/policy/layer-relaxations.json"

RULE_E2E_RUN_ONLY = "e2e_only_calls_ops_run_entrypoints"
RULE_HELM_INSTALL = "forbid_helm_upgrade_install_in_e2e"
RULE_HELM_VALUES_WRITE = "forbid_writing_helm_values_from_e2e"
RULE_MUTATING_KUBECTL = "forbid_e2e_cluster_resource_mutations"
RULE_FIXUP_KUBECTL = "forbid_e2e_fixup_kubectl_calls"

MUTATING_VERBS = ("apply", "patch", "replace", "create", "delete", "run", "label", "annotate")
FIXUP_PATTERNS = (
    re.compile(r"\bkubectl\b.*\bpatch\b"),
    re.compile(r"\bkubectl\b.*\blabel\b"),
    re.compile(r"\bkubectl\b.*\bannotate\b"),
    re.compile(r"\bkubectl\b.*\bdelete\s+pod\b"),
)


def load_exceptions() -> dict[str, list[dict[str, str]]]:
    payload = json.loads(RELAXATIONS.read_text(encoding="utf-8"))
    today = dt.date.today()
    by_rule: dict[str, list[dict[str, str]]] = {}
    for exc in payload.get("exceptions", []):
        try:
            expiry = dt.date.fromisoformat(str(exc.get("expiry", "")))
        except ValueError:
            continue
        if expiry < today:
            continue
        rule = str(exc.get("rule", "")).strip()
        if not rule:
            continue
        by_rule.setdefault(rule, []).append(exc)
    return by_rule


def allowed(rule: str, rel_path: str, by_rule: dict[str, list[dict[str, str]]]) -> str | None:
    for exc in by_rule.get(rule, []):
        if fnmatch.fnmatch(rel_path, str(exc.get("path", ""))):
            return str(exc.get("id", ""))
    return None


def iter_e2e_files() -> list[Path]:
    out: list[Path] = []
    for ext in ("*.sh", "*.py"):
        out.extend(sorted(E2E.rglob(ext)))
    return out


def main() -> int:
    by_rule = load_exceptions()
    violations: list[str] = []
    files = iter_e2e_files()

    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        for no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            # Rule 3: e2e scripts should call canonical ops/run entrypoints only.
            non_run_match = re.search(r"ops/(?!run/)[A-Za-z0-9._/-]+", line)
            if non_run_match:
                exc = allowed(RULE_E2E_RUN_ONLY, rel, by_rule)
                if exc is None:
                    violations.append(
                        f"{rel}:{no}: {RULE_E2E_RUN_ONLY}: direct non-run call `{non_run_match.group(0)}`"
                    )

            # Rule 4: direct helm install/upgrade in e2e is forbidden.
            if re.search(r"\bhelm\b", line) and re.search(r"\b(install|upgrade)\b", line):
                exc = allowed(RULE_HELM_INSTALL, rel, by_rule)
                if exc is None:
                    violations.append(
                        f"{rel}:{no}: {RULE_HELM_INSTALL}: direct helm install/upgrade in e2e"
                    )

            # Rule 5: e2e must not write Helm values files.
            if re.search(r"values.*\.(ya?ml)\b", line) and re.search(r"(>|>>|tee\s)", line):
                exc = allowed(RULE_HELM_VALUES_WRITE, rel, by_rule)
                if exc is None:
                    violations.append(
                        f"{rel}:{no}: {RULE_HELM_VALUES_WRITE}: writes Helm values file from e2e"
                    )

            # Rule 2: mutating kubectl calls are forbidden from e2e scripts.
            if re.search(r"\bkubectl\b", line) and any(re.search(rf"\b{verb}\b", line) for verb in MUTATING_VERBS):
                exc = allowed(RULE_MUTATING_KUBECTL, rel, by_rule)
                if exc is None:
                    violations.append(
                        f"{rel}:{no}: {RULE_MUTATING_KUBECTL}: mutating kubectl command in e2e"
                    )

            # Rule 6: explicit fixup kubectl calls must be exception-backed.
            if any(p.search(line) for p in FIXUP_PATTERNS):
                exc = allowed(RULE_FIXUP_KUBECTL, rel, by_rule)
                if exc is None:
                    violations.append(
                        f"{rel}:{no}: {RULE_FIXUP_KUBECTL}: fixup kubectl call requires exception id"
                    )

    if violations:
        print("boundary lint violations:", file=sys.stderr)
        for v in violations:
            print(f"- {v}", file=sys.stderr)
        return 1

    print("boundary lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
