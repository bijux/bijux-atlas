from __future__ import annotations
import argparse
import hashlib
import json
import re
import shutil
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from typing import Callable
import yaml
from ..core.context import RunContext
from ..core.fs import ensure_evidence_path
from ..make.target_graph import parse_make_targets
@dataclass(frozen=True)
class DocsCheck:
    check_id: str
    description: str
    cmd: list[str] | None
    fn: Callable[[RunContext], tuple[int, str]] | None
    actionable: str
def _check(
    check_id: str,
    description: str,
    cmd: list[str] | None,
    actionable: str,
    fn: Callable[[RunContext], tuple[int, str]] | None = None,
) -> DocsCheck:
    return DocsCheck(check_id, description, cmd, fn, actionable)
DOCS_LINT_CHECKS: list[DocsCheck] = [
    _check(
        "docs-terminology-units",
        "Validate terminology and units SSOT usage",
        None,
        "Align terminology and units references with docs SSOT conventions.",
        fn=lambda ctx: _check_terminology_units_ssot(ctx),
    ),
    _check(
        "docs-status-lint",
        "Validate document status contract",
        None,
        "Fix missing/invalid status frontmatter values.",
        fn=lambda ctx: _lint_doc_status(ctx),
    ),
    _check(
        "docs-index-pages",
        "Validate index pages contract",
        None,
        "Ensure each docs directory has an index page where required.",
        fn=lambda ctx: _check_index_pages(ctx),
    ),
    _check(
        "docs-title-case",
        "Validate title case contract",
        None,
        "Normalize page titles to the required style.",
        fn=lambda ctx: _check_title_case(ctx),
    ),
    _check(
        "docs-no-orphans",
        "Validate no orphan docs",
        None,
        "Add nav links or remove orphaned docs pages.",
        fn=lambda ctx: _check_no_orphan_docs(ctx),
    ),
]
DOCS_GENERATE_COMMANDS: list[list[str]] = [
    ["python3", "-m", "atlasctl.cli", "docs", "generate-crates-map", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-architecture-map", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-k8s-values-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "concept-graph-generate", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-openapi-docs", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-observability-surface", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-badge", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-schema-docs", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-surface", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-contracts-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-make-targets-catalog", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-config-keys-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-env-vars-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "contracts-index", "--fix", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-chart-contract-index", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-k8s-install-matrix", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-make-targets-inventory", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-scripts-graph", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "ops", "k8s-surface-generate", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "runbook-map", "--fix", "--report", "text"],
]
def _run_check(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output.strip()
def _tracked_files(repo_root: Path, patterns: list[str] | None = None) -> list[str]:
    cmd = ["git", "ls-files"]
    if patterns:
        cmd.extend(patterns)
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]
def _load_public_targets(repo_root: Path) -> set[str]:
    payload = json.loads((repo_root / "configs/ops/public-surface.json").read_text(encoding="utf-8"))
    return set(payload.get("make_targets", []))
def _load_public_surface_exceptions(repo_root: Path) -> set[str]:
    path = repo_root / "configs/ops/public-surface-doc-exceptions.txt"
    if not path.exists():
        return set()
    return {line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip() and not line.startswith("#")}
def _check_public_surface_docs(ctx: RunContext) -> tuple[int, str]:
    public_targets = _load_public_targets(ctx.repo_root)
    exceptions = _load_public_surface_exceptions(ctx.repo_root)
    docs_roots = [ctx.repo_root / "docs" / "operations", ctx.repo_root / "docs" / "quickstart", ctx.repo_root / "docs" / "development"]
    make_re = re.compile(r"\bmake\s+([a-zA-Z0-9_.-]+)")
    ops_script_re = re.compile(r"\./(ops/[^\s`]+(?:\.sh|\.py))")
    errs: list[str] = []
    for base in docs_roots:
        if not base.exists():
            continue
        for md in base.rglob("*.md"):
            text = md.read_text(encoding="utf-8", errors="ignore")
            rel = md.relative_to(ctx.repo_root).as_posix()
            for target in make_re.findall(text):
                if target == "ops-":
                    continue
                if not (target.startswith("ops-") or target in {"root", "root-local", "gates", "explain", "help"}):
                    continue
                key = f"{rel}::make {target}"
                if target not in public_targets and key not in exceptions:
                    errs.append(f"{rel}: non-public make target referenced: {target}")
            for script in ops_script_re.findall(text):
                key = f"{rel}::./{script}"
                if script.startswith("ops/run/"):
                    continue
                if key not in exceptions:
                    errs.append(f"{rel}: non-public ops script referenced: ./{script}")
    return (0, "") if not errs else (1, "\n".join(errs))
def _check_docs_make_only(ctx: RunContext) -> tuple[int, str]:
    docs = sorted((ctx.repo_root / "docs" / "operations").rglob("*.md"))
    patterns = [
        re.compile(r"(^|\s)(\./)?scripts/[\w./-]+"),
        re.compile(r"(^|\s)(\./)?ops/.+/scripts/[\w./-]+"),
    ]
    violations: list[str] = []
    for doc in docs:
        rel = doc.relative_to(ctx.repo_root).as_posix()
        for idx, line in enumerate(doc.read_text(encoding="utf-8").splitlines(), start=1):
            if line.strip().startswith("#"):
                continue
            if "`" not in line and "scripts/" not in line:
                continue
            if any(pat.search(line) for pat in patterns):
                violations.append(f"{rel}:{idx}: direct script path in docs; reference `make <target>` instead")
    return (0, "") if not violations else (1, "\n".join(violations))
def _snapshot_hashes(path: Path) -> dict[str, str]:
    if path.is_file():
        return {str(path): hashlib.sha256(path.read_bytes()).hexdigest()}
    if path.is_dir():
        out: dict[str, str] = {}
        for child in sorted(p for p in path.rglob("*") if p.is_file()):
            out[str(child)] = hashlib.sha256(child.read_bytes()).hexdigest()
        return out
    return {}
def _check_docs_freeze_drift(ctx: RunContext) -> tuple[int, str]:
    targets = [
        ctx.repo_root / "docs" / "_generated" / "contracts",
        ctx.repo_root / "docs" / "_generated" / "contracts" / "chart-contract-index.md",
        ctx.repo_root / "docs" / "_generated" / "openapi",
        ctx.repo_root / "docs" / "contracts" / "errors.md",
        ctx.repo_root / "docs" / "contracts" / "metrics.md",
        ctx.repo_root / "docs" / "contracts" / "tracing.md",
        ctx.repo_root / "docs" / "contracts" / "endpoints.md",
        ctx.repo_root / "docs" / "contracts" / "config-keys.md",
        ctx.repo_root / "docs" / "contracts" / "chart-values.md",
    ]
    before: dict[str, str] = {}
    for target in targets:
        before.update(_snapshot_hashes(target))
    cmds = [
        ["python3", "-m", "atlasctl.cli", "contracts", "generate", "--generators", "artifacts"],
        ["python3", "-m", "atlasctl.cli", "docs", "generate-chart-contract-index", "--report", "text"],
    ]
    for cmd in cmds:
        code, output = _run_check(cmd, ctx.repo_root)
        if code != 0:
            return 1, output
    after: dict[str, str] = {}
    for target in targets:
        after.update(_snapshot_hashes(target))
    if before == after:
        return 0, ""
    changed = sorted({*before.keys(), *after.keys()})
    drift = [str(Path(path).relative_to(ctx.repo_root)) for path in changed if before.get(path) != after.get(path)]
    return 1, "\n".join(drift)
def _resolve_openapi_schema(schema: dict[str, object], schemas: dict[str, dict[str, object]]) -> dict[str, object]:
    ref = schema.get("$ref")
    if not isinstance(ref, str):
        return schema
    name = ref.split("/")[-1]
    target = schemas.get(name, {})
    return _resolve_openapi_schema(target, schemas)
def _validate_openapi_example(
    value: object,
    schema: dict[str, object],
    schemas: dict[str, dict[str, object]],
    path: str,
) -> list[str]:
    resolved = _resolve_openapi_schema(schema, schemas)
    errs: list[str] = []
    typ = resolved.get("type")
    if typ == "object":
        if not isinstance(value, dict):
            return [f"{path}: expected object"]
        required = resolved.get("required", [])
        if isinstance(required, list):
            for req in required:
                if isinstance(req, str) and req not in value:
                    errs.append(f"{path}: missing required field `{req}`")
        props = resolved.get("properties", {})
        if isinstance(props, dict):
            for key, item in value.items():
                prop_schema = props.get(key)
                if isinstance(prop_schema, dict):
                    errs.extend(_validate_openapi_example(item, prop_schema, schemas, f"{path}.{key}"))
    elif typ == "array":
        if not isinstance(value, list):
            return [f"{path}: expected array"]
        items = resolved.get("items", {})
        if isinstance(items, dict):
            for idx, item in enumerate(value):
                errs.extend(_validate_openapi_example(item, items, schemas, f"{path}[{idx}]"))
    elif typ == "string" and not isinstance(value, str):
        errs.append(f"{path}: expected string")
    elif typ == "integer" and not isinstance(value, int):
        errs.append(f"{path}: expected integer")
    elif typ == "number" and not isinstance(value, (int, float)):
        errs.append(f"{path}: expected number")
    elif typ == "boolean" and not isinstance(value, bool):
        errs.append(f"{path}: expected boolean")
    return errs
def _check_openapi_examples(ctx: RunContext) -> tuple[int, str]:
    openapi_path = ctx.repo_root / "configs" / "openapi" / "v1" / "openapi.generated.json"
    openapi = json.loads(openapi_path.read_text(encoding="utf-8"))
    schemas_obj = openapi.get("components", {}).get("schemas", {})
    schemas: dict[str, dict[str, object]] = {
        k: v for k, v in schemas_obj.items() if isinstance(k, str) and isinstance(v, dict)
    }
    errors: list[str] = []
    paths = openapi.get("paths", {})
    if isinstance(paths, dict):
        for route, methods in paths.items():
            if not isinstance(methods, dict):
                continue
            for method, op in methods.items():
                if not isinstance(op, dict):
                    continue
                responses = op.get("responses", {})
                if not isinstance(responses, dict):
                    continue
                for status, resp in responses.items():
                    if not isinstance(resp, dict):
                        continue
                    content = resp.get("content", {})
                    if not isinstance(content, dict):
                        continue
                    for media, media_obj in content.items():
                        if not isinstance(media_obj, dict):
                            continue
                        schema = media_obj.get("schema")
                        if not isinstance(schema, dict):
                            continue
                        example = media_obj.get("example")
                        if example is not None:
                            errors.extend(
                                _validate_openapi_example(example, schema, schemas, f"{method} {route} {status} {media}")
                            )
                        examples = media_obj.get("examples", {})
                        if isinstance(examples, dict):
                            for ex_name, ex in examples.items():
                                if not isinstance(ex, dict) or "value" not in ex:
                                    continue
                                errors.extend(
                                    _validate_openapi_example(
                                        ex["value"],
                                        schema,
                                        schemas,
                                        f"{method} {route} {status} {media} examples.{ex_name}",
                                    )
                                )
    return (0, "") if not errors else (1, "\n".join(errors[:200]))
