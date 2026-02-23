from __future__ import annotations

from pathlib import Path

_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_CONTROL_PLANE_GROUPS = {"docs", "configs", "dev", "ops", "policies", "internal"}
_TOP_LEVEL_GROUP_MAP = {
    "app": "dev",
    "capability": "dev",
    "checks": "policies",
    "cli": "dev",
    "commands": "dev",
    "contracts": "dev",
    "core": "dev",
    "engine": "dev",
    "execution": "dev",
    "internal": "internal",
    "ops": "ops",
    "policies": "policies",
    "registry": "dev",
    "reporting": "dev",
    "runtime": "dev",
    "suite": "dev",
}
# Ops k8s test adapters now live under commands/ops/k8s/tests/checks/<group>/.
# This yields depth 6 and is intentional after shell-wrapper retirement.
_MAX_PACKAGE_DEPTH = 6
_CANONICAL_CONCEPT_HOME = {
    "registry": "registry",
    "runner": "engine",
    "contracts": "contracts",
    "output": "reporting",
}
_FORBIDDEN_CONCEPT_ALIASES = {
    "registry": {"registries"},
    "runner": {"runner", "runners"},
    "contracts": {"contract"},
    "output": {"output", "outputs", "reports"},
}
_ATLASCTL_PACKAGE_ROOT_ALLOWED = {
    "ARCHITECTURE.md",
    "LICENSE",
    "README.md",
    "docs",
    "pyproject.toml",
    "src",
    "tests",
    "requirements.in",
    "requirements.lock.txt",
    "uv.lock",
}
_CHECK_DOMAIN_PATHS = {
    "repo_shape": Path("packages/atlasctl/src/atlasctl/checks/domains/dev/repo_shape"),
    "makefiles": Path("packages/atlasctl/src/atlasctl/checks/domains/policies/make"),
    "ops": Path("packages/atlasctl/src/atlasctl/checks/domains/ops/ops_checks"),
    "docs": Path("packages/atlasctl/src/atlasctl/checks/domains/docs"),
    "observability": Path("packages/atlasctl/src/atlasctl/checks/domains/ops/observability"),
    "artifacts": Path("packages/atlasctl/src/atlasctl/checks/layout/domains/artifacts"),
}
_CHECKS_ROOT_REL = Path("packages/atlasctl/src/atlasctl/checks")
_CHECKS_ROOT_ALLOWED_FILES = {"README.md", "REGISTRY.toml", "REGISTRY.generated.json", "__init__.py"}
_CHECKS_DOMAINS_REL = _CHECKS_ROOT_REL / "domains"
_CHECKS_DOMAIN_MODULE_CAP = 10
_CHECKS_ROOT_DIR_CAP = 10
MAX_MODULE_LOC = 1000


def _iter_top_level_dirs(repo_root: Path) -> list[str]:
    root = repo_root / _SRC_ROOT
    if not root.exists():
        return []
    items: list[str] = []
    for path in sorted(root.iterdir()):
        if path.name == "__pycache__" or not path.is_dir():
            continue
        items.append(path.name)
    return items


def check_no_nested_same_name_packages(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*")):
        if not path.is_dir():
            continue
        if "__pycache__" in path.parts:
            continue
        parts = path.relative_to(src_root).parts
        for left, right in zip(parts, parts[1:]):
            if left == right:
                offenders.append(path.relative_to(repo_root).as_posix())
                break
    if offenders:
        return 1, [f"nested same-name package segment is forbidden: {item}" for item in offenders]
    return 0, []


def check_layout_domain_readmes(repo_root: Path) -> tuple[int, list[str]]:
    layout_root = repo_root / "packages/atlasctl/src/atlasctl/checks/layout"
    required_domains = (
        "root",
        "makefiles",
        "ops",
        "scripts",
        "docs",
        "workflows",
        "domains/artifacts",
        "domains/contracts",
        "domains/governance",
        "domains/public_surface",
        "domains/hygiene",
        "domains/policies",
        "domains/orphans",
        "domains/scenarios",
        "domains/shell",
    )
    missing: list[str] = []
    for domain in required_domains:
        readme = layout_root / domain / "README.md"
        if not readme.exists():
            missing.append(readme.relative_to(repo_root).as_posix())
    if missing:
        return 1, [f"missing layout domain README: {path}" for path in missing]
    return 0, []


def check_layout_no_legacy_imports(repo_root: Path) -> tuple[int, list[str]]:
    layout_root = repo_root / "packages/atlasctl/src/atlasctl/checks/layout"
    legacy_ns = "atlasctl." + "legacy"
    offenders: list[str] = []
    for path in sorted(layout_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if legacy_ns in text or "from ...legacy" in text or "from ....legacy" in text:
            offenders.append(rel)
    if offenders:
        return 1, [f"layout checks must not import {legacy_ns}: {path}" for path in offenders]
    return 0, []


def check_top_level_package_group_mapping(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for name in _iter_top_level_dirs(repo_root):
        group = _TOP_LEVEL_GROUP_MAP.get(name)
        if group is None:
            offenders.append(
                f"top-level package '{name}' has no control-plane group mapping; "
                "map it to one of configs/dev/ops/policies/internal",
            )
            continue
        if group not in _CONTROL_PLANE_GROUPS:
            offenders.append(f"top-level package '{name}' maps to invalid control-plane group '{group}'")
    if offenders:
        return 1, offenders
    return 0, []


def check_package_max_depth(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / _SRC_ROOT
    offenders: list[str] = []
    for path in sorted(root.rglob("*")):
        if not path.is_dir() or "__pycache__" in path.parts:
            continue
        depth = len(path.relative_to(root).parts)
        if depth > _MAX_PACKAGE_DEPTH:
            offenders.append(
                f"{path.relative_to(repo_root).as_posix()}: depth {depth} > {_MAX_PACKAGE_DEPTH}",
            )
    if offenders:
        return 1, offenders
    return 0, []


def check_canonical_concept_homes(repo_root: Path) -> tuple[int, list[str]]:
    top_level = set(_iter_top_level_dirs(repo_root))
    offenders: list[str] = []
    for concept, canonical in _CANONICAL_CONCEPT_HOME.items():
        if canonical not in top_level:
            offenders.append(f"canonical {concept} package missing: {_SRC_ROOT.as_posix()}/{canonical}")
    for concept, aliases in _FORBIDDEN_CONCEPT_ALIASES.items():
        canonical = _CANONICAL_CONCEPT_HOME[concept]
        for alias in sorted(aliases):
            if alias in top_level:
                offenders.append(f"duplicate {concept} concept package '{alias}' is forbidden; use '{canonical}'")
    if offenders:
        return 1, offenders
    return 0, []


def check_atlasctl_package_root_shape(repo_root: Path) -> tuple[int, list[str]]:
    package_root = repo_root / "packages/atlasctl"
    offenders: list[str] = []
    if not package_root.exists():
        return 1, ["missing package root: packages/atlasctl"]
    for child in sorted(package_root.iterdir(), key=lambda p: p.name):
        name = child.name
        if name.startswith("."):
            continue
        if name not in _ATLASCTL_PACKAGE_ROOT_ALLOWED:
            offenders.append(
                f"packages/atlasctl/{name}: not allowed in package root "
                "(allowed: LICENSE, README.md, docs/, pyproject.toml, src/, tests/, requirements.in, requirements.lock.txt)",
            )
    if offenders:
        return 1, offenders
    return 0, []


def check_checks_domain_split(repo_root: Path) -> tuple[int, list[str]]:
    missing: list[str] = []
    for name, rel_path in sorted(_CHECK_DOMAIN_PATHS.items()):
        if not (repo_root / rel_path).exists():
            missing.append(f"{name}: {rel_path.as_posix()}")
    if missing:
        return 1, [f"missing canonical check domain path: {item}" for item in missing]
    return 0, []


def check_checks_root_contract(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / _CHECKS_ROOT_REL
    if not checks_root.exists():
        return 1, [f"missing checks root: {_CHECKS_ROOT_REL.as_posix()}"]
    errors: list[str] = []
    py_files = sorted(p.name for p in checks_root.glob("*.py"))
    if py_files != ["__init__.py"]:
        errors.append(
            "checks root python files must be only __init__.py; found: " + ", ".join(py_files or ["<none>"])
        )
    for path in sorted(checks_root.iterdir(), key=lambda p: p.name):
        if path.name == "__pycache__":
            continue
        if path.is_file() and path.name not in _CHECKS_ROOT_ALLOWED_FILES:
            errors.append(f"unexpected root file in checks/: {path.name}")
    root_dirs = sorted(
        p.name
        for p in checks_root.iterdir()
        if p.is_dir() and p.name != "__pycache__"
    )
    if len(root_dirs) > _CHECKS_ROOT_DIR_CAP:
        errors.append(
            f"checks root directory cap exceeded: {len(root_dirs)} > {_CHECKS_ROOT_DIR_CAP}; "
            "regroup domain packages under checks/domains/",
        )
    checks_domains = repo_root / _CHECKS_DOMAINS_REL
    if not checks_domains.exists():
        errors.append(f"missing grouped checks domains root: {_CHECKS_DOMAINS_REL.as_posix()}")
        return (0 if not errors else 1), errors
    grouped_domains = sorted(
        p.name
        for p in checks_domains.iterdir()
        if p.is_dir() and p.name != "__pycache__"
    )
    if len(grouped_domains) > _CHECKS_DOMAIN_MODULE_CAP:
        errors.append(
            f"checks grouped-domain cap exceeded: {len(grouped_domains)} > {_CHECKS_DOMAIN_MODULE_CAP}; "
            f"reduce folders under {_CHECKS_DOMAINS_REL.as_posix()}",
        )
    return (0 if not errors else 1), errors


def check_module_size(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for py in sorted(root.rglob("*.py")):
        rel = py.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        loc = sum(1 for _ in py.open("r", encoding="utf-8"))
        if loc > MAX_MODULE_LOC:
            offenders.append(f"{rel}: {loc} LOC > {MAX_MODULE_LOC}")
    if offenders:
        return 1, offenders
    return 0, []
