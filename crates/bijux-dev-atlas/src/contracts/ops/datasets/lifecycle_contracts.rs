fn test_ops_dataset_001_manifest_and_lock(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-001";
    let test_id = "ops.datasets.manifest_and_lock_consistent";
    let manifest_path = ctx.repo_root.join("ops/datasets/manifest.json");
    let lock_path = ctx.repo_root.join("ops/datasets/manifest.lock");
    let Some(manifest) = read_json(&manifest_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "datasets manifest must be valid json",
            Some("ops/datasets/manifest.json".to_string()),
        )]);
    };
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "datasets manifest lock must be valid json",
            Some("ops/datasets/manifest.lock".to_string()),
        )]);
    };
    let mut manifest_ids = BTreeSet::new();
    if let Some(items) = manifest.get("datasets").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                manifest_ids.insert(id.to_string());
            }
        }
    }
    let mut lock_ids = BTreeSet::new();
    if let Some(items) = lock.get("entries").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                lock_ids.insert(id.to_string());
            }
        }
    }
    let mut violations = Vec::new();
    if manifest_ids != lock_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "datasets manifest ids must match manifest.lock ids",
            Some("ops/datasets/manifest.lock".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
fn test_ops_dataset_002_fixture_inventory_matches_disk(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-002";
    let test_id = "ops.datasets.fixture_inventory_matches_disk";
    let inventory_path = ctx.repo_root.join("ops/datasets/generated/fixture-inventory.json");
    let Some(inventory) = read_json(&inventory_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "fixture inventory must be valid json",
            Some("ops/datasets/generated/fixture-inventory.json".to_string()),
        )]);
    };
    let fixtures_dir = ctx.repo_root.join("ops/datasets/fixtures");
    let mut disk_names = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(&fixtures_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|v| v.to_str()) {
                disk_names.insert(name.to_string());
            }
        }
        }
    }
    let mut inventory_names = BTreeSet::new();
    if let Some(items) = inventory.get("fixtures").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                inventory_names.insert(name.to_string());
            }
            let lock = item.get("manifest_lock").and_then(|v| v.as_str()).unwrap_or("");
            let src_dir = item.get("src_dir").and_then(|v| v.as_str()).unwrap_or("");
            if !lock.is_empty() && !ctx.repo_root.join(lock).exists() {
                return TestResult::Fail(vec![violation(
                    contract_id,
                    test_id,
                    "fixture inventory references missing manifest_lock",
                    Some(lock.to_string()),
                )]);
            }
            if !src_dir.is_empty() && !ctx.repo_root.join(src_dir).exists() {
                return TestResult::Fail(vec![violation(
                    contract_id,
                    test_id,
                    "fixture inventory references missing src_dir",
                    Some(src_dir.to_string()),
                )]);
            }
        }
    }
    let mut violations = Vec::new();
    if inventory_names != disk_names {
        violations.push(violation(
            contract_id,
            test_id,
            "fixture inventory names must match fixture directories on disk",
            Some("ops/datasets/generated/fixture-inventory.json".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_datasets_011_fixture_archives_match_manifest_lock(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-011";
    let test_id = "ops.datasets.fixture_archives_match_manifest_lock";
    let mut violations = Vec::new();
    let rows = [
        (
            "ops/datasets/fixtures/medium/v1/assets/bijux-atlas-medium-fixture-v1.tar.gz",
            "ops/datasets/fixtures/medium/v1/manifest.lock",
        ),
        (
            "ops/datasets/fixtures/release-diff/v1/assets/bijux-atlas-release-diff-v1.tar.gz",
            "ops/datasets/fixtures/release-diff/v1/manifest.lock",
        ),
    ];
    for (asset_rel, lock_rel) in rows {
        let asset_path = ctx.repo_root.join(asset_rel);
        let lock_text = match std::fs::read_to_string(ctx.repo_root.join(lock_rel)) {
            Ok(v) => v,
            Err(_) => {
                violations.push(violation(contract_id, test_id, "fixture manifest lock must be readable", Some(lock_rel.to_string())));
                continue;
            }
        };
        let expected = lock_text
            .lines()
            .find_map(|line| line.strip_prefix("sha256="))
            .unwrap_or("")
            .trim()
            .to_string();
        if expected.is_empty() {
            violations.push(violation(contract_id, test_id, "fixture manifest lock must declare sha256", Some(lock_rel.to_string())));
            continue;
        }
        if !asset_path.exists() {
            violations.push(violation(contract_id, test_id, "fixture archive is missing", Some(asset_rel.to_string())));
            continue;
        }
        let Some(digest) = file_sha256(&asset_path) else {
            violations.push(violation(contract_id, test_id, "fixture archive must be readable", Some(asset_rel.to_string())));
            continue;
        };
        if digest != expected {
            violations.push(violation(contract_id, test_id, "fixture archive digest must match manifest lock", Some(asset_rel.to_string())));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_ops_datasets_012_real_datasets_have_provenance(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-012";
    let test_id = "ops.datasets.real_datasets_have_provenance";
    let rel = "ops/datasets/real-datasets.json";
    let Some(payload) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(contract_id, test_id, "real datasets registry must be valid json", Some(rel.to_string()))]);
    };
    let mut violations = Vec::new();
    for item in payload.get("datasets").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
        let dataset_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("<unknown>");
        for field in ["source_kind", "source_ref", "review_status"] {
            if item.get(field).and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
                violations.push(violation(contract_id, test_id, "real dataset entries must declare provenance fields", Some(format!("{rel}:{dataset_id}:{field}"))));
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_ops_datasets_013_allowed_file_types_only(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASETS-013";
    let test_id = "ops.datasets.allowed_file_types_only";
    let allowed_ext = [
        "json",
        "lock",
        "md",
        "gz",
        "gff3",
        "fa",
        "fai",
        "sqlite",
    ];
    let mut violations = Vec::new();
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/datasets"), &mut files);
    files.sort();
    for path in files {
        let rel = path
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name == "CONTRACT.md" || name == "README.md" {
            continue;
        }
        let allowed = allowed_ext.iter().any(|ext| name.ends_with(&format!(".{ext}")))
            || name == "manifest.lock";
        if !allowed {
            violations.push(violation(contract_id, test_id, "datasets tree contains a file type outside the allowlist", Some(rel)));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}
