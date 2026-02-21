from __future__ import annotations

from pathlib import Path


_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_WRITE_ALLOW_SEGMENTS = (
    "/core/fs.py",
    "/core/context.py",
    "/contracts/generators.py",
    "/checks/layout/ops/generation/",
    "/checks/layout/makefiles/tools/",
    "/checks/layout/artifacts/",
    "/checks/layout/",
    "/checks/docs/",
    "/commands/docs/runtime_chunks/",
    "/checks/ops/",
    "/reporting/",
    "/policies/",
    "/suite/",
    "/gen/",
    "/commands/doctor.py",
    "/commands/compat.py",
    "/commands/check/command.py",
    "/commands/ops/ops_k8s.py",
    "/commands/ops/runtime_modules/",
    "/legacy/",
    "/tests/",
)
_ENV_ALLOW_SEGMENTS = (
    "/core/env.py",
    "/core/runtime/env_guard.py",
    "/cli/main.py",
    "/policies/",
    "/suite/",
    "/commands/",
    "/legacy/",
    "/tests/",
)
_SUBPROCESS_ALLOW_SEGMENTS = (
    "/core/exec.py",
    "/core/process.py",
    "/checks/layout/",
    "/checks/docs/",
    "/commands/docs/runtime_chunks/",
    "/checks/ops/",
    "/policies/",
    "/suite/",
    "/gen/",
    "/deps/",
    "/configs/",
    "/make/",
    "/migrate/",
    "/env/",
    "/docker/",
    "/python_tools/",
    "/test_tools/",
    "/commands/compat.py",
    "/commands/check/command.py",
    "/commands/doctor.py",
    "/legacy/",
    "/tests/",
)


def _iter_modern_py(repo_root: Path) -> list[Path]:
    root = repo_root / _SRC_ROOT
    return sorted(path for path in root.rglob("*.py") if "/legacy/" not in path.as_posix())


def _is_allowed(rel: str, allow_segments: tuple[str, ...]) -> bool:
    return any(segment in rel for segment in allow_segments)


def check_forbidden_effect_calls(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in _iter_modern_py(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if "/commands/" not in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")

        if ("subprocess.run(" in text or "subprocess.Popen(" in text or "subprocess.check_output(" in text) and not _is_allowed(rel, _SUBPROCESS_ALLOW_SEGMENTS):
            errors.append(f"{rel}: forbidden subprocess call outside core.exec/core.process")
        if "os.environ[" in text and not _is_allowed(rel, _ENV_ALLOW_SEGMENTS):
            errors.append(f"{rel}: forbidden direct os.environ access outside core.env wrappers")
        if ".write_text(" in text and not _is_allowed(rel, _WRITE_ALLOW_SEGMENTS):
            errors.append(f"{rel}: forbidden direct Path.write_text outside fs/generator/reporting layers")
    return (0 if not errors else 1), sorted(set(errors))


def check_subprocess_boundary(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in _iter_modern_py(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if "/commands/" not in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if ("import subprocess" in text or "from subprocess import " in text) and not _is_allowed(rel, _SUBPROCESS_ALLOW_SEGMENTS):
            errors.append(f"{rel}: subprocess import is restricted to core.exec/core.process")
    return (0 if not errors else 1), sorted(set(errors))
