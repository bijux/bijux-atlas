// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schema::{PolicyMode, PolicySet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PolicySeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyViolation {
    pub id: &'static str,
    pub severity: PolicySeverity,
    pub message: &'static str,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryMetrics {
    pub dataset_count: u32,
    pub open_shards_per_pod: u32,
    pub disk_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
enum RuleKind {
    BoolEquals(bool),
    NumberMin(u64),
    ArrayNonEmpty,
}

#[derive(Debug, Clone, Copy)]
struct RuleSpec {
    id: &'static str,
    severity: PolicySeverity,
    message: &'static str,
    path: &'static str,
    kind: RuleKind,
}

const POLICY_RULES: &[RuleSpec] = &[
    RuleSpec {
        id: "policy.global.allow_override.forbidden",
        severity: PolicySeverity::Error,
        message: "allow_override must be false",
        path: "allow_override",
        kind: RuleKind::BoolEquals(false),
    },
    RuleSpec {
        id: "policy.global.network_in_unit_tests.forbidden",
        severity: PolicySeverity::Error,
        message: "network_in_unit_tests must be false",
        path: "network_in_unit_tests",
        kind: RuleKind::BoolEquals(false),
    },
    RuleSpec {
        id: "policy.telemetry.metrics.required",
        severity: PolicySeverity::Error,
        message: "telemetry.metrics_enabled must be true",
        path: "telemetry.metrics_enabled",
        kind: RuleKind::BoolEquals(true),
    },
    RuleSpec {
        id: "policy.telemetry.tracing.required",
        severity: PolicySeverity::Error,
        message: "telemetry.tracing_enabled must be true",
        path: "telemetry.tracing_enabled",
        kind: RuleKind::BoolEquals(true),
    },
    RuleSpec {
        id: "policy.telemetry.request_id.required",
        severity: PolicySeverity::Error,
        message: "telemetry.request_id_required must be true",
        path: "telemetry.request_id_required",
        kind: RuleKind::BoolEquals(true),
    },
    RuleSpec {
        id: "policy.telemetry.labels.non_empty",
        severity: PolicySeverity::Error,
        message: "telemetry.required_metric_labels must not be empty",
        path: "telemetry.required_metric_labels",
        kind: RuleKind::ArrayNonEmpty,
    },
    RuleSpec {
        id: "policy.query_budget.max_limit.min",
        severity: PolicySeverity::Error,
        message: "query_budget.max_limit must be > 0",
        path: "query_budget.max_limit",
        kind: RuleKind::NumberMin(1),
    },
    RuleSpec {
        id: "policy.response_budget.max_serialization_bytes.min",
        severity: PolicySeverity::Error,
        message: "response_budget.max_serialization_bytes must be > 0",
        path: "response_budget.max_serialization_bytes",
        kind: RuleKind::NumberMin(1),
    },
    RuleSpec {
        id: "policy.cache_budget.max_disk_bytes.min",
        severity: PolicySeverity::Error,
        message: "cache_budget.max_disk_bytes must be > 0",
        path: "cache_budget.max_disk_bytes",
        kind: RuleKind::NumberMin(1),
    },
    RuleSpec {
        id: "policy.publish_gates.required_indexes.non_empty",
        severity: PolicySeverity::Error,
        message: "publish_gates.required_indexes must not be empty",
        path: "publish_gates.required_indexes",
        kind: RuleKind::ArrayNonEmpty,
    },
];

#[must_use]
pub fn evaluate_policy_set(policy: &PolicySet) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();
    let value = match serde_json::to_value(policy) {
        Ok(value) => value,
        Err(error) => {
            violations.push(PolicyViolation {
                id: "policy.serialization.failure",
                severity: PolicySeverity::Error,
                message: "policy set serialization failed",
                evidence: error.to_string(),
            });
            return violations;
        }
    };

    for rule in POLICY_RULES {
        apply_rule(&value, *rule, &mut violations);
    }

    let strict_allow_override = policy.modes.strict.allow_override;
    if strict_allow_override {
        violations.push(PolicyViolation {
            id: "policy.mode.strict.allow_override.forbidden",
            severity: PolicySeverity::Error,
            message: "strict mode must keep allow_override=false",
            evidence: "modes.strict.allow_override=true".to_string(),
        });
    }

    let dev_allow_override = policy.modes.dev.allow_override;
    if dev_allow_override {
        violations.push(PolicyViolation {
            id: "policy.mode.dev.allow_override.forbidden",
            severity: PolicySeverity::Error,
            message: "dev mode must keep allow_override=false",
            evidence: "modes.dev.allow_override=true".to_string(),
        });
    }

    let active = match policy.mode {
        PolicyMode::Strict => &policy.modes.strict,
        PolicyMode::Compat => &policy.modes.compat,
        PolicyMode::Dev => &policy.modes.dev,
    };

    if active.max_page_size == 0 || active.max_region_span == 0 || active.max_response_bytes == 0 {
        violations.push(PolicyViolation {
            id: "policy.mode.active.cap_values.invalid",
            severity: PolicySeverity::Error,
            message: "active mode cap table values must be > 0",
            evidence: format!("active_mode={}", policy.mode.as_str()),
        });
    }

    violations
}

#[must_use]
pub fn evaluate_repository_metrics(
    policy: &PolicySet,
    metrics: &RepositoryMetrics,
) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();

    if metrics.dataset_count > policy.cache_budget.max_dataset_count {
        violations.push(PolicyViolation {
            id: "policy.cache.dataset_count.exceeded",
            severity: PolicySeverity::Error,
            message: "dataset count exceeds cache policy budget",
            evidence: format!(
                "dataset_count={} max_dataset_count={}",
                metrics.dataset_count, policy.cache_budget.max_dataset_count
            ),
        });
    }

    if metrics.open_shards_per_pod > policy.cache_budget.max_open_shards_per_pod {
        violations.push(PolicyViolation {
            id: "policy.cache.open_shards_per_pod.exceeded",
            severity: PolicySeverity::Error,
            message: "open shard count exceeds cache policy budget",
            evidence: format!(
                "open_shards_per_pod={} max_open_shards_per_pod={}",
                metrics.open_shards_per_pod, policy.cache_budget.max_open_shards_per_pod
            ),
        });
    }

    if metrics.disk_bytes > policy.cache_budget.max_disk_bytes {
        violations.push(PolicyViolation {
            id: "policy.cache.disk_bytes.exceeded",
            severity: PolicySeverity::Warning,
            message: "disk usage exceeds cache policy budget",
            evidence: format!(
                "disk_bytes={} max_disk_bytes={}",
                metrics.disk_bytes, policy.cache_budget.max_disk_bytes
            ),
        });
    }

    violations
}

fn apply_rule(root: &Value, rule: RuleSpec, out: &mut Vec<PolicyViolation>) {
    let Some(value) = field_path(root, rule.path) else {
        out.push(PolicyViolation {
            id: rule.id,
            severity: PolicySeverity::Error,
            message: "required policy path missing",
            evidence: format!("path={}", rule.path),
        });
        return;
    };

    let pass = match rule.kind {
        RuleKind::BoolEquals(expected) => value.as_bool() == Some(expected),
        RuleKind::NumberMin(min) => value.as_u64().is_some_and(|n| n >= min),
        RuleKind::ArrayNonEmpty => value.as_array().is_some_and(|v| !v.is_empty()),
    };

    if !pass {
        out.push(PolicyViolation {
            id: rule.id,
            severity: rule.severity,
            message: rule.message,
            evidence: format!("path={} value={}", rule.path, value),
        });
    }
}

fn field_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;
    for seg in path.split('.') {
        cur = cur.get(seg)?;
    }
    Some(cur)
}
