from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
import re
from typing import Callable, Mapping, Protocol, runtime_checkable

from .effects import CheckEffect, normalize_effect


_CHECK_ID_PATTERN = re.compile(r"^checks_[a-z0-9]+(?:_[a-z0-9]+)+$")
_SEGMENT_PATTERN = re.compile(r"^[a-z][a-z0-9_]*$")
_RESULT_CODE_PATTERN = re.compile(r"^[A-Z][A-Z0-9_]*$")
_DOMAIN_VOCAB = frozenset({"checks", "configs", "contracts", "docker", "docs", "license", "make", "ops", "policies", "python", "repo"})


@dataclass(frozen=True, order=True)
class CheckId:
    value: str

    def __post_init__(self) -> None:
        object.__setattr__(self, "value", str(self.value).strip())

    @classmethod
    def parse(cls, value: str, *, domain: str | None = None) -> "CheckId":
        raw = str(value).strip()
        if not _CHECK_ID_PATTERN.fullmatch(raw):
            raise ValueError(f"invalid canonical check id `{raw}`: expected checks_<domain>_<name> snake_case")
        if domain:
            parsed_domain = raw.split("_", 2)[1]
            if parsed_domain != str(domain).strip():
                raise ValueError(f"invalid canonical check id `{raw}`: domain segment `{parsed_domain}` must match `{domain}`")
        return cls(raw)

    @classmethod
    def coerce(cls, value: str, *, domain: str | None = None, allow_legacy: bool = True) -> "CheckId":
        raw = str(value).strip()
        if allow_legacy and raw and not raw.startswith("checks_"):
            return cls(raw)
        return cls.parse(raw, domain=domain)

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True, order=True)
class DomainId:
    value: str

    def __post_init__(self) -> None:
        value = str(self.value).strip()
        if value not in _DOMAIN_VOCAB:
            raise ValueError(f"invalid domain `{value}`: must be one of {sorted(_DOMAIN_VOCAB)}")
        object.__setattr__(self, "value", value)

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True, order=True)
class OwnerId:
    value: str

    def __post_init__(self) -> None:
        value = str(self.value).strip()
        if not _SEGMENT_PATTERN.fullmatch(value):
            raise ValueError(f"invalid owner id `{value}`: expected lowercase snake_case")
        object.__setattr__(self, "value", value)

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True, order=True)
class Tag:
    value: str

    def __post_init__(self) -> None:
        value = str(self.value).strip()
        if not value:
            raise ValueError("tag cannot be empty")
        object.__setattr__(self, "value", value)

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True, order=True)
class ResultCode:
    value: str

    def __post_init__(self) -> None:
        value = str(self.value).strip()
        if not _RESULT_CODE_PATTERN.fullmatch(value):
            raise ValueError(f"invalid result_code `{value}`: expected UPPER_SNAKE_CASE")
        object.__setattr__(self, "value", value)

    def __str__(self) -> str:
        return self.value


class Severity(str, Enum):
    ERROR = "error"
    WARN = "warn"
    INFO = "info"


class Effect(str, Enum):
    FS_READ = CheckEffect.FS_READ.value
    FS_WRITE = CheckEffect.FS_WRITE.value
    SUBPROCESS = CheckEffect.SUBPROCESS.value
    NETWORK = CheckEffect.NETWORK.value


class CheckStatus(str, Enum):
    PASS = "pass"
    FAIL = "fail"
    SKIP = "skip"
    ERROR = "error"


class CheckCategory(str, Enum):
    LINT = "lint"
    CHECK = "check"
    POLICY = "check"
    HYGIENE = "check"
    CONTRACT = "check"
    DRIFT = "check"
    SECURITY = "check"


@dataclass(frozen=True)
class Violation:
    code: ResultCode | str
    message: str
    hint: str = ""
    path: str = ""
    line: int = 0
    column: int = 0
    severity: Severity = Severity.ERROR

    def __post_init__(self) -> None:
        object.__setattr__(self, "code", str(self.code).strip() or "CHECK_GENERIC")
        object.__setattr__(self, "message", str(self.message).strip())
        object.__setattr__(self, "hint", str(self.hint).strip())
        object.__setattr__(self, "path", str(self.path).strip())
        object.__setattr__(self, "line", int(self.line or 0))
        object.__setattr__(self, "column", int(self.column or 0))

    @property
    def canonical_key(self) -> tuple[str, str, str, int, int]:
        return (str(self.path), str(self.code), self.message, self.line, self.column)


@dataclass(frozen=True)
class CheckOutcome:
    violations: tuple[Violation, ...] = ()
    warnings: tuple[str, ...] = ()
    metrics: Mapping[str, object] = field(default_factory=dict)
    evidence_paths: tuple[str, ...] = ()

    @property
    def canonical_key(self) -> tuple[tuple[str, str, str, int, int], ...]:
        return tuple(sorted((v.canonical_key for v in self.violations)))


@runtime_checkable
class RepoFS(Protocol):
    def read_text(self, path: Path, *, encoding: str = "utf-8") -> str: ...


@runtime_checkable
class GitView(Protocol):
    def ls_files(self, *pathspecs: str) -> list[str]: ...


@dataclass(frozen=True)
class CheckContext:
    repo_root: Path
    fs: RepoFS
    git: GitView | None = None
    clock: object | None = None
    env: Mapping[str, str] = field(default_factory=dict)


class CheckFn(Protocol):
    def __call__(self, ctx: CheckContext) -> list[Violation] | CheckOutcome: ...


class Check(Protocol):
    id: str
    title: str
    domain: str

    def run(self, repo_root: Path) -> CheckResult: ...


@dataclass(frozen=True)
class CheckSelector:
    ids: tuple[CheckId, ...] = ()
    domains: tuple[DomainId, ...] = ()
    tags: tuple[Tag, ...] = ()
    patterns: tuple[str, ...] = ()

    @property
    def canonical_key(self) -> tuple[tuple[str, ...], tuple[str, ...], tuple[str, ...], tuple[str, ...]]:
        return (
            tuple(sorted(str(x) for x in self.ids)),
            tuple(sorted(str(x) for x in self.domains)),
            tuple(sorted(str(x) for x in self.tags)),
            tuple(sorted(self.patterns)),
        )


CheckFunc = Callable[[Path], tuple[int, list[str]]]
CheckFnLegacy = CheckFunc


@dataclass(frozen=True)
class CheckDef:
    check_id: CheckId | str
    domain: DomainId | str
    description: str
    budget_ms: int
    fn: callable
    canonical_id: CheckId | str | None = None
    legacy_check_id: CheckId | str | None = None
    severity: Severity = Severity.ERROR
    category: CheckCategory = CheckCategory.CHECK
    fix_hint: str = "Review check output and apply the documented fix."
    intent: str = ""
    remediation_short: str = "Review check output and apply the documented fix."
    remediation_link: str = "packages/atlasctl/docs/checks/check-id-migration-rules.md"
    slow: bool = False
    tags: tuple[Tag | str, ...] = ()
    effects: tuple[Effect | str, ...] = (Effect.FS_READ.value,)
    owners: tuple[OwnerId | str, ...] = ()
    external_tools: tuple[str, ...] = ()
    evidence: tuple[str, ...] = ()
    writes_allowed_roots: tuple[str, ...] = ("artifacts/evidence/",)
    result_code: ResultCode | str = "CHECK_GENERIC"

    def __post_init__(self) -> None:
        did = self.domain if isinstance(self.domain, DomainId) else DomainId(str(self.domain))
        cid = self.check_id if isinstance(self.check_id, CheckId) else CheckId.coerce(str(self.check_id), domain=str(did))
        canonical = self.canonical_id
        canonical_id = cid if canonical in (None, "") else (canonical if isinstance(canonical, CheckId) else CheckId.parse(str(canonical), domain=str(did)))
        object.__setattr__(self, "check_id", str(cid))
        object.__setattr__(self, "domain", str(did))
        object.__setattr__(self, "canonical_id", str(canonical_id))
        object.__setattr__(self, "legacy_check_id", None if self.legacy_check_id in (None, "") else str(CheckId.coerce(str(self.legacy_check_id))))
        object.__setattr__(self, "description", str(self.description).strip())
        object.__setattr__(self, "budget_ms", int(self.budget_ms))
        object.__setattr__(self, "intent", str(self.intent).strip())
        object.__setattr__(self, "result_code", str(ResultCode(str(self.result_code or "CHECK_GENERIC"))))
        object.__setattr__(self, "tags", tuple(str(Tag(str(t))) for t in self.tags if str(t).strip()))
        object.__setattr__(self, "effects", tuple(normalize_effect(str(e)) for e in self.effects if str(e).strip()) or (Effect.FS_READ.value,))
        object.__setattr__(self, "owners", tuple(str(OwnerId(str(o))) for o in self.owners if str(o).strip()))

    @property
    def id(self) -> str:
        return str(self.canonical_id)

    @property
    def title(self) -> str:
        return self.description

    @property
    def canonical_key(self) -> tuple[str, str]:
        return (str(self.check_id), str(self.domain))


@dataclass(frozen=True)
class CheckResult:
    id: CheckId | str
    title: str
    domain: DomainId | str
    status: CheckStatus | str
    violations: tuple[Violation, ...] = ()
    warnings: tuple[str, ...] = ()
    evidence_paths: tuple[str, ...] = ()
    metrics: Mapping[str, object] = field(default_factory=dict)
    description: str = ""
    fix_hint: str = ""
    category: CheckCategory | str = CheckCategory.CHECK
    severity: Severity = Severity.ERROR
    tags: tuple[Tag | str, ...] = ()
    effects: tuple[Effect | str, ...] = (Effect.FS_READ.value,)
    effects_used: tuple[Effect | str, ...] = (Effect.FS_READ.value,)
    owners: tuple[OwnerId | str, ...] = ()
    writes_allowed_roots: tuple[str, ...] = ("artifacts/evidence/",)
    result_code: ResultCode | str = "CHECK_GENERIC"
    errors: tuple[str, ...] = ()

    def __post_init__(self) -> None:
        cid = self.id if isinstance(self.id, CheckId) else CheckId.coerce(str(self.id))
        did = self.domain if isinstance(self.domain, DomainId) else DomainId(str(self.domain))
        status = self.status if isinstance(self.status, CheckStatus) else CheckStatus(str(self.status).strip().lower())
        category = self.category if isinstance(self.category, CheckCategory) else CheckCategory(str(self.category).strip().lower())
        object.__setattr__(self, "id", str(cid))
        object.__setattr__(self, "domain", str(did))
        object.__setattr__(self, "status", status)
        object.__setattr__(self, "category", category)
        object.__setattr__(self, "result_code", str(ResultCode(str(self.result_code or "CHECK_GENERIC"))))
        object.__setattr__(self, "warnings", tuple(str(x).strip() for x in self.warnings if str(x).strip()))
        object.__setattr__(self, "evidence_paths", tuple(str(x).strip() for x in self.evidence_paths if str(x).strip()))
        object.__setattr__(self, "tags", tuple(str(Tag(str(t))) for t in self.tags if str(t).strip()))
        object.__setattr__(self, "effects", tuple(normalize_effect(str(e)) for e in self.effects if str(e).strip()))
        object.__setattr__(self, "effects_used", tuple(normalize_effect(str(e)) for e in self.effects_used if str(e).strip()))
        object.__setattr__(self, "owners", tuple(str(OwnerId(str(o))) for o in self.owners if str(o).strip()))

        legacy_errors = tuple(str(msg).strip() for msg in self.errors if str(msg).strip())
        if self.violations:
            normalized_violations = tuple(sorted(self.violations, key=lambda row: row.canonical_key))
        else:
            normalized_violations = tuple(
                Violation(code=str(self.result_code), message=msg, hint=self.fix_hint, severity=self.severity) for msg in legacy_errors
            )
        object.__setattr__(self, "violations", normalized_violations)
        if not self.errors:
            object.__setattr__(self, "errors", tuple(v.message for v in normalized_violations if v.severity != Severity.WARN))

    @property
    def canonical_key(self) -> tuple[str, str]:
        return (str(self.domain), str(self.id))


@dataclass(frozen=True)
class CheckRunReport:
    schema_version: int = 1
    kind: str = "check-run"
    status: str = "ok"
    rows: tuple[CheckResult, ...] = ()
    summary: Mapping[str, int] = field(default_factory=dict)
    timings: Mapping[str, int] = field(default_factory=dict)
    env: Mapping[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        ordered = tuple(sorted(self.rows, key=lambda row: row.canonical_key))
        object.__setattr__(self, "rows", ordered)
        if not self.summary:
            passed = sum(1 for row in ordered if row.status == CheckStatus.PASS)
            failed = sum(1 for row in ordered if row.status == CheckStatus.FAIL)
            skipped = sum(1 for row in ordered if row.status == CheckStatus.SKIP)
            errored = sum(1 for row in ordered if row.status == CheckStatus.ERROR)
            object.__setattr__(
                self,
                "summary",
                {
                    "passed": passed,
                    "failed": failed,
                    "skipped": skipped,
                    "errors": errored,
                    "total": len(ordered),
                },
            )


__all__ = [
    "CheckCategory",
    "Check",
    "CheckContext",
    "CheckDef",
    "CheckFn",
    "CheckFnLegacy",
    "CheckFunc",
    "CheckId",
    "CheckOutcome",
    "CheckResult",
    "CheckRunReport",
    "CheckSelector",
    "CheckStatus",
    "DomainId",
    "Effect",
    "OwnerId",
    "RepoFS",
    "GitView",
    "ResultCode",
    "Severity",
    "Tag",
    "Violation",
]
