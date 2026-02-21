from __future__ import annotations

import argparse
import datetime as dt
import json
import subprocess
from pathlib import Path

from ..core.context import RunContext


def _load(repo_root: Path) -> dict[str, object]:
    return json.loads((repo_root / "configs/layout/script-shim-expiries.json").read_text(encoding="utf-8"))


def _load_stale_exceptions(repo_root: Path) -> tuple[list[dict[str, str]], list[str]]:
    path = repo_root / "configs/layout/compat-stale-usage-exceptions.json"
    if not path.exists():
        return [], []
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows: list[dict[str, str]] = []
    errs: list[str] = []
    today = dt.date.today()
    for item in payload.get("exceptions", []):
        if not isinstance(item, dict):
            continue
        match = str(item.get("match", "")).strip()
        rid = str(item.get("id", "<missing-id>"))
        exp_raw = str(item.get("expires_on", ""))
        if not match:
            errs.append(f"compat stale-usage exception {rid} missing match")
            continue
        try:
            exp = dt.date.fromisoformat(exp_raw)
        except ValueError:
            errs.append(f"compat stale-usage exception {rid} invalid expires_on `{exp_raw}`")
            continue
        if exp < today:
            errs.append(f"compat stale-usage exception {rid} expired on {exp_raw}")
            continue
        rows.append({"id": rid, "match": match})
    return rows, errs


def _compat_list(repo_root: Path) -> dict[str, object]:
    cfg = _load(repo_root)
    rows: list[dict[str, str]] = []
    for row in cfg.get("shims", []):
        if not isinstance(row, dict):
            continue
        rows.append(
            {
                "path": str(row.get("path", "")),
                "replacement": str(row.get("replacement_cli", row.get("replacement", ""))),
                "expires_on": str(row.get("expires_on", "")),
                "owner": str(row.get("owner", "")),
            }
        )
    rows.sort(key=lambda x: x["path"])
    return {"schema_version": 1, "shims": rows}


def _compat_check(repo_root: Path, include_docs: bool) -> tuple[int, dict[str, object]]:
    today = dt.date.today()
    cfg = _load(repo_root)
    errors: list[str] = []
    shims = [row for row in cfg.get("shims", []) if isinstance(row, dict)]
    ignore_files = {
        "configs/layout/script-shim-expiries.json",
        "configs/layout/compat-stale-usage-exceptions.json",
        "docs/development/tooling/scripts-changelog.md",
        "docs/development/tooling/atlasctl.md",
    }
    stale_ex, stale_ex_errs = _load_stale_exceptions(repo_root)
    errors.extend(stale_ex_errs)
    for row in shims:
        rel = str(row.get("path", ""))
        exp_raw = str(row.get("expires_on", ""))
        if not rel:
            errors.append("shim entry missing path")
            continue
        p = repo_root / rel
        if not p.exists():
            errors.append(f"shim missing on disk: {rel}")
        try:
            exp = dt.date.fromisoformat(exp_raw)
            if exp < today:
                errors.append(f"shim expired: {rel} expired_on={exp_raw}")
        except ValueError:
            errors.append(f"shim expiry invalid: {rel} expires_on={exp_raw}")

    scan_paths = ["makefiles", "scripts", "ops", ".github/workflows"]
    if include_docs:
        scan_paths.append("docs")
    stale_usage: list[str] = []
    use_rg = bool(subprocess.run(["sh", "-c", "command -v rg >/dev/null 2>&1"], cwd=repo_root).returncode == 0)
    for row in shims:
        rel = str(row.get("path", ""))
        if not rel:
            continue
        if use_rg:
            cmd = ["rg", "-n", "--fixed-strings", rel, *scan_paths]
            proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
            lines = (proc.stdout or "").splitlines()
        else:
            lines = []
            for base in scan_paths:
                p = repo_root / base
                if not p.exists():
                    continue
                for file_path in p.rglob("*"):
                    if not file_path.is_file():
                        continue
                    if file_path.stat().st_size > 2_000_000:
                        continue
                    try:
                        text = file_path.read_text(encoding="utf-8", errors="ignore")
                    except OSError:
                        continue
                    if rel in text:
                        for i, line in enumerate(text.splitlines(), start=1):
                            if rel in line:
                                lines.append(f"{file_path.relative_to(repo_root).as_posix()}:{i}:{line.strip()}")
        for ln in lines:
            parts = ln.split(":", 2)
            if len(parts) < 2:
                continue
            file_rel = parts[0]
            if file_rel in ignore_files:
                continue
            if any(file_rel.startswith(ex["match"]) for ex in stale_ex):
                continue
            if file_rel == rel:
                continue
            stale_usage.append(ln)
    if stale_usage:
        errors.append("stale shim usage detected")
    payload = {
        "schema_version": 1,
        "status": "pass" if not errors else "fail",
        "active_shims": len(shims),
        "errors": errors,
        "stale_usage": stale_usage[:200],
    }
    return (0 if not errors else 1), payload


def _load_workspace_version(repo_root: Path) -> str:
    for line in (repo_root / "Cargo.toml").read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("version = "):
            return stripped.split("=", 1)[1].strip().strip('"').strip("'")
    raise ValueError("unable to detect workspace version from Cargo.toml")


def _next_minor_floor(version: str) -> str:
    major, minor, _patch = [int(p) for p in version.split(".", 2)]
    return f"{major}.{minor + 1}.0"


def _render_compat_matrix(atlas_version: str) -> str:
    umb_range = f">={atlas_version},<{_next_minor_floor(atlas_version)}"
    major = atlas_version.split(".", 1)[0]
    return (
        "# Compatibility Matrix: bijux Umbrella <-> bijux-atlas\n\n"
        "| bijux umbrella | bijux-atlas plugin | status | notes |\n"
        "|---|---|---|---|\n"
        f"| `{major}.x` | `{atlas_version}` | supported | plugin advertises `compatible_umbrella: {umb_range}` |\n"
        f"| future major | `{atlas_version}` | unsupported | plugin handshake must fail compatibility check |\n\n"
        "## Validation Rule\n\n"
        "The umbrella validates plugin metadata range against umbrella semver before dispatch.\n"
        "If incompatible, the umbrella returns a structured machine error and does not execute plugin commands.\n"
    )


def _compat_update_matrix(repo_root: Path, tag: str) -> tuple[int, str]:
    atlas_version = tag[1:] if tag.startswith("v") else tag
    out = repo_root / "docs/reference/compatibility/umbrella-atlas-matrix.md"
    out.write_text(_render_compat_matrix(atlas_version), encoding="utf-8")
    return 0, out.relative_to(repo_root).as_posix()


def _compat_validate_matrix(repo_root: Path) -> tuple[int, dict[str, object]]:
    ver = _load_workspace_version(repo_root)
    current = repo_root / "docs/reference/compatibility/umbrella-atlas-matrix.md"
    expected = _render_compat_matrix(ver)
    actual = current.read_text(encoding="utf-8")
    ok = actual == expected
    return (0 if ok else 1), {
        "schema_version": 1,
        "status": "pass" if ok else "fail",
        "workspace_version": ver,
        "file": current.relative_to(repo_root).as_posix(),
        "error": "" if ok else f"compatibility matrix is out of date for workspace version {ver}",
    }


def run_compat_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    repo = ctx.repo_root
    if ns.compat_cmd == "list":
        payload = _compat_list(repo)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            for row in payload["shims"]:
                print(f"{row['path']} -> {row['replacement']} (expires {row['expires_on']})")
        return 0
    if ns.compat_cmd == "check":
        code, payload = _compat_check(repo, ns.include_docs)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"compat check: {payload['status']} active_shims={payload['active_shims']}")
            for err in payload["errors"]:
                print(f"- {err}")
        return code
    if ns.compat_cmd == "update-matrix":
        code, out = _compat_update_matrix(repo, ns.tag)
        print(out)
        return code
    if ns.compat_cmd == "validate-matrix":
        code, payload = _compat_validate_matrix(repo)
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            if payload["status"] == "pass":
                print("compatibility matrix is up to date")
            else:
                print(payload["error"])
        return code
    return 2


def configure_compat_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("compat", help="scripts compatibility shim inspection commands")
    cs = p.add_subparsers(dest="compat_cmd", required=True)

    lst = cs.add_parser("list", help="list deprecated shim aliases and replacements")
    lst.add_argument("--report", choices=["text", "json"], default="text")

    chk = cs.add_parser("check", help="check shim expiry and stale usage")
    chk.add_argument("--report", choices=["text", "json"], default="text")
    chk.add_argument("--include-docs", action="store_true")

    upd = cs.add_parser("update-matrix", help="update umbrella-atlas compatibility matrix for a release tag")
    upd.add_argument("--tag", required=True)

    vld = cs.add_parser("validate-matrix", help="validate umbrella-atlas compatibility matrix against workspace version")
    vld.add_argument("--report", choices=["text", "json"], default="text")
