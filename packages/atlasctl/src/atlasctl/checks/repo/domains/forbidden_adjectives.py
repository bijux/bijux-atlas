from __future__ import annotations

import datetime as dt
import json
import re
from dataclasses import dataclass
from pathlib import Path

from ....core.process import run_command
from ....core.runtime.paths import write_text_file

_CONFIG_PATH = Path("configs/policy/forbidden-adjectives.json")
_APPROVALS_PATH = Path("configs/policy/forbidden-adjectives-approvals.json")
_REPORT_FALLBACK = Path("artifacts/reports/atlasctl/forbidden-adjectives.json")
_SCAN_EXTENSIONS = {
    ".md",
    ".mk",
    ".py",
    ".json",
    ".yaml",
    ".yml",
    ".toml",
    ".txt",
    ".rst",
    ".sh",
    ".rs",
}


@dataclass(frozen=True)
class Approval:
    path: str
    term: str
    reason: str
    approval_id: str
    expires_on: dt.date
    line: int | None


def _load_policy_config(repo_root: Path) -> tuple[list[str], set[str], Path, list[str]]:
    path = repo_root / _CONFIG_PATH
    if not path.exists():
        return [], set(), _REPORT_FALLBACK, [f"missing config: {_CONFIG_PATH.as_posix()}"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    terms = payload.get("terms", [])
    if not isinstance(terms, list) or not all(isinstance(term, str) and term.strip() for term in terms):
        return [], set(), _REPORT_FALLBACK, [f"{_CONFIG_PATH.as_posix()}: `terms` must be a non-empty string list"]
    exempt_raw = payload.get("exempt_paths", [])
    if exempt_raw and (not isinstance(exempt_raw, list) or not all(isinstance(item, str) and item.strip() for item in exempt_raw)):
        return [], set(), _REPORT_FALLBACK, [f"{_CONFIG_PATH.as_posix()}: `exempt_paths` must be a string list when present"]
    exempt_paths = {item.strip().replace("\\", "/") for item in exempt_raw if isinstance(item, str) and item.strip()}
    report_raw = str(payload.get("report_path", _REPORT_FALLBACK.as_posix()))
    report_path = Path(report_raw)
    if not report_path.is_absolute():
        report_path = repo_root / report_path
    return sorted({term.strip().lower() for term in terms}), exempt_paths, report_path, []


def _load_approvals(repo_root: Path) -> tuple[list[Approval], list[str]]:
    path = repo_root / _APPROVALS_PATH
    if not path.exists():
        return [], [f"missing approvals artifact: {_APPROVALS_PATH.as_posix()}"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    errors: list[str] = []
    if int(payload.get("schema_version", 0)) != 1:
        errors.append(f"{_APPROVALS_PATH.as_posix()}: schema_version must be 1")
    rows = payload.get("approvals", [])
    if not isinstance(rows, list):
        return [], [f"{_APPROVALS_PATH.as_posix()}: `approvals` must be a list"]
    approvals: list[Approval] = []
    today = dt.date.today()
    for idx, row in enumerate(rows):
        if not isinstance(row, dict):
            errors.append(f"{_APPROVALS_PATH.as_posix()}: approvals[{idx}] must be an object")
            continue
        path_value = str(row.get("path", "")).strip()
        term = str(row.get("term", "")).strip().lower()
        reason = str(row.get("reason", "")).strip()
        approval_id = str(row.get("approval_id", "")).strip()
        expires_raw = str(row.get("expires_on", "")).strip()
        line_raw = row.get("line")
        line = int(line_raw) if isinstance(line_raw, int) and line_raw > 0 else None
        if not path_value or not term or not reason or not approval_id or not expires_raw:
            errors.append(f"{_APPROVALS_PATH.as_posix()}: approvals[{idx}] missing required fields")
            continue
        try:
            expires_on = dt.date.fromisoformat(expires_raw)
        except ValueError:
            errors.append(f"{_APPROVALS_PATH.as_posix()}: approvals[{idx}] invalid expires_on `{expires_raw}`")
            continue
        if expires_on < today:
            errors.append(f"{_APPROVALS_PATH.as_posix()}: approvals[{idx}] expired on {expires_raw}")
            continue
        approvals.append(
            Approval(
                path=path_value,
                term=term,
                reason=reason,
                approval_id=approval_id,
                expires_on=expires_on,
                line=line,
            )
        )
    return approvals, errors


def _tracked_files(repo_root: Path) -> list[str]:
    result = run_command(["git", "ls-files"], cwd=repo_root)
    if result.code != 0:
        return []
    return sorted(line.strip() for line in result.stdout.splitlines() if line.strip())


def _is_approved(match_path: str, match_term: str, match_line: int, approvals: list[Approval]) -> bool:
    for approval in approvals:
        if approval.path != match_path:
            continue
        if approval.term != match_term:
            continue
        if approval.line is not None and approval.line != match_line:
            continue
        return True
    return False


def _scan(repo_root: Path, terms: list[str], approvals: list[Approval], exempt_paths: set[str]) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    pattern = re.compile(r"\b(?:" + "|".join(re.escape(term) for term in terms) + r")\b", re.IGNORECASE)
    violations: list[dict[str, object]] = []
    approved: list[dict[str, object]] = []
    for rel in _tracked_files(repo_root):
        if rel in exempt_paths:
            continue
        path = repo_root / rel
        if path.suffix.lower() not in _SCAN_EXTENSIONS or not path.exists() or not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for line_no, line in enumerate(text.splitlines(), start=1):
            for hit in pattern.finditer(line):
                term = hit.group(0).lower()
                row = {"path": rel, "line": line_no, "term": term, "excerpt": line.strip()}
                if _is_approved(rel, term, line_no, approvals):
                    approved.append(row)
                else:
                    violations.append(row)
    return violations, approved


def _write_report(repo_root: Path, report_path: Path, payload: dict[str, object]) -> None:
    target = report_path if report_path.is_absolute() else (repo_root / report_path)
    write_text_file(target, json.dumps(payload, indent=2, sort_keys=True) + "\n")


def check_forbidden_adjectives(repo_root: Path) -> tuple[int, list[str]]:
    terms, exempt_paths, report_path, config_errors = _load_policy_config(repo_root)
    approvals, approval_errors = _load_approvals(repo_root)
    violations, approved = ([], [])
    if terms:
        violations, approved = _scan(repo_root, terms, approvals, exempt_paths)

    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "forbidden-adjectives-report",
        "status": "ok" if not violations and not config_errors and not approval_errors else "error",
        "terms": terms,
        "exempt_paths": sorted(exempt_paths),
        "violations": violations,
        "approved_matches": approved,
        "config_errors": config_errors,
        "approval_errors": approval_errors,
    }
    _write_report(repo_root, report_path, payload)

    errors = [f"{row['path']}:{row['line']}: forbidden adjective `{row['term']}`" for row in violations]
    errors.extend(config_errors)
    errors.extend(approval_errors)
    return (0 if not errors else 1), errors
