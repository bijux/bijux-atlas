from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
from pathlib import Path

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path

RELAXATION_FILES = (
    "configs/policy/pin-relaxations.json",
    "configs/policy/budget-relaxations.json",
    "configs/policy/layer-relaxations.json",
    "configs/policy/ops-smoke-budget-relaxations.json",
    "configs/policy/ops-lint-relaxations.json",
)
SELF_CLI = ["python3", "-m", "bijux_atlas_scripts.cli"]


def _run(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    return proc.returncode, ((proc.stdout or "") + (proc.stderr or "")).strip()


def _validate_ops_lint_relax_schema(repo_root: Path) -> list[str]:
    import jsonschema

    schema = json.loads((repo_root / "configs/_schemas/ops-lint-relaxations.schema.json").read_text(encoding="utf-8"))
    data = json.loads((repo_root / "configs/policy/ops-lint-relaxations.json").read_text(encoding="utf-8"))
    errs: list[str] = []
    try:
        jsonschema.validate(data, schema)
    except jsonschema.ValidationError as exc:
        errs.append(f"configs/policy/ops-lint-relaxations.json schema violation: {exc.message}")
    return errs


def _extract_entries(payload: dict[str, object]) -> list[dict[str, str]]:
    if isinstance(payload.get("exceptions"), list):
        return [x for x in payload.get("exceptions", []) if isinstance(x, dict)]  # type: ignore[return-value]
    if isinstance(payload.get("relaxations"), list):
        return [x for x in payload.get("relaxations", []) if isinstance(x, dict)]  # type: ignore[return-value]
    return []


def _check_relaxations(repo_root: Path, require_docs_ref: bool) -> tuple[int, dict[str, object]]:
    today = dt.date.today()
    errs: list[str] = []
    active: list[dict[str, str]] = []
    for rel in RELAXATION_FILES:
        path = repo_root / rel
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        for item in _extract_entries(payload):
            rid = str(item.get("id", item.get("check_id", ""))).strip()
            owner = str(item.get("owner", "")).strip()
            issue = str(item.get("issue", "")).strip()
            expiry_raw = str(item.get("expiry", item.get("expires_on", ""))).strip()
            if not rid:
                errs.append(f"{rel}: missing id/check_id")
                continue
            if not owner:
                errs.append(f"{rel}:{rid}: missing owner")
            if not issue:
                errs.append(f"{rel}:{rid}: missing issue")
            try:
                expiry = dt.date.fromisoformat(expiry_raw)
            except ValueError:
                errs.append(f"{rel}:{rid}: invalid expiry `{expiry_raw}`")
                continue
            if expiry < today:
                errs.append(f"{rel}:{rid}: expired on {expiry_raw}")
            else:
                active.append({"id": rid, "owner": owner, "issue": issue, "expiry": expiry_raw, "file": rel})
            if require_docs_ref:
                docs_parts: list[str] = []
                docs_root = repo_root / "docs"
                if docs_root.exists():
                    for p in docs_root.rglob("*.md"):
                        docs_parts.append(p.read_text(encoding="utf-8", errors="ignore"))
                if rid not in "\n".join(docs_parts):
                    errs.append(f"{rel}:{rid}: not referenced in docs")

    errs.extend(_validate_ops_lint_relax_schema(repo_root))
    status = 0 if not errs else 1
    payload = {"schema_version": 1, "active_relaxations": active, "errors": errs}
    return status, payload


def _bypass_scan(repo_root: Path) -> tuple[int, dict[str, object]]:
    patt = re.compile(r"\b(?:BYPASS|SKIP_CHECK|NO_VERIFY|ALLOW_BYPASS)\b")
    offenders: list[str] = []
    for base in ("makefiles", "scripts", "ops", ".github/workflows"):
        root = repo_root / base
        if not root.exists():
            continue
        for p in root.rglob("*"):
            if not p.is_file() or p.suffix not in {".sh", ".py", ".mk", ".yml", ".yaml", ".md", ".json"}:
                continue
            rel = p.relative_to(repo_root).as_posix()
            text = p.read_text(encoding="utf-8", errors="ignore")
            for i, line in enumerate(text.splitlines(), 1):
                if patt.search(line) and "RELAXATION_ID" not in line:
                    if "allowlist" in line.lower() or line.strip().startswith("#"):
                        continue
                    offenders.append(f"{rel}:{i}")
                    break
    payload = {"schema_version": 1, "offenders": offenders}
    return (0 if not offenders else 1), payload


def _write_report(ctx: RunContext, section: str, payload: dict[str, object]) -> None:
    out = ensure_evidence_path(ctx, ctx.evidence_root / "policies" / section / ctx.run_id / "report.json")
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _scan_rust_relaxations(repo_root: Path, out_path: Path) -> dict[str, object]:
    findings: list[dict[str, object]] = []
    scan_roots = [repo_root / "crates", repo_root / "packages"]
    for root in scan_roots:
        if not root.exists():
            continue
        for path in sorted(root.rglob("*.rs")):
            if "generated" in path.parts:
                continue
            rel = path.relative_to(repo_root).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            for idx, line in enumerate(text.splitlines(), 1):
                trimmed = line.strip()
                exception_id = None
                for part in line.split():
                    if part.startswith("ATLAS-EXC-"):
                        exception_id = part.strip(",;")
                        break
                if "#[cfg(test)]" in trimmed or "#[cfg_attr(test" in trimmed:
                    findings.append(
                        {
                            "source": "rust-ast",
                            "pattern_id": "cfg_test_attribute",
                            "requires_exception": False,
                            "severity": "info",
                            "file": rel,
                            "line": idx,
                            "exception_id": exception_id,
                        }
                    )
                if (
                    trimmed.startswith("#[allow(")
                    or trimmed.startswith("#![allow(")
                    or (trimmed.startswith("#[cfg_attr(") and "allow(" in trimmed)
                ):
                    findings.append(
                        {
                            "source": "rust-ast",
                            "pattern_id": "allow_attribute",
                            "requires_exception": True,
                            "severity": "error",
                            "file": rel,
                            "line": idx,
                            "exception_id": exception_id,
                        }
                    )
    findings = sorted(findings, key=lambda x: (str(x["file"]), int(x["line"]), str(x["pattern_id"])))
    payload = {"schema_version": 1, "findings": findings}
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return payload


def run_policies_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    repo = ctx.repo_root

    if ns.policies_cmd == "relaxations-check":
        code, payload = _check_relaxations(repo, require_docs_ref=getattr(ns, "require_docs_ref", False))
        if ns.emit_artifacts:
            _write_report(ctx, "relaxations", payload)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print("policy relaxations passed" if code == 0 else "policy relaxations failed")
        if code != 0 and ns.report != "json":
            for err in payload["errors"][:20]:
                print(f"- {err}")
        return code

    if ns.policies_cmd == "ownership-check":
        code, payload = _check_relaxations(repo, require_docs_ref=False)
        payload["errors"] = [e for e in payload["errors"] if "missing owner" in e]
        code = 0 if not payload["errors"] else 1
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print("policy ownership passed" if code == 0 else "policy ownership failed")
        return code

    if ns.policies_cmd == "bypass-scan":
        code, payload = _bypass_scan(repo)
        if ns.emit_artifacts:
            _write_report(ctx, "bypass-scan", payload)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print("policy bypass scan passed" if code == 0 else "policy bypass scan failed")
        if code != 0 and ns.report != "json":
            for x in payload["offenders"][:20]:
                print(f"- {x}")
        return code

    if ns.policies_cmd == "report":
        _, payload = _check_relaxations(repo, require_docs_ref=False)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    if ns.policies_cmd == "check":
        steps = [
            [*SELF_CLI, "policies", "relaxations-check", "--report", "json"],
            [*SELF_CLI, "policies", "bypass-scan", "--report", "json"],
            ["make", "-s", "policy-lint"],
            ["make", "-s", "policy-schema-drift"],
            ["make", "-s", "policy-audit"],
            ["make", "-s", "policy-enforcement-status"],
            ["make", "-s", "policy-allow-env-lint"],
        ]
        errors: list[str] = []
        for cmd in steps:
            code, out = _run(cmd, repo)
            if code != 0:
                errors.append(f"{' '.join(cmd)} => {out.splitlines()[:1][0] if out else 'failed'}")
                if ns.fail_fast:
                    break
        payload = {"schema_version": 1, "errors": errors, "status": "pass" if not errors else "fail"}
        if ns.emit_artifacts:
            _write_report(ctx, "check", payload)
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else f"policies check: {payload['status']}")
        return 0 if not errors else 1

    if ns.policies_cmd == "scan-rust-relaxations":
        out_rel = getattr(ns, "out", None) or "artifacts/policy/relaxations-rust.json"
        out_path = repo / out_rel
        payload = _scan_rust_relaxations(repo, out_path)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(out_path.as_posix())
        return 0

    return 2


def configure_policies_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("policies", help="policy relaxations and bypass checks")
    ps = p.add_subparsers(dest="policies_cmd", required=True)

    check = ps.add_parser("check", help="run canonical policies checks")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--emit-artifacts", action="store_true")
    check.add_argument("--fail-fast", action="store_true")

    relax = ps.add_parser("relaxations-check", help="validate policy relaxations and expiry")
    relax.add_argument("--report", choices=["text", "json"], default="text")
    relax.add_argument("--emit-artifacts", action="store_true")
    relax.add_argument("--require-doc-refs", action="store_true")

    own = ps.add_parser("ownership-check", help="ensure all relaxations have owners")
    own.add_argument("--report", choices=["text", "json"], default="text")

    bp = ps.add_parser("bypass-scan", help="scan for bypass patterns missing RELAXATION_ID")
    bp.add_argument("--report", choices=["text", "json"], default="text")
    bp.add_argument("--emit-artifacts", action="store_true")

    rep = ps.add_parser("report", help="print active relaxations summary")
    rep.add_argument("--report", choices=["text", "json"], default="json")

    rust_scan = ps.add_parser("scan-rust-relaxations", help="scan Rust sources for relaxation markers")
    rust_scan.add_argument("--out", help="output JSON path", default="artifacts/policy/relaxations-rust.json")
    rust_scan.add_argument("--report", choices=["text", "json"], default="text")
