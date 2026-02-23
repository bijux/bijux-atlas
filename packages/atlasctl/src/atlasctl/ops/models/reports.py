from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class OpsTestReport:
    test_kind: str
    run_id: str
    status: str
    command: list[str]
    out_rel: str | None = None
    evidence: list[str] = field(default_factory=list)

    def to_payload(self) -> dict[str, object]:
        return {
            "schema_name": "atlasctl.ops-test-report.v1",
            "kind": f"ops-{self.test_kind}-report",
            "run_id": self.run_id,
            "status": self.status,
            "command": list(self.command),
            "out": self.out_rel,
            "evidence": list(self.evidence),
        }


@dataclass(frozen=True)
class OpsEvidenceCollectReport:
    run_id: str
    status: str
    namespace: str
    pods: list[str]
    attachments: dict[str, str]

    def to_payload(self) -> dict[str, object]:
        return {
            "schema_name": "atlasctl.ops-evidence-collect.v1",
            "kind": "ops-evidence-collect",
            "run_id": self.run_id,
            "status": self.status,
            "namespace": self.namespace,
            "pods": list(self.pods),
            "attachments": dict(self.attachments),
        }
