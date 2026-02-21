from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path


def scan_rust_relaxations(repo_root: Path, out_path: Path) -> dict[str, object]:
    findings: list[dict[str, object]] = []
    for root in [repo_root / "crates", repo_root / "packages"]:
        if not root.exists():
            continue
        for path in sorted(root.rglob("*.rs")):
            if "generated" in path.parts:
                continue
            rel = path.relative_to(repo_root).as_posix()
            for idx, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), 1):
                trimmed = line.strip()
                exception_id = next((part.strip(",;") for part in line.split() if part.startswith("ATLAS-EXC-")), None)
                if "#[cfg(test)]" in trimmed or "#[cfg_attr(test" in trimmed:
                    findings.append({"source": "rust-ast", "pattern_id": "cfg_test_attribute", "requires_exception": False, "severity": "info", "file": rel, "line": idx, "exception_id": exception_id})
                if trimmed.startswith("#[allow(") or trimmed.startswith("#![allow(") or (trimmed.startswith("#[cfg_attr(") and "allow(" in trimmed):
                    findings.append({"source": "rust-ast", "pattern_id": "allow_attribute", "requires_exception": True, "severity": "error", "file": rel, "line": idx, "exception_id": exception_id})
    findings = sorted(findings, key=lambda item: (str(item["file"]), int(item["line"]), str(item["pattern_id"])))
    payload = {"schema_version": 1, "findings": findings}
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return payload


def scan_grep_relaxations(repo_root: Path, out_path: Path) -> dict[str, object]:
    patterns: tuple[tuple[str, str, bool, str], ...] = (
        ("allowlist_token", "allowlist", False, "info"),
        ("skip_token", r"\bskip\b", False, "info"),
        ("bypass_token", "bypass", False, "warning"),
        ("cfg_test_token", r"cfg\(test\)", False, "info"),
        ("todo_relax_token", "TODO relax", True, "error"),
        ("unsafe_token", r"\bunsafe\b", False, "warning"),
        ("unwrap_token", r"unwrap\(", False, "warning"),
        ("temporary_token", "temporary", True, "warning"),
        ("compat_token", "compat", False, "info"),
        ("legacy_token", "legacy", False, "info"),
        ("ignore_token", r"\bignore\b", False, "info"),
    )
    findings: list[dict[str, object]] = []
    scan_paths = ["crates", "scripts", "makefiles", ".github/workflows", "Makefile"]
    include_globs = ["*.rs", "*.sh", "*.py", "*.mk", "*.yml", "*.yaml", "Makefile"]
    for pattern_id, regex, requires_exception, severity in patterns:
        cmd = ["rg", "-n", "--no-heading", "-S", regex, *(str(repo_root / p) for p in scan_paths), *(f"-g{g}" for g in include_globs), "-g!**/target/**", "-g!**/artifacts/**"]
        proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
        for line in (proc.stdout or "").splitlines():
            parts = line.split(":", 2)
            if len(parts) < 3:
                continue
            file_abs, line_no, text = parts
            file_rel = Path(file_abs).resolve().relative_to(repo_root.resolve()).as_posix()
            if file_rel == "packages/atlasctl/src/atlasctl/policies/command.py":
                continue
            match = re.search(r"(ATLAS-EXC-[0-9]{4})", text)
            findings.append({"source": "grep", "pattern_id": pattern_id, "requires_exception": requires_exception, "severity": severity, "file": file_rel, "line": int(line_no), "exception_id": match.group(1) if match else None})
    findings.sort(key=lambda item: (str(item["file"]), int(item["line"]), str(item["pattern_id"])))
    payload = {"schema_version": 1, "findings": findings}
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return payload


def policy_drift_diff(repo_root: Path, from_ref: str, to_ref: str) -> str:
    paths = [
        "configs/policy/policy.json",
        "configs/policy/policy.schema.json",
        "configs/policy/policy-relaxations.json",
        "configs/policy/policy-enforcement-coverage.json",
        "docs/contracts/POLICY_SCHEMA.json",
    ]
    lines: list[str] = []
    for path in paths:
        lines.append(f"### {path}: {from_ref}..{to_ref}")
        proc = subprocess.run(["git", "diff", "--", from_ref, to_ref, "--", path], cwd=repo_root, text=True, capture_output=True, check=False)
        lines.append((proc.stdout or "").rstrip())
    return "\n".join(lines).rstrip() + "\n"
