use std::cmp::Ordering;

fn matches_any_filter(filters: &[String], value: &str) -> bool {
    filters.is_empty() || filters.iter().any(|filter| wildcard_match(filter, value))
}

fn matches_skip_filter(filters: &[String], value: &str) -> bool {
    !filters.is_empty() && filters.iter().any(|filter| wildcard_match(filter, value))
}

pub fn contract_mode(contract: &Contract) -> ContractMode {
    let has_pure = contract.tests.iter().any(|case| case.kind == TestKind::Pure);
    let has_effect = contract
        .tests
        .iter()
        .any(|case| matches!(case.kind, TestKind::Subprocess | TestKind::Network));
    match (has_pure, has_effect) {
        (true, true) => ContractMode::Both,
        (false, true) => ContractMode::Effect,
        _ => ContractMode::Static,
    }
}

pub fn contract_effects(contract: &Contract) -> Vec<EffectKind> {
    let mut effects = BTreeSet::new();
    for case in &contract.tests {
        match case.kind {
            TestKind::Pure => {}
            TestKind::Subprocess => {
                effects.insert(EffectKind::Subprocess);
            }
            TestKind::Network => {
                effects.insert(EffectKind::Network);
            }
        }
    }
    effects.into_iter().collect()
}

fn derived_contract_tags(contract: &Contract) -> BTreeSet<&'static str> {
    let mut tags = BTreeSet::from(["ci"]);
    let mut has_pure = false;
    let mut has_effect = false;
    for case in &contract.tests {
        match case.kind {
            TestKind::Pure => has_pure = true,
            TestKind::Subprocess | TestKind::Network => has_effect = true,
        }
    }
    if has_pure {
        tags.insert("static");
    }
    if has_effect {
        tags.insert("effect");
        tags.insert("local");
        tags.insert("slow");
    }
    tags
}

fn matches_tags(filters: &[String], contract: &Contract) -> bool {
    if filters.is_empty() {
        return true;
    }
    let tags = derived_contract_tags(contract);
    filters.iter().any(|filter| {
        tags.iter()
            .any(|tag| wildcard_match(&filter.to_ascii_lowercase(), tag))
    })
}

pub fn required_effects_for_selection(
    contracts: &[Contract],
    mode: Mode,
    contract_filter: Option<&str>,
    test_filter: Option<&str>,
    only_contracts: &[String],
    only_tests: &[String],
    skip_contracts: &[String],
    tags: &[String],
) -> EffectRequirement {
    let mut required = EffectRequirement {
        allow_subprocess: false,
        allow_network: false,
        allow_k8s: false,
        allow_fs_write: false,
        allow_docker_daemon: false,
    };
    if mode != Mode::Effect {
        return required;
    }
    for contract in contracts {
        if !matches_filter(&contract_filter.map(ToOwned::to_owned), &contract.id.0)
            || !matches_any_filter(only_contracts, &contract.id.0)
            || matches_skip_filter(skip_contracts, &contract.id.0)
            || !matches_tags(tags, contract)
        {
            continue;
        }
        for case in &contract.tests {
            if !matches_filter(&test_filter.map(ToOwned::to_owned), &case.id.0)
                || !matches_any_filter(only_tests, &case.id.0)
            {
                continue;
            }
            match case.kind {
                TestKind::Pure => {}
                TestKind::Subprocess => required.allow_subprocess = true,
                TestKind::Network => required.allow_network = true,
            }
        }
    }
    required
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let mut regex = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            _ => regex.push_str(&regex::escape(&ch.to_string())),
        }
    }
    regex.push('$');
    regex::Regex::new(&regex)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

fn matches_filter(filter: &Option<String>, value: &str) -> bool {
    filter
        .as_deref()
        .map(|f| wildcard_match(f, value))
        .unwrap_or(true)
}

fn case_status_from_result(result: &TestResult) -> CaseStatus {
    match result {
        TestResult::Pass => CaseStatus::Pass,
        TestResult::Fail(_) => CaseStatus::Fail,
        TestResult::Skip(_) => CaseStatus::Skip,
        TestResult::Error(_) => CaseStatus::Error,
    }
}

fn worst_status(current: CaseStatus, next: CaseStatus) -> CaseStatus {
    fn rank(s: CaseStatus) -> u8 {
        match s {
            CaseStatus::Pass => 0,
            CaseStatus::Skip => 1,
            CaseStatus::Fail => 2,
            CaseStatus::Error => 3,
        }
    }
    match rank(current).cmp(&rank(next)) {
        Ordering::Less => next,
        _ => current,
    }
}
