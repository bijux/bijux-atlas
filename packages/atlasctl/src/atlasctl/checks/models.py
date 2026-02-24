from __future__ import annotations

from dataclasses import asdict, dataclass, field, is_dataclass
from enum import Enum
import json
from pathlib import Path
import re
from typing import Any, Mapping

SCHEMA_VERSION = 1

_CHECK_ID_RE = re.compile(r"^checks_[a-z0-9]+(?:_[a-z0-9]+)+$")
_ID_SEGMENT_RE = re.compile(r"^[a-z][a-z0-9_]{0,63}$")
_TAG_RE = re.compile(r"^[a-z0-9][a-z0-9_:.\-/]{0,63}$")

_DOMAIN_VOCAB = frozenset({"root", "python", "docs", "make", "ops", "policies", "product"})


class Status(str, Enum):
    PASS = "pass"
    FAIL = "fail"
    SKIP = "skip"
    ERROR = "error"


class Visibility(str, Enum):
    PUBLIC = "public"
    INTERNAL = "internal"


class Speed(str, Enum):
    FAST = "fast"
    SLOW = "slow"


class InternalError(RuntimeError):
    pass


class EffectDeniedError(InternalError):
    pass


class RegistryError(InternalError):
    pass


class SelectorError(InternalError):
    pass


class RepoRootError(InternalError):
    pass


class DeterminismError(InternalError):
    pass


@dataclass(frozen=True, order=True)
class CheckId:
    value: str

    @classmethod
    def parse(cls, value: str) -> "CheckId":
        raw = str(value).strip()
        if not _CHECK_ID_RE.fullmatch(raw):
            raise ValueError(f"invalid check id `{raw}`")
        return cls(raw)

    def __post_init__(self) -> None:
        object.__setattr__(self, "value", str(self.value).strip())

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True, order=True)
class DomainId:
    value: str

    @classmethod
    def parse(cls, value: str) -> "DomainId":
        return cls(value)

    def __post_init__(self) -> None:
        value = str(self.value).strip()
        if value not in _DOMAIN_VOCAB:
            raise ValueError(f"invalid domain `{value}`: expected one of {sorted(_DOMAIN_VOCAB)}")
        object.__setattr__(self, "value", value)

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True, order=True)
class SuiteId:
    value: str

    @classmethod
    def parse(cls, value: str) -> "SuiteId":
        return cls(value)

    def __post_init__(self) -> None:
        value = str(self.value).strip()
        if not _ID_SEGMENT_RE.fullmatch(value):
            raise ValueError(f"invalid suite id `{value}`")
        object.__setattr__(self, "value", value)

    def __str__(self) -> str:
        return self.value


@dataclass(frozen=True)
class EffectCaps:
    subprocess: bool = False
    network: bool = False
    fs_write: bool = False


@dataclass(frozen=True)
class CheckSpec:
    id: CheckId | str
    domain: DomainId | str
    title: str
    docs: str
    tags: tuple[str, ...] = ()
    speed: Speed | str = Speed.FAST
    visibility: Visibility | str = Visibility.PUBLIC
    effects_required: tuple[str, ...] = ("fs_read",)

    def __post_init__(self) -> None:
        cid = self.id if isinstance(self.id, CheckId) else CheckId.parse(str(self.id))
        did = self.domain if isinstance(self.domain, DomainId) else DomainId.parse(str(self.domain))
        speed = self.speed if isinstance(self.speed, Speed) else Speed(str(self.speed).strip().lower())
        visibility = self.visibility if isinstance(self.visibility, Visibility) else Visibility(str(self.visibility).strip().lower())
        tags = tuple(validate_tag(tag) for tag in self.tags)
        object.__setattr__(self, "id", cid)
        object.__setattr__(self, "domain", did)
        object.__setattr__(self, "title", str(self.title).strip())
        object.__setattr__(self, "docs", str(self.docs).strip())
        object.__setattr__(self, "tags", tags)
        object.__setattr__(self, "speed", speed)
        object.__setattr__(self, "visibility", visibility)
        object.__setattr__(self, "effects_required", tuple(sorted(set(str(item).strip() for item in self.effects_required if str(item).strip()))))


@dataclass(frozen=True)
class CheckContext:
    repo_root: Path
    artifacts_root: Path
    run_id: str
    env: Mapping[str, str] = field(default_factory=dict)
    adapters: Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(self, "repo_root", Path(self.repo_root))
        object.__setattr__(self, "artifacts_root", Path(self.artifacts_root))
        run_id = str(self.run_id).strip()
        if not run_id:
            raise ValueError("run_id cannot be empty")
        object.__setattr__(self, "run_id", run_id)


@dataclass(frozen=True)
class EvidenceRef:
    kind: str
    path: str
    description: str = ""
    content_type: str = "text/plain"

    def __post_init__(self) -> None:
        object.__setattr__(self, "kind", str(self.kind).strip())
        object.__setattr__(self, "path", str(self.path).strip())
        object.__setattr__(self, "description", str(self.description).strip())
        object.__setattr__(self, "content_type", str(self.content_type).strip())


@dataclass(frozen=True)
class Violation:
    code: str
    message: str
    hint: str = ""
    path: str = ""
    line: int = 0

    def __post_init__(self) -> None:
        object.__setattr__(self, "code", str(self.code).strip())
        object.__setattr__(self, "message", str(self.message).strip())
        object.__setattr__(self, "hint", str(self.hint).strip())
        object.__setattr__(self, "path", str(self.path).strip())
        object.__setattr__(self, "line", int(self.line or 0))


@dataclass(frozen=True)
class SkipReason:
    code: str
    message: str


@dataclass(frozen=True)
class Timing:
    duration_ms: int = 0
    budget_ms: int = 0


@dataclass(frozen=True)
class CheckResult:
    id: CheckId | str
    status: Status | str
    violations: tuple[Violation, ...] = ()
    evidence: tuple[EvidenceRef, ...] = ()
    timing: Timing = Timing()
    skip_reason: SkipReason | None = None

    def __post_init__(self) -> None:
        cid = self.id if isinstance(self.id, CheckId) else CheckId.parse(str(self.id))
        status = self.status if isinstance(self.status, Status) else Status(str(self.status).strip().lower())
        violations = stable_violations(self.violations)
        evidence = stable_evidence(self.evidence)
        object.__setattr__(self, "id", cid)
        object.__setattr__(self, "status", status)
        object.__setattr__(self, "violations", violations)
        object.__setattr__(self, "evidence", evidence)


@dataclass(frozen=True)
class RegistryRecord:
    id: str
    domain: str
    title: str
    tags: tuple[str, ...] = ()
    speed: str = Speed.FAST.value
    visibility: str = Visibility.PUBLIC.value


@dataclass(frozen=True)
class Selector:
    ids: tuple[CheckId, ...] = ()
    domains: tuple[DomainId, ...] = ()
    tags: tuple[str, ...] = ()
    suites: tuple[SuiteId, ...] = ()
    include_internal: bool = False


def validate_tag(value: str, *, banned_adjectives: set[str] | None = None) -> str:
    tag = str(value).strip()
    if not _TAG_RE.fullmatch(tag):
        raise ValueError(f"invalid tag `{tag}`")
    banned = banned_adjectives or set()
    if tag in banned:
        raise ValueError(f"forbidden tag `{tag}`")
    return tag


def stable_violations(rows: tuple[Violation, ...] | list[Violation]) -> tuple[Violation, ...]:
    return tuple(sorted(tuple(rows), key=lambda row: (row.path, row.line, row.code, row.message)))


def stable_evidence(rows: tuple[EvidenceRef, ...] | list[EvidenceRef]) -> tuple[EvidenceRef, ...]:
    return tuple(sorted(tuple(rows), key=lambda row: (row.kind, row.path, row.content_type, row.description)))


def _json_default(value: Any) -> Any:
    if isinstance(value, Path):
        return value.as_posix()
    if isinstance(value, Enum):
        return value.value
    if is_dataclass(value):
        return asdict(value)
    raise TypeError(f"unsupported value for json serialization: {type(value)}")


def to_json_dict(value: Any) -> dict[str, Any]:
    data = json.loads(json.dumps(value, default=_json_default, sort_keys=True))
    if not isinstance(data, dict):
        raise ValueError("serialized payload is not a JSON object")
    return data


def to_json_text(value: Any) -> str:
    return json.dumps(to_json_dict(value), indent=2, sort_keys=True) + "\n"


__all__ = [
    "SCHEMA_VERSION",
    "CheckContext",
    "CheckId",
    "CheckResult",
    "CheckSpec",
    "DeterminismError",
    "DomainId",
    "EffectCaps",
    "EffectDeniedError",
    "EvidenceRef",
    "InternalError",
    "RegistryError",
    "RegistryRecord",
    "RepoRootError",
    "Selector",
    "SelectorError",
    "SkipReason",
    "Speed",
    "Status",
    "SuiteId",
    "Timing",
    "Violation",
    "Visibility",
    "stable_evidence",
    "stable_violations",
    "to_json_dict",
    "to_json_text",
    "validate_tag",
]
