from __future__ import annotations
import argparse
import datetime as dt
import json
import re
from pathlib import Path
from ....checks.tools.repo_domain.domains.forbidden_adjectives import check_forbidden_adjectives
from ....checks.tools.make_enforcement import collect_bypass_inventory, render_text_report
from ....core.context import RunContext
from ....core.exec import run
from ....core.fs import ensure_evidence_path
from ....core.runtime.paths import write_text_file
from .culprits import (
    collect_dir_stats,
    evaluate_metric,  # compat for tests/monkeypatch consumers
)
from .budgets.handlers import handle_budget_command
from .dead_modules import analyze_dead_modules
from .scans.repo_scans import policy_drift_diff, scan_grep_relaxations, scan_rust_relaxations
RELAXATION_FILES = ("configs/policy/pin-relaxations.json", "configs/policy/budget-relaxations.json", "configs/policy/layer-relaxations.json", "configs/policy/ops-smoke-budget-relaxations.json", "configs/policy/ops-lint-relaxations.json")
SELF_CLI = ["python3", "-m", "atlasctl.cli"]
_POLICIES_ITEMS: tuple[str, ...] = ("allow-env-lint", "bypass-scan", "bypass-list", "bypass-report", "check", "check-dir-entry-budgets", "check-py-files-per-dir", "culprits", "culprits-biggest-dirs", "culprits-biggest-files", "culprits-files-per-dir", "culprits-largest-files", "culprits-loc-per-dir", "culprits-modules-per-dir", "culprits-suite", "dead-modules", "drift-diff", "enforcement-status", "explain", "forbidden-adjectives", "ownership-check", "relaxations-check", "repo-stats", "report", "report-budgets", "scan-grep-relaxations", "scan-rust-relaxations", "schema-drift")
def _run(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    return proc.returncode, ((proc.stdout or "") + (proc.stderr or "")).strip()

def _validate_ops_lint_relax_schema(repo_root: Path) -> list[str]:
    import jsonschema
    schema = json.loads((repo_root / "configs/schema/ops-lint-relaxations.schema.json").read_text(encoding="utf-8"))
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
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

def _write_out_file(repo_root: Path, out_file: str, content: str) -> None:
    if not out_file:
        return
    out_path = repo_root / out_file
    write_text_file(out_path, content + "\n", encoding="utf-8")


def _bypass_burndown(repo_root: Path, *, update_trend: bool = False) -> dict[str, object]:
    payload = collect_bypass_inventory(repo_root)
    entries = payload.get("entries", []) if isinstance(payload.get("entries"), list) else []
    today = dt.date.today()
    grouped: dict[str, dict[str, object]] = {}
    for row in entries:
        if not isinstance(row, dict):
            continue
        source = str(row.get("source", "")).strip()
        domain = source.split("/", 2)[1] if source.startswith("ops/") else source.split("/", 2)[1] if "/" in source else "repo"
        slot = grouped.setdefault(domain, {"domain": domain, "count": 0, "ages": []})
        slot["count"] = int(slot["count"]) + 1
        created_at = str(row.get("created_at", "")).strip()
        if created_at:
            try:
                age = (today - dt.date.fromisoformat(created_at)).days
                ages = slot.get("ages")
                if isinstance(ages, list):
                    ages.append(age)
            except ValueError:
                pass
    items: list[dict[str, object]] = []
    for domain, slot in grouped.items():
        ages = [int(x) for x in (slot.get("ages") or [])]
        items.append(
            {
                "domain": domain,
                "count": int(slot.get("count", 0)),
                "avg_age_days": round(sum(ages) / len(ages), 1) if ages else None,
                "max_age_days": max(ages) if ages else None,
            }
        )
    items.sort(key=lambda r: (-int(r["count"]), str(r["domain"])))
    result = {
        "schema_version": 1,
        "kind": "bypass-burn-down",
        "entry_count": int(payload.get("entry_count", 0)),
        "items": items,
    }
    if update_trend:
        scorecard_path = repo_root / "ops/_generated.example/scorecard.json"
        scorecard: dict[str, object] = {}
        if scorecard_path.exists():
            try:
                scorecard = json.loads(scorecard_path.read_text(encoding="utf-8"))
            except Exception:
                scorecard = {}
        trend = scorecard.get("bypass_trend")
        rows = trend if isinstance(trend, list) else []
        rows.append({"date": str(today), "count": result["entry_count"]})
        scorecard["bypass_trend"] = rows[-30:]
        write_text_file(scorecard_path, json.dumps(scorecard, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return result


def _bypass_pr_checklist(repo_root: Path) -> dict[str, object]:
    payload = collect_bypass_inventory(repo_root)
    entries = payload.get("entries", []) if isinstance(payload.get("entries"), list) else []
    touched = [row for row in entries if isinstance(row, dict) and str(row.get("source", "")).startswith("ops/_meta/")]
    lines = ["### Bypass Impact", ""]
    if not touched:
        lines.append("- [ ] No bypass inventory entries changed.")
    else:
        lines.append("- [ ] Bypass entries touched in this PR:")
        for row in sorted(touched, key=lambda r: (str(r.get("source", "")), str(r.get("key", ""))))[:50]:
            line = f"  - `{row.get('source')}:{row.get('key')}`"
            if row.get("owner"):
                line += f" owner={row.get('owner')}"
            if row.get("expires_at"):
                line += f" expires={row.get('expires_at')}"
            lines.append(line)
    return {"schema_version": 1, "kind": "bypass-pr-checklist", "entry_count": len(touched), "lines": lines}


def _bypass_drill(repo_root: Path, *, strict: bool) -> dict[str, object]:
    inventory = collect_bypass_inventory(repo_root)
    burndown = _bypass_burndown(repo_root, update_trend=False)
    checks = [
        "checks_policies_bypass_has_owner",
        "checks_policies_bypass_has_expiry",
        "checks_policies_bypass_has_reason",
        "checks_policies_bypass_has_ticket_or_doc_ref",
        "checks_policies_bypass_count_nonincreasing",
    ]
    if strict:
        checks += ["checks_policies_bypass_mainline_strict_mode", "checks_policies_bypass_count_nonincreasing_hard"]
    return {
        "schema_version": 1,
        "kind": "bypass-drill",
        "strict": strict,
        "entry_count": int(inventory.get("entry_count", 0)),
        "oldest_age_days": burndown.get("oldest_age_days"),
        "by_domain": burndown.get("items", []),
        "fix_plan": {
            "stages": ["stale", "overly-broad", "cosmetic", "structural", "hard"],
            "recommended_checks": checks,
            "commands": [
                "./bin/atlasctl policies culprits --format json --blame",
                "./bin/atlasctl policies bypass burn-down --report json",
                "./bin/atlasctl policies bypass tighten --step 1 --report json",
            ],
        },
    }


def _with_bypass_blame(repo_root: Path, payload: dict[str, object]) -> dict[str, object]:
    import fnmatch

    entries = payload.get("entries", [])
    if not isinstance(entries, list):
        return payload
    out_entries: list[dict[str, object]] = []
    file_cache: dict[str, list[str]] = {}
    for row in entries:
        if not isinstance(row, dict):
            continue
        src_rel = str(row.get("source", "")).strip()
        key = str(row.get("key", "")).strip()
        blame: list[str] = []
        src = repo_root / src_rel
        if src.exists():
            lines = file_cache.setdefault(src_rel, src.read_text(encoding="utf-8", errors="ignore").splitlines())
            patterns = [key]
            if "|" in key:
                patterns = key.split("|", 1)
            for i, line in enumerate(lines, start=1):
                text = line.strip()
                if not text or text.startswith("#"):
                    continue
                if any(p and p in line for p in patterns):
                    blame.append(f"{src_rel}:{i}")
                    break
            if not blame and any(ch in key for ch in "*?[]"):
                for i, line in enumerate(lines, start=1):
                    if fnmatch.fnmatch(line.strip(), key):
                        blame.append(f"{src_rel}:{i}")
                        break
        out = dict(row)
        out["blame"] = blame
        out_entries.append(out)
    result = dict(payload)
    result["entries"] = out_entries
    return result


def _repo_stats_payload(repo_root: Path) -> dict[str, object]:
    rows = collect_dir_stats(repo_root)
    top_dirs = sorted(rows, key=lambda row: row.total_loc, reverse=True)[:20]
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "total_dirs": len(rows),
        "top_dirs": [
            {
                "dir": row.dir,
                "py_files": row.py_files,
                "modules": row.modules,
                "shell_files": row.shell_files,
                "total_loc": row.total_loc,
                "rule": row.rule,
                "enforce": row.enforce,
            }
            for row in top_dirs
        ],
    }

def _repo_stats_diff(current: dict[str, object], previous: dict[str, object]) -> dict[str, object]:
    now = {str(item["dir"]): int(item["total_loc"]) for item in current.get("top_dirs", []) if isinstance(item, dict)}
    old = {str(item["dir"]): int(item["total_loc"]) for item in previous.get("top_dirs", []) if isinstance(item, dict)}
    deltas: list[dict[str, object]] = []
    for directory in sorted(set(now) | set(old)):
        deltas.append(
            {
                "dir": directory,
                "loc_now": now.get(directory, 0),
                "loc_prev": old.get(directory, 0),
                "delta_loc": now.get(directory, 0) - old.get(directory, 0),
            }
        )
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "deltas": sorted(deltas, key=lambda row: abs(int(row["delta_loc"])), reverse=True)[:20],
    }

def _policy_schema_drift(repo_root: Path) -> tuple[int, list[str]]:
    schema_path = repo_root / "configs/policy/policy.schema.json"
    config_path = repo_root / "configs/policy/policy.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    config = json.loads(config_path.read_text(encoding="utf-8"))
    required = set(schema.get("required", []))
    config_keys = set(config.keys())
    errs: list[str] = []
    if required != config_keys:
        missing = sorted(required - config_keys)
        extra = sorted(config_keys - required)
        errs.append(f"policy schema drift: required/config keys mismatch; missing={missing} extra={extra}")
    if schema.get("additionalProperties", True):
        errs.append("policy schema drift: top-level additionalProperties must be false")
    canonical = json.dumps(schema, indent=2, sort_keys=True) + "\n"
    if schema_path.read_text(encoding="utf-8") != canonical:
        errs.append("policy schema drift: schema file is not canonical (run formatter/regenerate)")
    return (0 if not errs else 1), errs

def _policy_allow_env_lint(repo_root: Path) -> tuple[int, list[str]]:
    schema = repo_root / "configs/ops/env.schema.json"
    declared = set(json.loads(schema.read_text(encoding="utf-8")).get("variables", {}).keys())
    allow_pattern = re.compile(r"\b(?:ATLAS_ALLOW_[A-Z0-9_]+|ALLOW_NON_KIND)\b")
    rg = run(
        ["rg", "-n", r"\b(?:ATLAS_ALLOW_[A-Z0-9_]+|ALLOW_NON_KIND)\b", "crates", "scripts", "makefiles", ".github", "docs"],
        cwd=repo_root,
        text=True,
        capture_output=True,
    )
    violations: list[str] = []
    for line in rg.stdout.splitlines():
        parts = line.split(":", 2)
        if len(parts) < 3:
            continue
        path, line_no, text = parts
        for token in allow_pattern.findall(text):
            if token not in declared:
                violations.append(f"{path}:{line_no}: undeclared ALLOW var `{token}`")
    return (0 if not violations else 1), sorted(set(violations))

def _policy_enforcement_status(repo_root: Path, enforce: bool) -> tuple[int, list[str], str]:
    coverage = repo_root / "configs/policy/policy-enforcement-coverage.json"
    out = repo_root / "docs/_generated/policy-enforcement-status.md"
    data = json.loads(coverage.read_text(encoding="utf-8"))
    hard = set(data.get("hard_policies", []))
    rows = []
    violations: list[str] = []
    covered_hard = 0
    total_hard = len(hard)
    search_roots = [p for p in ("crates", "scripts", "makefiles", "docs") if (repo_root / p).exists()]

    def _has_ref(needle: str) -> bool:
        if not needle:
            return False
        if not search_roots:
            return False
        proc = run(
            ["rg", "-n", "--fixed-strings", needle, *search_roots],
            cwd=repo_root,
            text=True,
            capture_output=True,
                    )
        return bool(proc.stdout.strip())

    for policy in data.get("policies", []):
        pid = str(policy.get("id", "")).strip()
        pass_test = str(policy.get("pass_test", "")).strip()
        fail_test = str(policy.get("fail_test", "")).strip()
        is_hard = bool(policy.get("hard", False)) or pid in hard
        pass_ok = _has_ref(pass_test)
        fail_ok = _has_ref(fail_test)
        status = "PASS" if pass_ok and fail_ok else "FAIL"
        if is_hard and status == "PASS":
            covered_hard += 1
        if not pass_ok:
            violations.append(f"{pid}: missing pass test reference `{pass_test}`")
        if not fail_ok:
            violations.append(f"{pid}: missing fail test reference `{fail_test}`")
        if pass_test == fail_test:
            violations.append(f"{pid}: pass/fail tests must be distinct")
        rows.append((pid, "hard" if is_hard else "soft", pass_test, fail_test, status))
    hard_percent = 100 if total_hard == 0 else int((covered_hard / total_hard) * 100)
    lines = [
        "# Policy Enforcement Status",
        "",
        "- Owner: `atlas-platform`",
        "- Generated from: `configs/policy/policy-enforcement-coverage.json`",
        f"- Hard policy coverage: `{covered_hard}/{total_hard}` (`{hard_percent}%`)",
        "",
        "| Policy | Class | Pass Test | Fail Test | Status |",
        "| --- | --- | --- | --- | --- |",
    ]
    for pid, klass, p, f, status in sorted(rows, key=lambda r: r[0]):
        lines.append(f"| `{pid}` | `{klass}` | `{p}` | `{f}` | `{status}` |")
    write_text_file(out, "\n".join(lines) + "\n", encoding="utf-8")
    if enforce and hard_percent < 100:
        violations.append("hard policy coverage must be 100%")
    return (0 if not (enforce and violations) else 1), violations, out.relative_to(repo_root).as_posix()

def _forbidden_report_path(repo_root: Path) -> Path:
    config_path = repo_root / "configs/policy/forbidden-adjectives.json"
    if not config_path.exists():
        return repo_root / "artifacts/reports/atlasctl/forbidden-adjectives.json"
    payload = json.loads(config_path.read_text(encoding="utf-8"))
    report_raw = str(payload.get("report_path", "artifacts/reports/atlasctl/forbidden-adjectives.json"))
    report_path = Path(report_raw)
    if report_path.is_absolute():
        return report_path
    return repo_root / report_path

def run_policies_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    repo = ctx.repo_root
    if not getattr(ns, "policies_cmd", None) and bool(getattr(ns, "list", False)):
        if bool(getattr(ns, "json", False)):
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "group": "policies", "items": list(_POLICIES_ITEMS)}, sort_keys=True))
        else:
            for item in _POLICIES_ITEMS:
                print(item)
        return 0

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

    if ns.policies_cmd in {"bypass-list", "bypass"} and getattr(ns, "bypass_cmd", "list") in {"list", "inventory"}:
        payload = collect_bypass_inventory(repo)
        if bool(getattr(ns, "blame", False)):
            payload = _with_bypass_blame(repo, payload)
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else render_text_report(payload))
        return 0 if not payload.get("errors") else 1

    if ns.policies_cmd in {"bypass-report", "bypass"} and getattr(ns, "bypass_cmd", "report") == "report":
        payload = collect_bypass_inventory(repo)
        out_rel = str(getattr(ns, "out", "") or "artifacts/reports/atlasctl/policies-bypass-report.json")
        write_text_file(repo / out_rel, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps({"schema_version": 1, "status": "ok", "out": out_rel}, sort_keys=True) if ns.report == "json" else out_rel)
        return 0 if not payload.get("errors") else 1

    if ns.policies_cmd == "bypass" and getattr(ns, "bypass_cmd", "entry") == "entry":
        payload = collect_bypass_inventory(repo)
        entry_id = str(getattr(ns, "entry_id", ""))
        entry = next((row for row in payload.get("entries", []) if isinstance(row, dict) and entry_id in {str(row.get("key", "")).strip(), f"{str(row.get('source', '')).strip()}:{str(row.get('key', '')).strip()}"}), None) if entry_id else None
        if entry is None:
            out = {"schema_version": 1, "status": "not_found", "entry_id": entry_id}
            print(json.dumps(out, sort_keys=True) if ns.report == "json" else f"bypass entry not found: {entry_id}")
            return 1
        result = {"schema_version": 1, "status": "ok", "entry_id": f"{entry.get('source')}:{entry.get('key')}", "entry": entry, "next_removal_steps": ["Confirm linked issue is active and still scoped for this bypass.", "Implement the owning fix and delete this bypass record from source config.", "Run `./bin/atlasctl check run --group repo` and verify bypass checks remain green."]}
        if ns.report == "json":
            print(json.dumps(result, sort_keys=True))
        else:
            print(f"bypass entry: {result['entry_id']}")
            print(f"owner={entry.get('owner', '(none)')} issue={entry.get('issue_id', '(none)')} expiry={entry.get('expiry', '(none)')}")
            print(f"removal_plan: {entry.get('removal_plan', '(none)')}")
            print("\n".join(f"- {step}" for step in result["next_removal_steps"]))
        return 0

    if ns.policies_cmd == "bypass" and getattr(ns, "bypass_cmd", "") == "tighten":
        payload = collect_bypass_inventory(repo)
        step = max(1, int(getattr(ns, "step", 1)))
        ordered_groups = [
            ("stale", ["checks_policies_bypass_entry_matches_nothing", "checks_policies_bypass_entry_paths_exist"]),
            ("overly-broad", ["checks_policies_bypass_entry_matches_too_broad"]),
            ("cosmetic", ["checks_policies_bypass_has_reason", "checks_policies_bypass_has_ticket_or_doc_ref"]),
            ("structural", ["checks_policies_bypass_has_owner", "checks_policies_bypass_has_expiry", "checks_policies_bypass_has_test_coverage"]),
            ("hard", ["checks_policies_bypass_count_nonincreasing", "checks_policies_bypass_budget_trend", "checks_policies_bypass_mainline_strict_mode"]),
        ]
        idx = min(step - 1, len(ordered_groups) - 1)
        lane, checks = ordered_groups[idx]
        result = {
            "schema_version": 1,
            "kind": "bypass-tighten-plan",
            "step": step,
            "selected_stage": lane,
            "entry_count": int(payload.get("entry_count", 0)),
            "recommended_checks": checks,
            "next_order": [name for name, _ in ordered_groups[idx + 1 :]],
            "removal_order": [name for name, _ in ordered_groups],
        }
        if ns.report == "json":
            print(json.dumps(result, sort_keys=True))
        else:
            print(f"bypass tighten step {step}: {lane}")
            print(f"entry_count={result['entry_count']}")
            for cid in checks:
                print(f"- {cid}")
        return 0

    if ns.policies_cmd == "report":
        _, payload = _check_relaxations(repo, require_docs_ref=False)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    if ns.policies_cmd == "check":
        steps = [
            [*SELF_CLI, "policies", "relaxations-check", "--report", "json", "--require-doc-refs"],
            [*SELF_CLI, "policies", "bypass-scan", "--report", "json"],
            [*SELF_CLI, "policies", "check-dir-entry-budgets", "--json"],
            [*SELF_CLI, "policies", "check-py-files-per-dir", "--json"],
            [*SELF_CLI, "policies", "schema-drift"],
            [*SELF_CLI, "policies", "enforcement-status", "--enforce"],
            [*SELF_CLI, "policies", "allow-env-lint"],
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
        payload = scan_rust_relaxations(repo, out_path)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(out_path.as_posix())
        return 0
    if ns.policies_cmd == "scan-grep-relaxations":
        out_rel = getattr(ns, "out", None) or "artifacts/policy/relaxations-grep.json"
        out_path = repo / out_rel
        payload = scan_grep_relaxations(repo, out_path)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(out_path.as_posix())
        return 0
    if ns.policies_cmd == "schema-drift":
        code, errs = _policy_schema_drift(repo)
        if code == 0:
            print("policy schema drift check passed")
        else:
            for err in errs:
                print(err)
        return code
    if ns.policies_cmd == "allow-env-lint":
        code, violations = _policy_allow_env_lint(repo)
        if code == 0:
            print("allow-env schema lint passed")
        else:
            for v in violations:
                print(f"allow-env violation: {v}")
        return code
    if ns.policies_cmd == "enforcement-status":
        code, violations, out = _policy_enforcement_status(repo, bool(getattr(ns, "enforce", False)))
        print(f"wrote {out}")
        if violations and getattr(ns, "enforce", False):
            for v in violations:
                print(f"policy-enforcement violation: {v}")
        return code
    if ns.policies_cmd == "drift-diff":
        print(policy_drift_diff(repo, ns.from_ref, ns.to_ref), end="")
        return 0
    if ns.policies_cmd == "forbidden-adjectives":
        code, errors = check_forbidden_adjectives(repo)
        report_path = _forbidden_report_path(repo)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok" if code == 0 else "error",
            "report": report_path.relative_to(repo).as_posix() if report_path.is_relative_to(repo) else report_path.as_posix(),
            "errors": errors,
        }
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            if code == 0:
                print(f"forbidden adjectives: pass ({payload['report']})")
            else:
                print(f"forbidden adjectives: fail ({payload['report']})")
                for err in errors[:50]:
                    print(f"- {err}")
        return code
    if ns.policies_cmd == "culprits" and getattr(ns, "culprits_metric", None) in {None, "bypass"}:
        payload = collect_bypass_inventory(repo)
        if bool(getattr(ns, "blame", False)):
            payload = _with_bypass_blame(repo, payload)
        fmt = str(getattr(ns, "format", "") or ns.report)
        output = json.dumps(payload, sort_keys=True) if fmt == "json" else render_text_report(payload)
        _write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0 if not payload.get("errors") else 1
    budget_code = handle_budget_command(ns, repo, _write_out_file)
    if budget_code is not None:
        return budget_code
    if ns.policies_cmd == "dead-modules":
        payload = analyze_dead_modules(repo)
        output = json.dumps(payload, sort_keys=True) if ns.report == "json" else "\n".join(
            ["dead modules candidates:", *[f"- {row['module']} ({row['path']})" for row in payload["candidates"]]]
        )
        _write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        if getattr(ns, "fail_on_found", False) and payload["candidate_count"] > 0:
            return 1
        return 0
    if ns.policies_cmd == "repo-stats":
        payload = _repo_stats_payload(repo)
        out_rel = str(getattr(ns, "out_file", "")) or f"artifacts/reports/atlasctl/repo-stats/{ctx.run_id}.json"
        out_path = repo / out_rel
        write_text_file(out_path, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        if getattr(ns, "diff_previous", False):
            previous = sorted(out_path.parent.glob("*.json"))
            prior = [p for p in previous if p != out_path]
            if prior:
                prev_payload = json.loads(prior[-1].read_text(encoding="utf-8"))
                diff_payload = _repo_stats_diff(payload, prev_payload)
                diff_path = out_path.with_suffix(".diff.json")
                write_text_file(diff_path, json.dumps(diff_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
                payload["diff_file"] = diff_path.relative_to(repo).as_posix()
                payload["diff"] = diff_payload
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if ns.policies_cmd == "bypass" and getattr(ns, "bypass_cmd", "") == "burn-down":
        payload = _bypass_burndown(repo, update_trend=bool(getattr(ns, "update_trend", False)))
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if ns.policies_cmd == "bypass" and getattr(ns, "bypass_cmd", "") == "pr-checklist":
        payload = _bypass_pr_checklist(repo)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print("\n".join(str(x) for x in payload.get("lines", [])))
        return 0
    if ns.policies_cmd == "bypass" and getattr(ns, "bypass_cmd", "") == "drill":
        payload = _bypass_drill(repo, strict=bool(getattr(ns, "strict", False)))
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0

    return 2

def configure_policies_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("policies", help="policy relaxations and bypass checks")
    p.add_argument("--list", action="store_true", help="list available policies commands")
    p.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    ps = p.add_subparsers(dest="policies_cmd", required=False)

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

    bypass_list = ps.add_parser("bypass-list", help="list policy bypass/allowlist inventory")
    bypass_list.add_argument("--report", choices=["text", "json"], default="text")
    bypass_list.add_argument("--blame", action="store_true")
    bypass_report = ps.add_parser("bypass-report", help="emit consolidated policy bypass report")
    bypass_report.add_argument("--report", choices=["text", "json"], default="text")
    bypass_report.add_argument("--out", default="artifacts/reports/atlasctl/policies-bypass-report.json")
    bypass = ps.add_parser("bypass", help="bypass inventory helpers")
    bypass_sub = bypass.add_subparsers(dest="bypass_cmd", required=True)
    bp_list = bypass_sub.add_parser("list", help="list policy bypass/allowlist inventory")
    bp_list.add_argument("--report", choices=["text", "json"], default="text")
    bp_list.add_argument("--blame", action="store_true")
    bp_inv = bypass_sub.add_parser("inventory", help="alias of bypass list inventory")
    bp_inv.add_argument("--report", choices=["text", "json"], default="text")
    bp_inv.add_argument("--blame", action="store_true")
    rep2 = bypass_sub.add_parser("report", help="emit consolidated policy bypass report")
    rep2.add_argument("--report", choices=["text", "json"], default="text")
    rep2.add_argument("--out", default="artifacts/reports/atlasctl/policies-bypass-report.json")
    entry = bypass_sub.add_parser("entry", help="show one bypass entry with next removal steps")
    entry.add_argument("--id", dest="entry_id", required=True)
    entry.add_argument("--report", choices=["text", "json"], default="text")
    tighten = bypass_sub.add_parser("tighten", help="print a staged bypass tightening plan")
    tighten.add_argument("--step", type=int, default=1)
    tighten.add_argument("--report", choices=["text", "json"], default="text")
    burn = bypass_sub.add_parser("burn-down", help="group bypass entries by domain and age")
    burn.add_argument("--report", choices=["text", "json"], default="text")
    burn.add_argument("--update-trend", action="store_true")
    prc = bypass_sub.add_parser("pr-checklist", help="generate a PR checklist snippet for bypasses")
    prc.add_argument("--report", choices=["text", "json"], default="text")
    drill = bypass_sub.add_parser("drill", help="run a local strict bypass drill and print a fix plan")
    drill.add_argument("--strict", action="store_true")
    drill.add_argument("--report", choices=["text", "json"], default="text")

    rep = ps.add_parser("report", help="print active relaxations summary")
    rep.add_argument("--report", choices=["text", "json"], default="json")

    rust_scan = ps.add_parser("scan-rust-relaxations", help="scan Rust sources for relaxation markers")
    rust_scan.add_argument("--out", help="output JSON path", default="artifacts/policy/relaxations-rust.json")
    rust_scan.add_argument("--report", choices=["text", "json"], default="text")

    grep_scan = ps.add_parser("scan-grep-relaxations", help="scan code surfaces for policy-relaxation grep markers")
    grep_scan.add_argument("--out", help="output JSON path", default="artifacts/policy/relaxations-grep.json")
    grep_scan.add_argument("--report", choices=["text", "json"], default="text")
    ps.add_parser("schema-drift", help="detect drift between policy config keys and schema")
    ps.add_parser("allow-env-lint", help="forbid ALLOW_* vars unless declared in env schema")
    enf = ps.add_parser("enforcement-status", help="validate policy enforcement coverage and generate status doc")
    enf.add_argument("--enforce", action="store_true")
    diff = ps.add_parser("drift-diff", help="diff policy contracts between two refs")
    diff.add_argument("--from-ref", default="HEAD~1")
    diff.add_argument("--to-ref", default="HEAD")
    culprits = ps.add_parser("culprits", help="report directory budget culprits")
    culprits.add_argument(
        "culprits_metric",
        nargs="?",
        choices=[
            "bypass",
            "files-per-dir",
            "modules-per-dir",
            "py-files-per-dir",
            "shell-files-per-dir",
            "loc-per-dir",
            "dir-loc",
            "largest-files",
            "imports-per-file",
            "public-symbols-per-file",
            "complexity-heuristic",
        ],
    )
    culprits.add_argument("--report", choices=["text", "json"], default="text")
    culprits.add_argument("--format", choices=["text", "json"], help="alias of --report")
    culprits.add_argument("--blame", action="store_true", help="include file:line blame for bypass inventory mode")
    culprits.add_argument("--out-file", help="write report to file path", default="")
    files_per_dir = ps.add_parser("culprits-files-per-dir", help="report python files-per-dir budget culprits")
    files_per_dir.add_argument("--report", choices=["text", "json"], default="text")
    files_per_dir.add_argument("--out-file", help="write report to file path", default="")
    modules_per_dir = ps.add_parser("culprits-modules-per-dir", help="report modules-per-dir budget culprits")
    modules_per_dir.add_argument("--report", choices=["text", "json"], default="text")
    modules_per_dir.add_argument("--out-file", help="write report to file path", default="")
    loc_per_dir = ps.add_parser("culprits-loc-per-dir", help="report loc-per-dir budget culprits")
    loc_per_dir.add_argument("--report", choices=["text", "json"], default="text")
    loc_per_dir.add_argument("--out-file", help="write report to file path", default="")
    largest_files = ps.add_parser("culprits-largest-files", help="report largest-files budget culprits")
    largest_files.add_argument("--limit", type=int, default=20)
    largest_files.add_argument("--report", choices=["text", "json"], default="text")
    largest_files.add_argument("--out-file", help="write report to file path", default="")
    big_files = ps.add_parser("culprits-biggest-files", help="top N biggest python files")
    big_files.add_argument("--limit", type=int, default=20)
    big_files.add_argument("--report", choices=["text", "json"], default="text")
    big_files.add_argument("--out-file", help="write report to file path", default="")
    big_dirs = ps.add_parser("culprits-biggest-dirs", help="top N biggest python directories")
    big_dirs.add_argument("--limit", type=int, default=20)
    big_dirs.add_argument("--report", choices=["text", "json"], default="text")
    big_dirs.add_argument("--out-file", help="write report to file path", default="")
    suite = ps.add_parser("culprits-suite", help="run full directory budget suite")
    suite.add_argument("--report", choices=["text", "json"], default="text")
    suite.add_argument("--out-file", help="write report to file path", default="")
    dead = ps.add_parser("dead-modules", help="report potential dead modules in atlasctl src tree")
    dead.add_argument("--report", choices=["text", "json"], default="text")
    dead.add_argument("--out-file", help="write report to file path", default="")
    dead.add_argument("--fail-on-found", action="store_true", help="exit non-zero when candidates are found")
    stats = ps.add_parser("repo-stats", help="emit repository directory stats and optional diff against prior run")
    stats.add_argument("--report", choices=["text", "json"], default="json")
    stats.add_argument("--out-file", help="write report to file path", default="")
    stats.add_argument("--diff-previous", action="store_true")
    explain = ps.add_parser("explain", help="explain policy configuration surfaces")
    explain.add_argument("subject", choices=["budgets", "forbidden-adjectives"])
    explain.add_argument("--report", choices=["text", "json"], default="text")
    forbidden = ps.add_parser("forbidden-adjectives", help="scan tracked files for forbidden wording")
    forbidden.add_argument("--report", choices=["text", "json"], default="text")
    entry_budget = ps.add_parser("check-dir-entry-budgets", help="enforce max directory entries per dir in atlasctl source/tests")
    entry_budget.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    entry_budget.add_argument("--print-culprits", action="store_true", help="print only top offenders")
    entry_budget.add_argument("--top", type=int, default=10, help="top offender count for --print-culprits")
    entry_budget.add_argument("--fail-on-warn", action="store_true", help="treat warnings as failures")
    entry_budget.add_argument("--out-file", default="", help="write report to file path")
    py_budget = ps.add_parser("check-py-files-per-dir", help="enforce max .py files per dir (excluding __init__.py)")
    py_budget.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    py_budget.add_argument("--print-culprits", action="store_true", help="print only top offenders")
    py_budget.add_argument("--top", type=int, default=10, help="top offender count for --print-culprits")
    py_budget.add_argument("--fail-on-warn", action="store_true", help="treat warnings as failures")
    py_budget.add_argument("--out-file", default="", help="write report to file path")
    report_budgets = ps.add_parser("report-budgets", help="ranked budget report across entry and python-file budgets")
    report_budgets.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    report_budgets.add_argument("--by-domain", action="store_true", help="aggregate offenders by top-level domain")
    report_budgets.add_argument("--out-file", default="", help="write report to file path")
