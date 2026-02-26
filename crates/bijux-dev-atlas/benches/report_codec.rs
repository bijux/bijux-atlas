// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::model::{
    schema_version, ArtifactPath, CheckId, CheckResult, CheckStatus, EvidenceRef, RunId, RunReport,
    RunSummary, Severity, Violation, ViolationId,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::BTreeMap;

fn large_report(size: usize) -> RunReport {
    let mut results = Vec::with_capacity(size);
    let mut timings = BTreeMap::new();
    for idx in 0..size {
        let check_id = CheckId::parse(&format!("checks_ops_surface_manifest_{idx}"))
            .unwrap_or_else(|_| CheckId::parse("checks_ops_surface_manifest").expect("id"));
        let violation = Violation {
            schema_version: schema_version(),
            code: ViolationId::parse("ops_contract_missing").expect("code"),
            message: format!("missing contract {idx}"),
            hint: Some("restore file".to_string()),
            path: Some(ArtifactPath::parse("ops/CONTRACT.md").expect("path")),
            line: Some(1),
            severity: Severity::Error,
        };
        let evidence = EvidenceRef {
            schema_version: schema_version(),
            kind: "text".to_string(),
            path: ArtifactPath::parse("artifacts/atlas-dev/registry_run/report.json")
                .expect("path"),
            content_type: "application/json".to_string(),
            description: "report output".to_string(),
        };
        results.push(CheckResult {
            schema_version: schema_version(),
            id: check_id.clone(),
            status: CheckStatus::Fail,
            skip_reason: None,
            violations: vec![violation],
            duration_ms: 12,
            evidence: vec![evidence],
        });
        timings.insert(check_id, 12);
    }
    let summary = RunSummary {
        schema_version: schema_version(),
        passed: 0,
        failed: size as u64,
        skipped: 0,
        errors: 0,
        total: size as u64,
    };
    RunReport {
        schema_version: schema_version(),
        run_id: RunId::from_seed("registry_run"),
        repo_root: "/repo".to_string(),
        command: "check run".to_string(),
        selections: BTreeMap::new(),
        capabilities: BTreeMap::new(),
        results,
        durations_ms: timings.clone(),
        counts: summary.clone(),
        summary,
        timings_ms: timings,
    }
}

fn bench_encode_decode(c: &mut Criterion) {
    let report = large_report(200);
    c.bench_function("report_json_encode", |b| {
        b.iter(|| {
            serde_json::to_string(&report).expect("encode");
        })
    });

    let json = serde_json::to_string(&report).expect("encode sample");
    c.bench_function("report_json_decode", |b| {
        b.iter(|| {
            let _: RunReport = serde_json::from_str(&json).expect("decode");
        })
    });
}

criterion_group!(report_codec, bench_encode_decode);
criterion_main!(report_codec);
