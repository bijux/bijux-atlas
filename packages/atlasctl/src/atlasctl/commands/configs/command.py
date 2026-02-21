from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover - python<3.11
    tomllib = None  # type: ignore[assignment]
from pathlib import Path

import jsonschema

from ...core.context import RunContext
from ...core.fs import ensure_evidence_path
from ...core.runtime.tooling import read_pins, read_tool_versions

CONFIG_SCHEMA_PAIRS: tuple[tuple[str, str], ...] = (
    ("configs/_meta/ownership.json", "configs/_schemas/configs-ownership.schema.json"),
    ("configs/ops/tool-versions.json", "configs/_schemas/tool-versions.schema.json"),
    ("configs/ops/public-surface.json", "configs/_schemas/public-surface.schema.json"),
    ("configs/policy/policy-relaxations.json", "configs/_schemas/policy-relaxations.schema.json"),
    ("configs/policy/layer-relaxations.json", "configs/_schemas/layer-relaxations.schema.json"),
    ("configs/policy/ops-lint-relaxations.json", "configs/_schemas/ops-lint-relaxations.schema.json"),
    ("configs/policy/layer-live-diff-allowlist.json", "configs/_schemas/layer-live-diff-allowlist.schema.json"),
    ("configs/ops/target-renames.json", "configs/_schemas/target-renames.schema.json"),
    ("configs/ops/hpa-safety-caps.json", "configs/_schemas/hpa-safety-caps.schema.json"),
    ("configs/meta/ownership.json", "configs/_schemas/meta-ownership.schema.json"),
)

SKIP_PARTS = {"_schemas", "__pycache__", ".vale"}
KEY_RE = re.compile(r"`(ATLAS_[A-Z0-9_]+|BIJUX_[A-Z0-9_]+|HOME|HOSTNAME|XDG_CACHE_HOME|XDG_CONFIG_HOME|REDIS_URL)`")
VERSION_NAME = re.compile(r".*version[s]?(?:[-_.].*)?\.(json|yaml|yml|toml)$", re.IGNORECASE)
_CONFIGS_ITEMS: tuple[str, ...] = ("drift", "generate", "print", "sync", "validate")


def normalize_config_key(raw: str) -> str:
    value = raw.strip().replace("-", "_").upper()
    if not re.fullmatch(r"[A-Z][A-Z0-9_]*", value):
        raise ValueError(f"invalid config key: {raw}")
    return value


def _load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _parse_yaml(path: Path) -> str | None:
    proc = subprocess.run(
        [
            "python3",
            "-c",
            "import sys,yaml; yaml.safe_load(open(sys.argv[1], 'r', encoding='utf-8').read())",
            str(path),
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode == 0:
        return None
    return proc.stderr.strip() or proc.stdout.strip() or "yaml parse failed"


def _collect_config_files(repo_root: Path) -> list[Path]:
    out: list[Path] = []
    for p in sorted((repo_root / "configs").rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(repo_root)
        if any(part in SKIP_PARTS for part in rel.parts):
            continue
        out.append(p)
    return out


def _validate_schema_pairs(repo_root: Path) -> list[str]:
    errors: list[str] = []
    for data_rel, schema_rel in CONFIG_SCHEMA_PAIRS:
        try:
            data = _load_json(repo_root / data_rel)
            schema = _load_json(repo_root / schema_rel)
            jsonschema.validate(data, schema)
        except Exception as exc:  # noqa: BLE001
            errors.append(f"{data_rel} vs {schema_rel}: {exc}")
    return errors


def _validate_file_well_formed(repo_root: Path) -> list[str]:
    errors: list[str] = []
    for p in _collect_config_files(repo_root):
        rel = p.relative_to(repo_root).as_posix()
        if p.suffix == ".json":
            try:
                json.loads(p.read_text(encoding="utf-8"))
            except Exception as exc:  # noqa: BLE001
                errors.append(f"{rel}: invalid json ({exc})")
        elif p.suffix in {".yaml", ".yml"}:
            err = _parse_yaml(p)
            if err:
                errors.append(f"{rel}: invalid yaml ({err})")
        elif p.suffix == ".toml":
            if tomllib is None:
                continue
            try:
                tomllib.loads(p.read_text(encoding="utf-8"))
            except Exception as exc:  # noqa: BLE001
                errors.append(f"{rel}: invalid toml ({exc})")
    return errors


def _check_ownership(repo_root: Path) -> list[str]:
    own = _load_json(repo_root / "configs/_meta/ownership.json")
    areas = {str(k) for k in own.get("areas", {}).keys()}
    errors: list[str] = []
    for d in sorted((repo_root / "configs").iterdir()):
        if not d.is_dir() or d.name.startswith("_"):
            continue
        rel = f"configs/{d.name}"
        if rel not in areas:
            errors.append(f"missing ownership mapping: {rel}")
    for area in sorted(areas):
        if not (repo_root / area).exists():
            errors.append(f"ownership points to missing area: {area}")
    return errors


def _check_keys_docs_coverage(repo_root: Path) -> list[str]:
    registry = (repo_root / "configs/config-key-registry.md").read_text(encoding="utf-8")
    docs = (repo_root / "docs/contracts/config-keys.md").read_text(encoding="utf-8")
    internal = {
        ln.strip()
        for ln in (repo_root / "configs/ops/internal-config-keys.txt").read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.strip().startswith("#")
    }
    keys = set(KEY_RE.findall(registry))
    missing = sorted(k for k in keys if k not in internal and f"`{k}`" not in docs)
    return [f"missing docs coverage for key: {k}" for k in missing]


def _check_generated_drift(repo_root: Path) -> list[str]:
    expected = _generate_outputs(repo_root, write=False)
    errors: list[str] = []
    for rel, text in expected.items():
        path = repo_root / rel
        if not path.exists() or path.read_text(encoding="utf-8") != text:
            errors.append(f"generated drift: {rel}")
    return errors


def _generate_outputs(repo_root: Path, write: bool) -> dict[str, str]:
    own = _load_json(repo_root / "configs/_meta/ownership.json").get("areas", {})
    lines_index = ["# Configs Index", "", "Canonical configuration surface for repository behavior.", "", "## Areas"]
    for area in sorted(own):
        lines_index.append(f"- `{area}` owner: `{own[area]}`")
    lines_index += ["", "See also: `configs/_meta/ownership.json`.", ""]
    configs_index = "\n".join(lines_index)

    lines_surface = ["# Configs Surface", "", "Generated from `configs/` structure and ownership map.", ""]
    for d in sorted((repo_root / "configs").iterdir()):
        if not d.is_dir() or d.name.startswith("_"):
            continue
        rel = f"configs/{d.name}"
        owner = own.get(rel, "<unowned>")
        readme = d / "README.md"
        lines_surface.append(f"## `{rel}`")
        lines_surface.append(f"- Owner: `{owner}`")
        lines_surface.append(f"- README: `{readme.relative_to(repo_root).as_posix() if readme.exists() else 'MISSING'}`")
        lines_surface.append("")
    configs_surface = "\n".join(lines_surface)

    reg = (repo_root / "configs/config-key-registry.md").read_text(encoding="utf-8")
    env_keys = sorted(set(k for k in KEY_RE.findall(reg) if k.startswith(("ATLAS_", "BIJUX_"))))
    env_keys.append("ATLAS_DEV_ALLOW_UNKNOWN_ENV")
    env_contract = json.dumps(
        {
            "schema_version": 1,
            "description": "Runtime env allowlist contract for atlas-server; unknown ATLAS_/BIJUX_ keys are rejected unless dev escape hatch is enabled.",
            "enforced_prefixes": ["ATLAS_", "BIJUX_"],
            "dev_mode_allow_unknown_env": "ATLAS_DEV_ALLOW_UNKNOWN_ENV",
            "allowed_env": sorted(set(env_keys)),
        },
        indent=2,
        sort_keys=True,
    ) + "\n"

    tooling = read_tool_versions(repo_root)
    lines_tooling = ["# Tooling Versions", "", "Generated from `configs/ops/tool-versions.json`.", ""]
    for k, v in sorted(tooling.items()):
        lines_tooling.append(f"- `{k}`: `{v}`")
    lines_tooling.append("")
    tooling_doc = "\n".join(lines_tooling)

    outputs = {
        "configs/INDEX.md": configs_index,
        "docs/_generated/configs-surface.md": configs_surface,
        "configs/contracts/env.schema.json": env_contract,
        "docs/_generated/tooling-versions.md": tooling_doc,
    }
    if write:
        for rel, text in outputs.items():
            out = repo_root / rel
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(text, encoding="utf-8")
    return outputs


def _sync_slo(repo_root: Path, write: bool) -> tuple[bool, str]:
    src = _load_json(repo_root / "configs/ops/slo/slo.v1.json")
    expected = {
        "schema_version": src.get("schema_version", 1),
        "source": "configs/ops/slo/slo.v1.json",
        "slis": src.get("slis", []),
        "slos": src.get("slos", []),
        "change_policy": src.get("change_policy", {}),
    }
    dst = repo_root / "configs/slo/slo.json"
    if write:
        dst.write_text(json.dumps(expected, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return True, "configs/slo/slo.json"
    current = _load_json(dst)
    return current == expected, "configs/slo/slo.json"


def _print_payload(repo_root: Path) -> dict[str, object]:
    sources = [
        "configs/policy/policy.json",
        "configs/ops/env.schema.json",
        "configs/ops/tool-versions.json",
        "configs/ops/observability-pack.json",
        "configs/perf/k6-thresholds.v1.json",
        "configs/slo/slo.json",
    ]
    provenance = []
    for src in sources:
        raw = (repo_root / src).read_bytes()
        provenance.append({"path": src, "sha256": hashlib.sha256(raw).hexdigest()})
    return {
        "policy": _load_json(repo_root / "configs/policy/policy.json"),
        "ops_env_schema": _load_json(repo_root / "configs/ops/env.schema.json"),
        "ops_tool_versions": _load_json(repo_root / "configs/ops/tool-versions.json"),
        "ops_pins": read_pins(repo_root),
        "ops_observability_pack": _load_json(repo_root / "configs/ops/observability-pack.json"),
        "perf_thresholds": _load_json(repo_root / "configs/perf/k6-thresholds.v1.json"),
        "slo": _load_json(repo_root / "configs/slo/slo.json"),
        "_provenance": provenance,
    }


def _emit(payload: dict[str, object], report: str) -> None:
    print(json.dumps(payload, sort_keys=True) if report == "json" else json.dumps(payload, indent=2, sort_keys=True))


def run_configs_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    report = getattr(ns, "report", "text")
    repo = ctx.repo_root
    if not getattr(ns, "configs_cmd", None) and bool(getattr(ns, "list", False)):
        if bool(getattr(ns, "json", False)):
            _emit({"schema_version": 1, "tool": "atlasctl", "status": "ok", "group": "configs", "items": list(_CONFIGS_ITEMS)}, "json")
        else:
            for item in _CONFIGS_ITEMS:
                print(item)
        return 0

    if ns.configs_cmd == "print":
        _emit({"schema_version": 1, "tool": "atlasctl", "status": "pass", "payload": _print_payload(repo)}, report)
        return 0

    if ns.configs_cmd == "validate":
        errors = []
        errors.extend(_validate_schema_pairs(repo))
        errors.extend(_validate_file_well_formed(repo))
        errors.extend(_check_ownership(repo))
        errors.extend(_check_keys_docs_coverage(repo))
        ok_sync, _ = _sync_slo(repo, write=False)
        if not ok_sync:
            errors.append("SLO sync drift: configs/slo/slo.json")
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "pass" if not errors else "fail",
            "checks": {
                "schema_pairs": "pass" if not _validate_schema_pairs(repo) else "fail",
                "well_formed": "pass" if not _validate_file_well_formed(repo) else "fail",
                "ownership": "pass" if not _check_ownership(repo) else "fail",
                "keys_docs": "pass" if not _check_keys_docs_coverage(repo) else "fail",
                "slo_sync": "pass" if ok_sync else "fail",
            },
            "errors": errors,
        }
        if getattr(ns, "emit_artifacts", False):
            out = ensure_evidence_path(ctx, repo / "artifacts/evidence/configs/validate" / ctx.run_id / "report.json")
            out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        _emit(payload, report)
        return 0 if not errors else 1

    if ns.configs_cmd == "generate":
        _generate_outputs(repo, write=True)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "pass",
            "outputs": sorted(_generate_outputs(repo, write=False).keys()),
        }
        if getattr(ns, "check", False):
            drift = _check_generated_drift(repo)
            payload["status"] = "pass" if not drift else "fail"
            payload["errors"] = drift
            _emit(payload, report)
            return 0 if not drift else 1
        _emit(payload, report)
        return 0

    if ns.configs_cmd == "sync":
        ok, target = _sync_slo(repo, write=getattr(ns, "write", False))
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "pass" if ok else "fail", "target": target}
        _emit(payload, report)
        return 0 if ok else 1

    if ns.configs_cmd == "drift":
        drift = _check_generated_drift(repo)
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "pass" if not drift else "fail", "errors": drift}
        _emit(payload, report)
        return 0 if not drift else 1

    return 2


def configure_configs_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("configs", help="native configs validation/generation/sync commands")
    p.add_argument("--list", action="store_true", help="list available configs commands")
    p.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    cfg = p.add_subparsers(dest="configs_cmd", required=False)

    c_print = cfg.add_parser("print", help="print canonical merged config payload")
    c_print.add_argument("--report", choices=["text", "json"], default="json")

    c_validate = cfg.add_parser("validate", help="validate schemas, ownership, keys docs, and sync")
    c_validate.add_argument("--report", choices=["text", "json"], default="text")
    c_validate.add_argument("--emit-artifacts", action="store_true")

    c_generate = cfg.add_parser("generate", help="generate configs docs/contracts artifacts")
    c_generate.add_argument("--report", choices=["text", "json"], default="text")
    c_generate.add_argument("--check", action="store_true", help="fail if generated outputs drift")

    c_sync = cfg.add_parser("sync", help="validate or update sync outputs (SLO)")
    c_sync.add_argument("--report", choices=["text", "json"], default="text")
    c_sync.add_argument("--write", action="store_true", help="write synced output")

    c_drift = cfg.add_parser("drift", help="check generated outputs drift")
    c_drift.add_argument("--report", choices=["text", "json"], default="text")
