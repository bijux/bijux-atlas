// SPDX-License-Identifier: Apache-2.0
//! System invariants registry for institutional verification.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvariantSeverity {
    Critical,
    High,
    Medium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvariantGroup {
    Config,
    Runtime,
    Ops,
    Registry,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct SystemInvariant {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub severity: InvariantSeverity,
    pub group: InvariantGroup,
}

pub fn registry() -> Vec<SystemInvariant> {
    let mut rows = vec![
        SystemInvariant {
            id: "INV-CONFIG-SCHEMA-VERSION-001",
            title: "Config schema version matches runtime loader version",
            summary: "The governed config schema version must match the runtime loader contract.",
            severity: InvariantSeverity::Critical,
            group: InvariantGroup::Config,
        },
        SystemInvariant {
            id: "INV-ARTIFACT-HASH-REGISTRY-001",
            title: "Artifact hashes match registry manifest entries",
            summary: "Release evidence manifest checksums must match on-disk artifacts.",
            severity: InvariantSeverity::Critical,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-DATASET-ID-UNIQUE-001",
            title: "Dataset IDs are globally unique",
            summary: "Generated dataset index and manifest lock must not contain duplicate dataset IDs.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-DATASET-SCHEMA-METADATA-001",
            title: "Dataset schema version matches stored metadata",
            summary: "Dataset index schema version must match manifest lock schema version.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-SHARD-ID-DETERMINISTIC-001",
            title: "Shard identifiers are deterministic",
            summary: "Shard IDs in generated lineage must be sorted and stable.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-SHARD-DIR-REGISTRY-001",
            title: "Shard directories map to registry entries",
            summary: "Shard-like artifact directories must be represented in release/evidence manifest entries.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-SHARD-ORPHAN-001",
            title: "No shard exists without registry metadata",
            summary: "All shard artifacts discovered in evidence must include metadata rows in manifest.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-REGISTRY-REF-EXISTS-001",
            title: "Registry references only existing artifacts",
            summary: "Every manifest path must resolve to an existing file under repository root.",
            severity: InvariantSeverity::Critical,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-REGISTRY-CHECKSUM-001",
            title: "Artifact registry entries include checksums",
            summary: "Every manifest artifact row must include non-empty SHA-256 checksum text.",
            severity: InvariantSeverity::Critical,
            group: InvariantGroup::Registry,
        },
        SystemInvariant {
            id: "INV-CONFIG-DATASET-REF-001",
            title: "Config references only known datasets",
            summary: "Configured pinned datasets must be present in generated dataset index.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Config,
        },
        SystemInvariant {
            id: "INV-CONFIG-PROFILE-REF-001",
            title: "Config references only known profiles",
            summary: "Install matrix profile references must exist in the stack profile registry.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Config,
        },
        SystemInvariant {
            id: "INV-PROFILE-NAME-UNIQUE-001",
            title: "Profile names are unique",
            summary: "Profile IDs in stack profile registry and ops stack profile list must be unique.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Ops,
        },
        SystemInvariant {
            id: "INV-PROFILE-INHERIT-CYCLE-001",
            title: "Profile inheritance has no cycles",
            summary: "Profile inheritance graph must be acyclic when inherits_from is declared.",
            severity: InvariantSeverity::Medium,
            group: InvariantGroup::Ops,
        },
        SystemInvariant {
            id: "INV-PROFILE-OVERRIDE-SCHEMA-001",
            title: "Profile overrides respect schema",
            summary: "Every values file referenced by profiles must exist and remain YAML-parseable.",
            severity: InvariantSeverity::High,
            group: InvariantGroup::Ops,
        },
        SystemInvariant {
            id: "INV-RUNTIME-START-GATE-001",
            title: "Runtime start requires invariant pass",
            summary: "Runtime start gate is represented by invariant runner failure status and stable exit code.",
            severity: InvariantSeverity::Critical,
            group: InvariantGroup::Runtime,
        },
    ];
    rows.sort_by_key(|row| row.id);
    rows
}
