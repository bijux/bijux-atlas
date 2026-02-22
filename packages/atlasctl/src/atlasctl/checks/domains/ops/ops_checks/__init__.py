from __future__ import annotations

import datetime as dt
import re
import json
import subprocess
from pathlib import Path

from .check_ops_manifests_schema import run as run_ops_manifests_schema_check
from ....repo.native import (
    check_committed_generated_hygiene,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ....core.base import CheckDef

_OPS_RUN_TEMP_APPROVALS = Path("configs/policy/ops-run-temp-script-approvals.json")
_ISSUE_ID_RE = re.compile(r"^ISSUE-[A-Z0-9-]+$")


def _load_ops_run_temp_approvals(repo_root: Path) -> tuple[list[dict[str, object]], list[str]]:
    path = repo_root / _OPS_RUN_TEMP_APPROVALS
    if not path.exists():
        return [], [f"missing {_OPS_RUN_TEMP_APPROVALS.as_posix()}"]
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [], [f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: invalid json: {exc}"]
    approvals = payload.get("approvals", [])
    if not isinstance(approvals, list):
        return [], [f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: `approvals` must be a list"]
    rows: list[dict[str, object]] = []
    errors: list[str] = []
    today = dt.date.today()
    for idx, item in enumerate(approvals):
        if not isinstance(item, dict):
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}] must be an object")
            continue
        script = str(item.get("script", "")).strip()
        owner = str(item.get("owner", "")).strip()
        issue = str(item.get("issue_id", "")).strip()
        reason = str(item.get("reason", "")).strip()
        expiry = str(item.get("expiry", "")).strip()
        if not script.startswith("ops/run/"):
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].script must start with ops/run/")
        if not owner:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].owner is required")
        if not _ISSUE_ID_RE.match(issue):
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].issue_id invalid (`{issue}`)")
        if not reason:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].reason is required")
        try:
            exp_date = dt.date.fromisoformat(expiry)
        except ValueError:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}].expiry must be YYYY-MM-DD")
            exp_date = None
        if exp_date is not None and exp_date < today:
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approvals[{idx}] expired on {expiry}")
        rows.append(item)
    return rows, errors


def check_ops_manifests_schema(repo_root: Path) -> tuple[int, list[str]]:
    del repo_root
    code = run_ops_manifests_schema_check()
    return code, []


def check_ops_no_direct_script_entrypoints(repo_root: Path) -> tuple[int, list[str]]:
    command_patterns = (
        re.compile(r"(?:^|\s)(?:\./)?ops/(?!run/)[A-Za-z0-9_./-]+\.(?:sh|py)\b"),
        re.compile(r"\b(?:bash|sh)\s+(?:\./)?ops/(?!run/)[A-Za-z0-9_./-]+\.(?:sh|py)\b"),
    )
    errors: list[str] = []
    scan_roots = [
        repo_root / "docs" / "development",
        repo_root / "docs" / "control-plane",
        repo_root / ".github" / "workflows",
        repo_root / "makefiles",
    ]
    for base in scan_roots:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix not in {".md", ".mk", ".yml", ".yaml"}:
                continue
            rel = path.relative_to(repo_root).as_posix()
            if rel.startswith("docs/_generated/") or rel.startswith("docs/_lint/"):
                continue
            for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
                stripped = line.strip()
                if not stripped or stripped.startswith("#"):
                    continue
                for pattern in command_patterns:
                    match = pattern.search(stripped)
                    if match is None:
                        continue
                    snippet = match.group(0).strip()
                    errors.append(f"{rel}:{lineno}: direct ops script entrypoint is forbidden (`{snippet}`)")
    return (0 if not errors else 1), sorted(set(errors))


def check_ops_scripts_are_data_only(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    manifests = repo_root / "ops" / "manifests"
    if not manifests.exists():
        return 1, ["missing ops/manifests directory"]
    for path in sorted(manifests.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(repo_root).as_posix()
        if path.suffix.lower() not in {".json", ".yaml", ".yml"}:
            errors.append(f"{rel}: manifests directory must contain json/yaml files only")
            continue
        if path.read_text(encoding="utf-8", errors="ignore").startswith("#!/"):
            errors.append(f"{rel}: data-only manifest must not be executable script")
    return (0 if not errors else 1), errors


def check_ops_shell_policy(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    run_dir = repo_root / "ops" / "run"
    if not run_dir.exists():
        return 1, ["missing ops/run directory"]
    for path in sorted(run_dir.glob("*.sh")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if '. "$ROOT/ops/_lib/common.sh"' not in text:
            errors.append(f"{rel}: must source ops/_lib/common.sh")
        if "ops_entrypoint_start " not in text:
            errors.append(f"{rel}: missing ops_entrypoint_start")
        if "ops_version_guard " not in text and path.name != "prereqs.sh":
            errors.append(f"{rel}: missing ops_version_guard")
    return (0 if not errors else 1), errors


def check_ops_effect_boundary_imports(repo_root: Path) -> tuple[int, list[str]]:
    """Ops command modules must not import CLI parser layers directly."""
    errors: list[str] = []
    ops_root = repo_root / "packages/atlasctl/src/atlasctl/commands/ops"
    for path in sorted(ops_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            stripped = line.strip()
            if stripped.startswith("#"):
                continue
            if "import atlasctl.cli" in stripped or "from atlasctl.cli" in stripped:
                errors.append(f"{rel}:{lineno}: ops command must not import atlasctl.cli layer")
            if re.search(r"from\s+atlasctl\.commands\.[a-z0-9_]+\.command\s+import", stripped):
                errors.append(f"{rel}:{lineno}: ops command must not import other command parser modules")
    return (0 if not errors else 1), errors


def check_ops_env_schema_is_used(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    parser = repo_root / "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_parser.py"
    runmod = repo_root / "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_run.py"
    ops_mk = repo_root / "makefiles/ops.mk"
    parser_text = parser.read_text(encoding="utf-8", errors="ignore") if parser.exists() else ""
    run_text = runmod.read_text(encoding="utf-8", errors="ignore") if runmod.exists() else ""
    mk_text = ops_mk.read_text(encoding="utf-8", errors="ignore") if ops_mk.exists() else ""
    if "configs/ops/env.schema.json" not in parser_text:
        errors.append("ops parser must default env schema to configs/ops/env.schema.json")
    if "ops_env_cmd" not in run_text or "validate" not in run_text or "print" not in run_text:
        errors.append("ops runtime must expose env validate/print handlers")
    if "./bin/atlasctl ops " not in mk_text:
        errors.append("makefiles/ops.mk must delegate to atlasctl ops interface")
    return (0 if not errors else 1), errors


def check_ops_pins_update_is_deterministic(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pins = repo_root / "configs/ops/pins.json"
    if not pins.exists():
        return 1, ["missing configs/ops/pins.json"]
    before = pins.read_text(encoding="utf-8")
    try:
        payload = json.loads(before)
    except json.JSONDecodeError as exc:
        return 1, [f"configs/ops/pins.json invalid json: {exc}"]
    canonical = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if before != canonical:
        errors.append("configs/ops/pins.json must be canonical sorted JSON")
    # schema stability check for required top-level keys
    required = {"schema_version", "contract_version", "tools", "images", "helm", "datasets", "policy"}
    missing = sorted(required - set(payload.keys()))
    if missing:
        errors.append(f"configs/ops/pins.json missing required keys: {', '.join(missing)}")
    return (0 if not errors else 1), errors


def check_ops_pins_no_unpinned_versions(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pins = repo_root / "configs/ops/pins.json"
    if not pins.exists():
        return 1, ["missing configs/ops/pins.json"]
    payload = json.loads(pins.read_text(encoding="utf-8"))
    tools = payload.get("tools", {})
    if not isinstance(tools, dict):
        errors.append("configs/ops/pins.json `tools` must be an object")
    else:
        for name, spec in sorted(tools.items()):
            if not isinstance(spec, dict):
                errors.append(f"configs/ops/pins.json `tools.{name}` must be an object")
                continue
            version = str(spec.get("version", "")).strip().lower()
            if not version:
                errors.append(f"configs/ops/pins.json `tools.{name}` missing version")
                continue
            if version in {"latest", "main", "master"}:
                errors.append(f"configs/ops/pins.json `tools.{name}` uses floating version `{version}`")
    images = payload.get("images", {})
    if not isinstance(images, dict):
        errors.append("configs/ops/pins.json `images` must be an object")
    else:
        for name, spec in sorted(images.items()):
            if not isinstance(spec, dict):
                errors.append(f"configs/ops/pins.json `images.{name}` must be an object")
                continue
            ref = str(spec.get("ref", "")).strip().lower()
            if not ref:
                errors.append(f"configs/ops/pins.json `images.{name}` missing ref")
                continue
            if ref.endswith(":latest") or ref in {"latest", "main", "master"}:
                errors.append(f"configs/ops/pins.json `images.{name}` uses floating ref `{ref}`")
    helm = payload.get("helm", {})
    if not isinstance(helm, dict):
        errors.append("configs/ops/pins.json `helm` must be an object")
    else:
        deps = helm.get("chart_dependencies", [])
        if deps is None:
            deps = []
        if not isinstance(deps, list):
            errors.append("configs/ops/pins.json `helm.chart_dependencies` must be a list")
        else:
            for idx, dep in enumerate(deps):
                if not isinstance(dep, dict):
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` must be an object")
                    continue
                version = str(dep.get("version", "")).strip().lower()
                if not version:
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` missing version")
                    continue
                if version in {"latest", "main", "master"}:
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` uses floating version `{version}`")
    return (0 if not errors else 1), errors


def check_ops_stack_versions_report_valid(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    src = repo_root / "configs/ops/tool-versions.json"
    out = repo_root / "ops/stack/versions.json"
    if not src.exists() or not out.exists():
        return 1, ["missing configs/ops/tool-versions.json or ops/stack/versions.json"]
    src_payload = json.loads(src.read_text(encoding="utf-8"))
    out_payload = json.loads(out.read_text(encoding="utf-8"))
    src_tools = src_payload.get("tools", {}) if isinstance(src_payload, dict) else {}
    out_tools = out_payload.get("tools", {}) if isinstance(out_payload, dict) else {}
    if src_tools != out_tools:
        errors.append("ops/stack/versions.json must match configs/ops/tool-versions.json tools map")
    return (0 if not errors else 1), errors


def check_ops_generated_not_tracked_unless_allowed(repo_root: Path) -> tuple[int, list[str]]:
    # Alias-level policy check for generated outputs.
    return check_ops_generated_tracked(repo_root)


def check_ops_scripts_count_nonincreasing(repo_root: Path) -> tuple[int, list[str]]:
    baseline_path = repo_root / "configs/policy/ops-scripts-count-baseline.json"
    if not baseline_path.exists():
        return 1, [f"missing baseline file: {baseline_path.relative_to(repo_root).as_posix()}"]
    payload = json.loads(baseline_path.read_text(encoding="utf-8"))
    baseline = int(payload.get("max_count", 0))
    patterns = ("*.sh", "*.py")
    roots = [repo_root / "ops" / "run", repo_root / "ops" / "k8s", repo_root / "ops" / "obs", repo_root / "ops" / "load", repo_root / "ops" / "datasets", repo_root / "ops" / "stack", repo_root / "ops" / "e2e"]
    count = 0
    for base in roots:
        if not base.exists():
            continue
        for pat in patterns:
            count += len([p for p in base.rglob(pat) if p.is_file()])
    if baseline <= 0:
        return 1, ["configs/policy/ops-scripts-count-baseline.json must define max_count > 0"]
    if count > baseline:
        return 1, [f"ops script migration gate regressed: count={count} baseline={baseline}"]
    return 0, []


def check_ops_run_only_allowlisted_scripts(repo_root: Path) -> tuple[int, list[str]]:
    run_dir = repo_root / "ops" / "run"
    if not run_dir.exists():
        return 1, ["missing ops/run directory"]
    allowlisted = {
        "CONTRACT.md",
        "INDEX.md",
        "OWNER.md",
        "README.md",
    }
    approvals, approval_errors = _load_ops_run_temp_approvals(repo_root)
    approved_scripts = {
        str(item.get("script", "")).removeprefix("ops/run/").strip()
        for item in approvals
        if isinstance(item, dict)
    }
    errors: list[str] = []
    errors.extend(approval_errors)
    for path in sorted(run_dir.rglob("*")):
        if path.is_dir():
            continue
        rel = path.relative_to(run_dir).as_posix()
        if rel in allowlisted:
            continue
        if rel in approved_scripts:
            continue
        if path.suffix not in {".sh", ".py"}:
            continue
        errors.append(f"ops/run contains non-allowlisted script: ops/run/{rel}")
    return (0 if not errors else 1), errors


def check_ops_run_non_executable_unless_allowlisted(repo_root: Path) -> tuple[int, list[str]]:
    run_dir = repo_root / "ops" / "run"
    if not run_dir.exists():
        return 1, ["missing ops/run directory"]
    approvals, approval_errors = _load_ops_run_temp_approvals(repo_root)
    executable_allowlist = {
        str(item.get("script", "")).removeprefix("ops/run/").strip()
        for item in approvals
        if isinstance(item, dict)
    }
    errors: list[str] = []
    errors.extend(approval_errors)
    for path in sorted(run_dir.rglob("*")):
        if not path.is_file() or path.suffix not in {".sh", ".py"}:
            continue
        rel = path.relative_to(run_dir).as_posix()
        mode = path.stat().st_mode
        is_exec = bool(mode & 0o111)
        if is_exec and rel not in executable_allowlist:
            errors.append(f"ops/run/{rel}: executable bit forbidden (allowlist only)")
    return (0 if not errors else 1), errors


def check_ops_run_temp_script_approvals(repo_root: Path) -> tuple[int, list[str]]:
    approvals, errors = _load_ops_run_temp_approvals(repo_root)
    run_dir = repo_root / "ops" / "run"
    for item in approvals:
        script = str(item.get("script", "")).strip()
        if not script:
            continue
        if not (repo_root / script).exists():
            errors.append(f"{_OPS_RUN_TEMP_APPROVALS.as_posix()}: approved script missing on disk (`{script}`)")
    return (0 if not errors else 1), sorted(errors)


def check_ops_no_new_run_scripts_without_approval_and_expiry(repo_root: Path) -> tuple[int, list[str]]:
    errs: list[str] = []
    for fn in (check_ops_run_only_allowlisted_scripts, check_ops_run_temp_script_approvals):
        code, rows = fn(repo_root)
        if code != 0:
            errs.extend(rows)
    return (0 if not errs else 1), sorted(set(errs))


def check_ops_obs_drift_goldens(repo_root: Path) -> tuple[int, list[str]]:
    scripts = [
        ["python3", "ops/obs/scripts/contracts/check_profile_goldens.py"],
        ["python3", "ops/obs/scripts/contracts/check_metrics_golden.py"],
        ["python3", "ops/obs/scripts/contracts/check_trace_golden.py"],
    ]
    errors: list[str] = []
    for cmd in scripts:
        proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
        if proc.returncode != 0:
            msg = ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip().splitlines()
            first = msg[0] if msg else "golden drift check failed"
            errors.append(f"{' '.join(cmd)} => {first}")
    return (0 if not errors else 1), errors


def check_ops_chart_version_contract(repo_root: Path) -> tuple[int, list[str]]:
    chart = repo_root / "ops/k8s/charts/bijux-atlas/Chart.yaml"
    if not chart.exists():
        return 1, ["missing ops/k8s/charts/bijux-atlas/Chart.yaml"]
    text = chart.read_text(encoding="utf-8", errors="ignore")
    version = ""
    app_version = ""
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith("version:"):
            version = line.split(":", 1)[1].strip().strip('"').strip("'")
        if line.startswith("appVersion:"):
            app_version = line.split(":", 1)[1].strip().strip('"').strip("'")
    errors: list[str] = []
    semver_re = re.compile(r"^\d+\.\d+\.\d+(?:[-+][A-Za-z0-9.-]+)?$")
    if not version:
        errors.append("Chart.yaml missing version")
    elif not semver_re.match(version):
        errors.append(f"Chart.yaml version must be semver, got `{version}`")
    if not app_version:
        errors.append("Chart.yaml missing appVersion")
    elif not semver_re.match(app_version):
        errors.append(f"Chart.yaml appVersion must be semver, got `{app_version}`")
    if version and app_version and version != app_version:
        errors.append(f"Chart.yaml version/appVersion mismatch (`{version}` != `{app_version}`)")
    return (0 if not errors else 1), errors


def check_ops_clean_allowed_roots_only(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_commands.py"
    if not path.exists():
        return 1, ["missing ops runtime commands module"]
    text = path.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    if 'ctx.repo_root / "ops" / "_generated"' not in text:
        errors.append("ops clean must target only ops/_generated root")
    if "shutil.rmtree(child)" not in text:
        errors.append("ops clean implementation changed; expected child-only deletion under ops/_generated")
    forbidden_markers = ['ctx.repo_root / "artifacts"', 'ctx.repo_root / "ops" / "_generated_committed"', "repo_root / \"/\""]
    for marker in forbidden_markers:
        if marker in text:
            errors.append(f"ops clean must not target non-allowed root marker `{marker}`")
    return (0 if not errors else 1), errors


CHECKS: tuple[CheckDef, ...] = (
    CheckDef("ops.no_tracked_generated", "ops", "forbid tracked files in generated ops dirs", 800, check_ops_generated_tracked, fix_hint="Untrack generated ops files."),
    CheckDef("ops.generated_not_tracked_unless_allowed", "ops", "forbid tracked generated ops outputs unless explicitly allowlisted", 800, check_ops_generated_not_tracked_unless_allowed, fix_hint="Keep generated outputs untracked or move to committed generated roots."),
    CheckDef("ops.no_tracked_timestamps", "ops", "forbid tracked timestamped paths", 1000, check_tracked_timestamp_paths, fix_hint="Remove timestamped tracked paths."),
    CheckDef("ops.committed_generated_hygiene", "ops", "validate deterministic committed generated assets", 1000, check_committed_generated_hygiene, fix_hint="Regenerate committed outputs deterministically."),
    CheckDef("ops.manifests_schema", "ops", "validate ops manifests against atlas.ops.manifest.v1 schema", 1000, check_ops_manifests_schema, fix_hint="Fix ops/manifests/*.json|*.yaml to satisfy atlas.ops.manifest.v1."),
    CheckDef("ops.no_direct_script_entrypoints", "ops", "forbid direct ops script entrypoints in docs/workflows/makefiles", 1000, check_ops_no_direct_script_entrypoints, fix_hint="Use ./bin/atlasctl ops ... or make wrappers, not ops/**/*.sh paths."),
    CheckDef("ops.scripts_are_data_only", "ops", "enforce ops/manifests data-only file policy", 1000, check_ops_scripts_are_data_only, fix_hint="Keep ops/manifests to json/yaml data only."),
    CheckDef("ops.shell_policy", "ops", "enforce shell runtime guard requirements for ops/run wrappers", 1000, check_ops_shell_policy, fix_hint="Source common.sh and call ops_entrypoint_start + ops_version_guard in ops/run/*.sh."),
    CheckDef("ops.effect_boundary_imports", "ops", "forbid ops command layer imports from CLI parser modules", 1000, check_ops_effect_boundary_imports, fix_hint="Keep ops commands behind runtime/effects layers and avoid importing cli parser modules."),
    CheckDef("ops.env_schema_is_used", "ops", "ensure ops env schema is the single interface for env validate/print", 1000, check_ops_env_schema_is_used, fix_hint="Use ops env validate/print with configs/ops/env.schema.json as default."),
    CheckDef("ops.pins_update_is_deterministic", "ops", "require deterministic canonical ops pins payload", 1000, check_ops_pins_update_is_deterministic, fix_hint="Keep configs/ops/pins.json canonical and stable."),
    CheckDef("ops.pins_no_unpinned_versions", "ops", "forbid floating/unpinned versions in ops pins", 1000, check_ops_pins_no_unpinned_versions, fix_hint="Pin tools/images/helm versions and image digests."),
    CheckDef("ops.stack_versions_report_valid", "ops", "require stack versions report to match tool versions SSOT", 1000, check_ops_stack_versions_report_valid, fix_hint="Regenerate ops/stack/versions.json from configs/ops/tool-versions.json."),
    CheckDef("ops.scripts_count_nonincreasing", "ops", "enforce migration gate: ops scripts count must not increase", 1000, check_ops_scripts_count_nonincreasing, fix_hint="Reduce ops scripts count or intentionally update baseline in one reviewed change."),
    CheckDef("ops.run_only_allowlisted_scripts", "ops", "ops/run contains no scripts except explicitly allowed fixtures", 1000, check_ops_run_only_allowlisted_scripts, fix_hint="Migrate behavior into atlasctl and delete ops/run scripts or extend transitional allowlist intentionally."),
    CheckDef("ops.run_non_executable_unless_allowlisted", "ops", "ops/run scripts must be non-executable except allowlisted", 1000, check_ops_run_non_executable_unless_allowlisted, fix_hint="Remove executable bits from ops/run scripts after migration, or update allowlist intentionally."),
    CheckDef("ops.run_temp_script_approvals", "ops", "temporary ops/run migration script approvals must be valid and unexpired", 1000, check_ops_run_temp_script_approvals, fix_hint="Keep configs/policy/ops-run-temp-script-approvals.json entries valid, justified, and unexpired."),
    CheckDef("ops.no_new_run_scripts_without_approval_and_expiry", "ops", "forbid new ops/run scripts without valid temporary approval and expiry", 1000, check_ops_no_new_run_scripts_without_approval_and_expiry, fix_hint="Keep ops/run scripts allowlisted only via valid non-expired temp approvals, or migrate/delete them."),
    CheckDef("ops.obs_drift_goldens", "ops", "fail on observability golden drift (generated vs golden)", 1000, check_ops_obs_drift_goldens, fix_hint="Regenerate/fix observability goldens and contracts."),
    CheckDef("ops.chart_version_contract", "ops", "enforce chart/app version contract for bijux-atlas chart", 1000, check_ops_chart_version_contract, fix_hint="Keep Chart.yaml version and appVersion present, semver, and aligned."),
    CheckDef("ops.clean_allowed_roots_only", "ops", "ensure ops clean only targets allowed generated roots", 1000, check_ops_clean_allowed_roots_only, fix_hint="Restrict ops clean to ops/_generated child deletion only."),
)
