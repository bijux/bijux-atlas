// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::{
    AuthorizationDecision, AuthorizationEngine, AuthorizationPolicy, PermissionCatalog,
    PermissionEvaluator, RoleCatalog, RoleRegistry,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn load_engine() -> AuthorizationEngine {
    let root = workspace_root();
    let role_catalog: RoleCatalog = serde_yaml::from_str(
        &fs::read_to_string(root.join("configs/security/roles.yaml")).expect("read roles"),
    )
    .expect("parse roles");
    let permission_catalog: PermissionCatalog = serde_yaml::from_str(
        &fs::read_to_string(root.join("configs/security/permissions.yaml"))
            .expect("read permissions"),
    )
    .expect("parse permissions");
    let policy: AuthorizationPolicy = serde_yaml::from_str(
        &fs::read_to_string(root.join("configs/security/policy.yaml")).expect("read policy"),
    )
    .expect("parse policy");
    let assignments: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(root.join("configs/security/role-assignments.yaml"))
            .expect("read role assignments"),
    )
    .expect("parse role assignments");

    let mut registry = RoleRegistry::new();
    for role in role_catalog.roles {
        registry.upsert_role(role);
    }
    if let Some(rows) = assignments.get("assignments").and_then(serde_yaml::Value::as_sequence) {
        for row in rows {
            let Some(principal) = row.get("principal").and_then(serde_yaml::Value::as_str) else {
                continue;
            };
            let Some(role_id) = row.get("role_id").and_then(serde_yaml::Value::as_str) else {
                continue;
            };
            registry.assign_role(principal, role_id);
        }
    }

    AuthorizationEngine::new(registry, PermissionEvaluator::new(permission_catalog), policy)
}

fn authorization_benchmarks(c: &mut Criterion) {
    let engine = load_engine();

    c.bench_function("security_authorization_evaluate_allow", |b| {
        b.iter(|| {
            let decision = engine.evaluate(
                black_box("operator"),
                black_box("ops.admin"),
                black_box("namespace"),
                black_box("/debug"),
            );
            assert_eq!(decision, AuthorizationDecision::Allow);
        })
    });

    c.bench_function("security_authorization_evaluate_deny", |b| {
        b.iter(|| {
            let decision = engine.evaluate(
                black_box("user"),
                black_box("ops.admin"),
                black_box("namespace"),
                black_box("/debug"),
            );
            assert_eq!(decision, AuthorizationDecision::Deny);
        })
    });
}

criterion_group!(authorization, authorization_benchmarks);
criterion_main!(authorization);
