#!/usr/bin/env python3
# owner: repo-surface
# purpose: inventory legacy references and enforce legacy baseline/policy contracts.
# stability: public
# called-by: make layout-check
from __future__ import annotations

import argparse
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
POLICY = ROOT / "configs/policy/legacy-policy.json"
BASELINE = ROOT / "configs/policy/legacy-baseline.json"
EVIDENCE = ROOT / "artifacts/evidence/legacy/inventory.json"

DOC_PAT = re.compile(r"\b(legacy|deprecated|old|v0)\b", re.IGNORECASE)


def _rg_lines(pattern: str, globs: list[str] | None = None) -> list[str]:
    cmd = ["rg", "-n", "--hidden", "--no-ignore-vcs", pattern, str(ROOT)]
    for g in globs or []:
        cmd.extend(["--glob", g])
    cmd.extend(
        [
            "--glob",
            "!.git/**",
            "--glob",
            "!artifacts/**",
            "--glob",
            "!ops/_artifacts/**",
            "--glob",
            "!ops/_evidence/**",
            "--glob",
            "!**/__pycache__/**",
        ]
    )
    proc = subprocess.run(cmd, check=False, capture_output=True, text=True)
    return [ln for ln in proc.stdout.splitlines() if ln.strip()]


def _callers(token: str, max_items: int = 8) -> list[str]:
    lines = _rg_lines(re.escape(token))
    out = []
    for ln in lines:
        p = ln.split(":", 2)[0]
        rel = str(Path(p).resolve().relative_to(ROOT))
        if rel.startswith("artifacts/"):
            continue
        if rel not in out:
            out.append(rel)
        if len(out) >= max_items:
            break
    return out


def _mk_legacy_targets() -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    for mk in sorted((ROOT / "makefiles").glob("*.mk")):
        text = mk.read_text(encoding="utf-8")
        for m in re.finditer(r"^([A-Za-z0-9_./-]*legacy[A-Za-z0-9_./-]*):", text, flags=re.MULTILINE):
            target = m.group(1)
            entries.append(
                {
                    "kind": "make-target",
                    "value": target,
                    "locations": [str(mk.relative_to(ROOT))],
                    "callers": _callers(target),
                }
            )
    return entries


def _legacy_paths() -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    for pat in ("ops/**/_legacy/**", "scripts/**/legacy/**"):
        for p in sorted(ROOT.glob(pat)):
            if p.is_file():
                rel = str(p.relative_to(ROOT))
                entries.append(
                    {
                        "kind": "script",
                        "value": rel,
                        "locations": [rel],
                        "callers": _callers(rel),
                    }
                )
    return entries


def _doc_refs() -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    for md in sorted((ROOT / "docs").rglob("*.md")):
        if str(md).startswith(str(ROOT / "docs/_generated")):
            continue
        text = md.read_text(encoding="utf-8")
        if DOC_PAT.search(text):
            rel = str(md.relative_to(ROOT))
            entries.append(
                {
                    "kind": "doc-ref",
                    "value": rel,
                    "locations": [rel],
                    "callers": [],
                }
            )
    return entries


def _config_env_refs() -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    env = ROOT / "configs/ops/env.schema.json"
    if env.exists():
        doc = json.loads(env.read_text(encoding="utf-8"))
        for key in sorted(doc.get("variables", {}).keys()):
            if DOC_PAT.search(key):
                entries.append(
                    {
                        "kind": "env-var",
                        "value": key,
                        "locations": [str(env.relative_to(ROOT))],
                        "callers": _callers(key),
                    }
                )
    def walk_keys(obj: Any, prefix: str = "") -> list[str]:
        out: list[str] = []
        if isinstance(obj, dict):
            for k, v in obj.items():
                path = f"{prefix}.{k}" if prefix else str(k)
                if DOC_PAT.search(str(k)):
                    out.append(path)
                out.extend(walk_keys(v, path))
        elif isinstance(obj, list):
            for i, v in enumerate(obj):
                out.extend(walk_keys(v, f"{prefix}[{i}]"))
        return out

    for cfg in sorted((ROOT / "configs").rglob("*.json")):
        rel = str(cfg.relative_to(ROOT))
        try:
            doc = json.loads(cfg.read_text(encoding="utf-8"))
        except Exception:
            continue
        for key_path in walk_keys(doc):
            entries.append(
                {
                    "kind": "config-key",
                    "value": key_path,
                    "locations": [rel],
                    "callers": [],
                }
            )
    return entries


def _classify(entry: dict[str, Any]) -> str:
    if entry["kind"] == "make-target":
        return "Collapse"
    if entry["kind"] in {"script", "env-var"}:
        return "Delete"
    return "Rename"


def _entry_id(entry: dict[str, Any]) -> str:
    return f"{entry['kind']}:{entry['value']}"


def build_inventory() -> dict[str, Any]:
    entries = _mk_legacy_targets() + _legacy_paths() + _doc_refs() + _config_env_refs()
    dedup: dict[str, dict[str, Any]] = {}
    for e in entries:
        eid = _entry_id(e)
        if eid in dedup:
            dedup[eid]["locations"] = sorted(set(dedup[eid]["locations"] + e["locations"]))
            dedup[eid]["callers"] = sorted(set(dedup[eid]["callers"] + e["callers"]))
        else:
            e["id"] = eid
            e["classification"] = _classify(e)
            e["owner"] = "repo-surface"
            dedup[eid] = e
    out = sorted(dedup.values(), key=lambda x: (x["kind"], x["value"]))
    return {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "entries": out,
    }


def apply_baseline(inv: dict[str, Any], baseline_path: Path) -> None:
    if not baseline_path.exists():
        return
    base = json.loads(baseline_path.read_text(encoding="utf-8"))
    by_id = {e["id"]: e for e in base.get("entries", [])}
    for e in inv["entries"]:
        b = by_id.get(e["id"])
        if b:
            e["owner"] = b.get("owner", "")
            e["classification"] = b.get("classification", e["classification"])


def check_policy(inv: dict[str, Any], policy_path: Path, baseline_path: Path) -> list[str]:
    errs: list[str] = []
    policy = json.loads(policy_path.read_text(encoding="utf-8")) if policy_path.exists() else {}
    allow_new = bool(policy.get("allow_new_entries", False))
    purge_enforced = bool(policy.get("purge_enforced", False))

    if baseline_path.exists():
        base = json.loads(baseline_path.read_text(encoding="utf-8"))
        base_ids = {e["id"] for e in base.get("entries", [])}
    else:
        base_ids = set()

    inv_ids = {_entry_id(e) for e in inv.get("entries", [])}
    new_ids = sorted(inv_ids - base_ids)
    if new_ids and not allow_new:
        errs.append(f"legacy policy violation: new legacy entries introduced ({len(new_ids)})")
        errs.extend([f"new-entry: {x}" for x in new_ids[:20]])

    for e in inv.get("entries", []):
        if not e.get("owner"):
            errs.append(f"owner required for legacy entry: {e['id']}")
        if e.get("classification") not in {"Delete", "Rename", "Collapse"}:
            errs.append(f"classification required (Delete|Rename|Collapse): {e['id']}")

    if purge_enforced and inv.get("entries"):
        errs.append("legacy/check failed: purge_enforced=true but legacy entries still exist")

    return errs


def render_text(inv: dict[str, Any]) -> str:
    lines = [f"legacy inventory entries: {len(inv['entries'])}"]
    for e in inv["entries"]:
        lines.append(
            f"- [{e['kind']}] {e['value']} :: {e.get('classification','?')} owner={e.get('owner','') or 'MISSING'}"
        )
    return "\n".join(lines)


def main() -> int:
    p = argparse.ArgumentParser(description="Legacy inventory and policy checker")
    p.add_argument("--json-out", default=str(EVIDENCE))
    p.add_argument("--baseline", default=str(BASELINE))
    p.add_argument("--policy", default=str(POLICY))
    p.add_argument("--format", choices=["text", "json"], default="text")
    p.add_argument("--check-policy", action="store_true")
    p.add_argument("--write-baseline", action="store_true")
    args = p.parse_args()

    inv = build_inventory()
    apply_baseline(inv, Path(args.baseline))

    out_path = Path(args.json_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(inv, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if args.write_baseline:
        Path(args.baseline).write_text(json.dumps(inv, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if args.format == "text":
        print(render_text(inv))
    else:
        print(json.dumps(inv, indent=2, sort_keys=True))

    if args.check_policy:
        errs = check_policy(inv, Path(args.policy), Path(args.baseline))
        if errs:
            for e in errs:
                print(e)
            return 1
        print("legacy policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
