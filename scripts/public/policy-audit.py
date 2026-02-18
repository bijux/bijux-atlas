#!/usr/bin/env python3
# owner: platform
# purpose: enforce policy-relaxation registry rules and emit deterministic audit reports.
# stability: public
# called-by: make policy-audit, make ci-policy-relaxations
from __future__ import annotations

import argparse
import datetime as dt
import difflib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REGISTRY = ROOT / "configs/policy/policy-relaxations.json"
GREP_OUT = ROOT / "artifacts/policy/relaxations-grep.json"
RUST_OUT = ROOT / "artifacts/policy/relaxations-rust.json"
REPORT_OUT = ROOT / "artifacts/policy/policy-audit-report.json"
PREV_OUT = ROOT / "artifacts/policy/policy-audit-report.prev.json"


def run(cmd: list[str]) -> None:
    subprocess.run(cmd, cwd=ROOT, check=True)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text())


def is_wildcard(value: str) -> bool:
    return any(tok in value for tok in ("*", "?", "[", "]"))


def is_sha256(value: str) -> bool:
    return len(value) == 64 and all(ch in "0123456789abcdef" for ch in value)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    run(["./scripts/policy/find_relaxations.sh", str(GREP_OUT)])
    run(["cargo", "run", "--manifest-path", "xtask/Cargo.toml", "--", "scan-relaxations", str(RUST_OUT)])

    grep_findings = load_json(GREP_OUT).get("findings", [])
    rust_findings = load_json(RUST_OUT).get("findings", [])
    findings = sorted(grep_findings + rust_findings, key=lambda f: (f["file"], f["line"], f["pattern_id"]))
    registry = load_json(REGISTRY)
    exceptions = registry.get("exceptions", [])
    budgets = registry.get("exception_budgets", {})
    by_id = {entry["id"]: entry for entry in exceptions}

    violations: list[str] = []
    today = dt.date.today()

    for entry in exceptions:
        for key in ("id", "policy", "scope", "file", "justification", "expiry", "owner", "risk"):
            if not str(entry.get(key, "")).strip():
                violations.append(f"exception {entry.get('id', '<missing-id>')} missing required field: {key}")
        expiry_raw = str(entry.get("expiry", ""))
        try:
            expiry = dt.date.fromisoformat(expiry_raw)
            if expiry < today:
                violations.append(f"exception {entry['id']} expired on {expiry_raw}")
        except ValueError:
            violations.append(f"exception {entry.get('id', '<missing-id>')} has invalid expiry: {expiry_raw}")
        if is_wildcard(str(entry.get("file", ""))) or is_wildcard(str(entry.get("scope", ""))):
            violations.append(f"exception {entry['id']} uses wildcard in file/scope (deny-by-default)")
        policy_name = str(entry.get("policy", ""))
        if "allowlist" in policy_name:
            artifact_hash = str(entry.get("artifact_hash", "")).strip()
            if not is_sha256(artifact_hash):
                violations.append(
                    f"exception {entry['id']} allowlist scope must be content-addressed via artifact_hash (sha256)"
                )
            dataset_identity = entry.get("dataset_identity")
            if not isinstance(dataset_identity, dict):
                violations.append(
                    f"exception {entry['id']} allowlist scope must include dataset_identity"
                )
            else:
                for key in ("release", "species", "assembly"):
                    if not str(dataset_identity.get(key, "")).strip():
                        violations.append(
                            f"exception {entry['id']} dataset_identity missing {key}"
                        )
            if str(entry.get("scope", "")).strip() == "repo":
                violations.append(
                    f"exception {entry['id']} allowlist scope cannot be repo-wide; scope to a dataset identity"
                )

    counts: dict[str, int] = {}
    for entry in exceptions:
        policy = str(entry.get("policy", ""))
        counts[policy] = counts.get(policy, 0) + 1
    for policy, count in sorted(counts.items()):
        budget = int(budgets.get(policy, 0))
        if count > budget:
            violations.append(f"policy {policy} exceeds exception budget: {count} > {budget}")

    findings_requiring = [f for f in findings if bool(f.get("requires_exception"))]
    for f in findings_requiring:
        exc_id = f.get("exception_id")
        if not exc_id:
            violations.append(
                f"relaxation without ATLAS-EXC tag: {f['pattern_id']} {f['file']}:{f['line']}"
            )
            continue
        if exc_id not in by_id:
            violations.append(
                f"relaxation references unknown exception id {exc_id}: {f['file']}:{f['line']}"
            )

    referenced = {str(f.get("exception_id")) for f in findings_requiring if f.get("exception_id")}
    for entry in exceptions:
        if entry["id"] not in referenced:
            violations.append(f"exception {entry['id']} not referenced by any code tag")

    report = {
        "schema_version": 1,
        "summary": {
            "findings_total": len(findings),
            "findings_require_exception": len(findings_requiring),
            "exceptions_total": len(exceptions),
            "violations_total": len(violations),
        },
        "violations": sorted(violations),
        "findings": findings,
        "exceptions": exceptions,
    }
    REPORT_OUT.parent.mkdir(parents=True, exist_ok=True)
    if REPORT_OUT.exists():
        PREV_OUT.write_text(REPORT_OUT.read_text())
    REPORT_OUT.write_text(json.dumps(report, indent=2) + "\n")

    if PREV_OUT.exists():
        old = PREV_OUT.read_text().splitlines()
        new = REPORT_OUT.read_text().splitlines()
        diff = "\n".join(difflib.unified_diff(old, new, fromfile=str(PREV_OUT), tofile=str(REPORT_OUT), lineterm=""))
        if diff:
            print(diff)

    print(REPORT_OUT)
    if args.enforce and violations:
        for v in violations:
            print(f"policy-audit violation: {v}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
