fn test_ops_dataset_003_no_fixture_drift_without_promotion_record(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-003";
    let test_id = "ops.datasets.no_fixture_drift_without_promotion_record";
    let index_path = ctx.repo_root.join("ops/datasets/generated/dataset-index.json");
    let Some(index) = read_json(&index_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "dataset index must be valid json",
            Some("ops/datasets/generated/dataset-index.json".to_string()),
        )]);
    };

    let missing_count = index
        .get("missing_dataset_ids")
        .and_then(|v| v.as_array())
        .map_or(0, |items| items.len());
    let stale_count = index
        .get("stale_dataset_ids")
        .and_then(|v| v.as_array())
        .map_or(0, |items| items.len());

    if missing_count + stale_count == 0 {
        return TestResult::Pass;
    }

    let promotion_rules_path = ctx.repo_root.join("ops/datasets/promotion-rules.json");
    let Some(promotion_rules) = read_json(&promotion_rules_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "promotion-rules must be valid json when dataset drift exists",
            Some("ops/datasets/promotion-rules.json".to_string()),
        )]);
    };
    let has_promotion_rule = promotion_rules
        .get("rules")
        .and_then(|v| v.as_array())
        .is_some_and(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .any(|rule| rule.contains("promotion"))
        });
    if !has_promotion_rule {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "fixture drift requires explicit promotion rule coverage",
            Some("ops/datasets/promotion-rules.json".to_string()),
        )]);
    }
    TestResult::Pass
}

fn test_ops_dataset_004_release_diff_fixtures_are_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-004";
    let test_id = "ops.datasets.release_diff_fixtures_deterministic";
    let lock_path = ctx
        .repo_root
        .join("ops/datasets/fixtures/release-diff/v1/manifest.lock");
    let Ok(lock_text) = std::fs::read_to_string(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "release-diff fixture manifest.lock is required",
            Some("ops/datasets/fixtures/release-diff/v1/manifest.lock".to_string()),
        )]);
    };
    let has_sha256 = lock_text.lines().any(|line| line.starts_with("sha256="));
    let has_archive = lock_text.lines().any(|line| line.starts_with("archive="));
    let mut violations = Vec::new();
    if !has_sha256 || !has_archive {
        violations.push(violation(
            contract_id,
            test_id,
            "release-diff manifest.lock must include sha256 and archive pins",
            Some("ops/datasets/fixtures/release-diff/v1/manifest.lock".to_string()),
        ));
    }

    for rel in [
        "ops/datasets/fixtures/release-diff/v1/release-diff-queries.v1.json",
        "ops/datasets/fixtures/release-diff/v1/release-diff-responses.v1.json",
    ] {
        let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "release-diff golden files must be valid json",
                Some(rel.to_string()),
            ));
            continue;
        };
        if !value.is_array() {
            violations.push(violation(
                contract_id,
                test_id,
                "release-diff golden files must be top-level arrays",
                Some(rel.to_string()),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

