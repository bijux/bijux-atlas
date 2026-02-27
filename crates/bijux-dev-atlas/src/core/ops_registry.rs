// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpsTag {
    Fast,
    Slow,
    Offline,
    Online,
    Destructive,
    RequiresKind,
    RequiresDocker,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpsCommandSpec {
    pub domain: &'static str,
    pub verb: &'static str,
    pub subverb: Option<&'static str>,
    pub tags: &'static [OpsTag],
}

pub fn builtin_ops_registry() -> Vec<OpsCommandSpec> {
    let mut entries = vec![
        OpsCommandSpec {
            domain: "stack",
            verb: "up",
            subverb: None,
            tags: &[
                OpsTag::Slow,
                OpsTag::Online,
                OpsTag::RequiresKind,
                OpsTag::RequiresDocker,
            ],
        },
        OpsCommandSpec {
            domain: "stack",
            verb: "down",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Destructive, OpsTag::RequiresKind],
        },
        OpsCommandSpec {
            domain: "stack",
            verb: "status",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "stack",
            verb: "logs",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "stack",
            verb: "ports",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "stack",
            verb: "versions",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "stack",
            verb: "doctor",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "stack",
            verb: "reset",
            subverb: None,
            tags: &[OpsTag::Destructive, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "k8s",
            verb: "render",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "k8s",
            verb: "install",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Online, OpsTag::RequiresKind],
        },
        OpsCommandSpec {
            domain: "k8s",
            verb: "uninstall",
            subverb: None,
            tags: &[OpsTag::Destructive, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "k8s",
            verb: "test",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Online, OpsTag::RequiresKind],
        },
        OpsCommandSpec {
            domain: "k8s",
            verb: "conformance",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Online, OpsTag::RequiresKind],
        },
        OpsCommandSpec {
            domain: "k8s",
            verb: "diff",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "k8s",
            verb: "rollout",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "load",
            verb: "run",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "load",
            verb: "baseline",
            subverb: Some("update"),
            tags: &[OpsTag::Slow, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "load",
            verb: "evaluate",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "load",
            verb: "report",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "load",
            verb: "list-suites",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "observe",
            verb: "up",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "observe",
            verb: "down",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Destructive, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "observe",
            verb: "validate",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "observe",
            verb: "snapshot",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "observe",
            verb: "drill",
            subverb: Some("run"),
            tags: &[OpsTag::Slow, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "observe",
            verb: "dashboards",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "datasets",
            verb: "list",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "datasets",
            verb: "ingest",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "datasets",
            verb: "publish",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "datasets",
            verb: "promote",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "datasets",
            verb: "rollback",
            subverb: None,
            tags: &[OpsTag::Destructive, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "datasets",
            verb: "qc",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "e2e",
            verb: "run",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "e2e",
            verb: "smoke",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "e2e",
            verb: "realdata",
            subverb: None,
            tags: &[OpsTag::Slow, OpsTag::Online],
        },
        OpsCommandSpec {
            domain: "e2e",
            verb: "list-suites",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "schema",
            verb: "validate",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "schema",
            verb: "diff",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "schema",
            verb: "coverage",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "schema",
            verb: "regen-index",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "inventory",
            verb: "validate",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "inventory",
            verb: "graph",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "inventory",
            verb: "diff",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "inventory",
            verb: "coverage",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "inventory",
            verb: "orphan-check",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "report",
            verb: "generate",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "report",
            verb: "diff",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "report",
            verb: "readiness",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "report",
            verb: "bundle",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
        OpsCommandSpec {
            domain: "docs",
            verb: "build",
            subverb: None,
            tags: &[OpsTag::Fast, OpsTag::Offline],
        },
    ];
    entries.sort();
    entries
}

pub fn ops_domains() -> BTreeSet<&'static str> {
    builtin_ops_registry()
        .into_iter()
        .map(|entry| entry.domain)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{builtin_ops_registry, ops_domains};

    #[test]
    fn ops_registry_covers_required_domains() {
        let domains = ops_domains();
        for required in [
            "stack",
            "k8s",
            "datasets",
            "e2e",
            "load",
            "observe",
            "report",
            "schema",
            "inventory",
            "docs",
        ] {
            assert!(
                domains.contains(required),
                "missing required ops domain `{required}`"
            );
        }
    }

    #[test]
    fn ops_registry_entries_are_sorted_and_unique() {
        let entries = builtin_ops_registry();
        let mut sorted = entries.clone();
        sorted.sort();
        assert_eq!(entries, sorted, "ops registry entries must stay sorted");
        sorted.dedup();
        assert_eq!(
            entries.len(),
            sorted.len(),
            "ops registry entries must be unique"
        );
    }

    #[test]
    fn ops_registry_covers_required_domain_verbs() {
        let entries = builtin_ops_registry();
        let present: BTreeSet<(&str, &str, Option<&str>)> = entries
            .iter()
            .map(|entry| (entry.domain, entry.verb, entry.subverb))
            .collect();

        for required in [
            ("stack", "up", None),
            ("stack", "down", None),
            ("stack", "status", None),
            ("stack", "logs", None),
            ("stack", "ports", None),
            ("stack", "versions", None),
            ("stack", "doctor", None),
            ("stack", "reset", None),
            ("k8s", "render", None),
            ("k8s", "install", None),
            ("k8s", "uninstall", None),
            ("k8s", "test", None),
            ("k8s", "conformance", None),
            ("k8s", "diff", None),
            ("k8s", "rollout", None),
            ("load", "run", None),
            ("load", "baseline", Some("update")),
            ("load", "evaluate", None),
            ("load", "report", None),
            ("load", "list-suites", None),
            ("observe", "up", None),
            ("observe", "down", None),
            ("observe", "validate", None),
            ("observe", "snapshot", None),
            ("observe", "drill", Some("run")),
            ("observe", "dashboards", None),
            ("datasets", "list", None),
            ("datasets", "ingest", None),
            ("datasets", "publish", None),
            ("datasets", "promote", None),
            ("datasets", "rollback", None),
            ("datasets", "qc", None),
            ("e2e", "run", None),
            ("e2e", "smoke", None),
            ("e2e", "realdata", None),
            ("e2e", "list-suites", None),
            ("schema", "validate", None),
            ("schema", "diff", None),
            ("schema", "coverage", None),
            ("schema", "regen-index", None),
            ("inventory", "validate", None),
            ("inventory", "graph", None),
            ("inventory", "diff", None),
            ("inventory", "coverage", None),
            ("inventory", "orphan-check", None),
            ("report", "generate", None),
            ("report", "diff", None),
            ("report", "readiness", None),
            ("report", "bundle", None),
        ] {
            assert!(
                present.contains(&required),
                "missing required ops command spec: {:?}",
                required
            );
        }
    }
}
