pub fn to_pretty(report: &RunReport) -> String {
    fn dotted_with_width(label: &str, status: &str, width: usize) -> String {
        let left = if label.len() >= width {
            label.to_string()
        } else {
            format!("{label} {}", ".".repeat(width - label.len()))
        };
        format!("{left} {status}")
    }
    fn dotted(label: &str, status: &str) -> String {
        const WIDTH: usize = 72;
        dotted_with_width(label, status, WIDTH)
    }

    let mut out = String::new();
    out.push_str(&format!(
        "Contracts: {} (mode={}, duration={}ms)\n",
        report.domain, report.mode, report.duration_ms
    ));
    for contract in &report.contracts {
        out.push_str(&format!(
            "{}\n",
            dotted(
                &format!("{} {}", contract.id, contract.title),
                &format!("{} ({}ms)", contract.status.as_colored(), contract.duration_ms)
            )
        ));
        for case in report.cases.iter().filter(|c| c.contract_id == contract.id) {
            out.push_str(&format!(
                "  {}\n",
                dotted_with_width(
                    &case.test_id,
                    &format!("{} ({}ms)", case.status.as_colored(), case.duration_ms),
                    70,
                )
            ));
            for violation in &case.violations {
                let location = match (&violation.file, violation.line) {
                    (Some(file), Some(line)) => format!("{file}:{line}"),
                    (Some(file), None) => file.clone(),
                    _ => "unknown-location".to_string(),
                };
                out.push_str(&format!("    - {}: {}\n", location, violation.message));
                if let Some(evidence) = &violation.evidence {
                    out.push_str(&format!("      evidence: {}\n", evidence.trim()));
                }
            }
            if let Some(note) = &case.note {
                out.push_str(&format!("    - note: {note}\n"));
            }
        }
    }
    out.push_str(&format!(
        "Summary: {} contracts, {} tests: {} pass, {} fail, {} skip, {} error\n",
        report.total_contracts(),
        report.total_tests(),
        report.pass_count(),
        report.fail_count(),
        report.skip_count(),
        report.error_count()
    ));
    if report.mode == Mode::Static && report.skip_count() > 0 {
        out.push_str("Note: effect-only tests are skipped in static mode; use --mode effect with required allow flags.\n");
    }
    out
}

pub fn to_table(report: &RunReport) -> String {
    let mut out = String::new();
    out.push_str("CONTRACT_ID | STATUS | TESTS | SUMMARY\n");
    for contract in &report.contracts {
        let tests = report
            .cases
            .iter()
            .filter(|case| case.contract_id == contract.id)
            .count();
        out.push_str(&format!(
            "{} | {} | {} | {}\n",
            contract.id,
            contract.status.as_str(),
            tests,
            contract.title
        ));
    }
    out.push_str(&format!(
        "SUMMARY | {} | {} | {} contracts, {} pass, {} fail, {} skip, {} error\n",
        if report.exit_code() == 0 { "PASS" } else { "FAIL" },
        report.total_tests(),
        report.total_contracts(),
        report.pass_count(),
        report.fail_count(),
        report.skip_count(),
        report.error_count()
    ));
    out
}

pub fn to_json(report: &RunReport) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "group": report.domain.clone(),
        "domain": report.domain.clone(),
        "mode": report.mode.to_string(),
        "run_id": report.metadata.run_id,
        "commit_sha": report.metadata.commit_sha,
        "dirty_tree": report.metadata.dirty_tree,
        "summary": {
            "contracts": report.total_contracts(),
            "tests": report.total_tests(),
            "pass": report.pass_count(),
            "fail": report.fail_count(),
            "skip": report.skip_count(),
            "error": report.error_count(),
            "exit_code": report.exit_code(),
            "duration_ms": report.duration_ms
        },
        "maturity": maturity_score(&report.contracts),
        "contracts": report.contracts.iter().map(|c| serde_json::json!({
            "group": report.domain.clone(),
            "id": c.id,
            "contract_id": c.id,
            "title": c.title,
            "mode": c.mode.as_str(),
            "effects": c.effects.iter().map(|effect| effect.as_str()).collect::<Vec<_>>(),
            "status": c.status.as_str(),
            "duration_ms": c.duration_ms,
            "summary": c.title,
            "tests": report.cases.iter().filter(|t| t.contract_id == c.id).count(),
            "checks": report.cases.iter().filter(|t| t.contract_id == c.id).map(|t| serde_json::json!({
                "test_id": t.test_id,
                "status": t.status.as_str(),
                "duration_ms": t.duration_ms,
                "details": t.note,
                "violations": t.violations.iter().map(|v| serde_json::json!({
                    "file": v.file,
                    "line": v.line,
                    "message": v.message,
                    "evidence": v.evidence
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "tests": report.cases.iter().map(|t| serde_json::json!({
            "contract_id": t.contract_id,
            "contract_title": t.contract_title,
            "test_id": t.test_id,
            "test_title": t.test_title,
            "kind": format!("{:?}", t.kind).to_ascii_lowercase(),
            "status": t.status.as_str(),
            "duration_ms": t.duration_ms,
            "note": t.note,
            "violations": t.violations.iter().map(|v| serde_json::json!({
                "contract_id": v.contract_id,
                "test_id": v.test_id,
                "file": v.file,
                "line": v.line,
                "message": v.message,
                "evidence": v.evidence
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

pub fn to_json_all(reports: &[RunReport]) -> serde_json::Value {
    let contracts = reports.iter().map(RunReport::total_contracts).sum::<usize>();
    let tests = reports.iter().map(RunReport::total_tests).sum::<usize>();
    let pass = reports.iter().map(RunReport::pass_count).sum::<usize>();
    let fail = reports.iter().map(RunReport::fail_count).sum::<usize>();
    let skip = reports.iter().map(RunReport::skip_count).sum::<usize>();
    let error = reports.iter().map(RunReport::error_count).sum::<usize>();
    let exit_code = if error > 0 || fail > 0 { 1 } else { 0 };
    serde_json::json!({
        "schema_version": 1,
        "group": "all",
        "domain": "all",
        "run_id": reports.first().map(|report| report.metadata.run_id.clone()).unwrap_or_else(|| "local".to_string()),
        "commit_sha": reports.first().and_then(|report| report.metadata.commit_sha.clone()),
        "dirty_tree": reports.first().map(|report| report.metadata.dirty_tree).unwrap_or(false),
        "summary": {
            "contracts": contracts,
            "tests": tests,
            "pass": pass,
            "fail": fail,
            "skip": skip,
            "error": error,
            "exit_code": exit_code,
            "duration_ms": reports.iter().map(|report| report.duration_ms).sum::<u64>()
        },
        "maturity": serde_json::json!({
            "domains": reports.iter().map(|report| serde_json::json!({
                "domain": report.domain,
                "scores": maturity_score(&report.contracts)
            })).collect::<Vec<_>>()
        }),
        "domains": reports.iter().map(to_json).collect::<Vec<_>>()
    })
}

pub fn to_pretty_all(reports: &[RunReport]) -> String {
    let mut out = String::new();
    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&to_pretty(report));
    }
    let contracts = reports.iter().map(RunReport::total_contracts).sum::<usize>();
    let tests = reports.iter().map(RunReport::total_tests).sum::<usize>();
    let pass = reports.iter().map(RunReport::pass_count).sum::<usize>();
    let fail = reports.iter().map(RunReport::fail_count).sum::<usize>();
    let skip = reports.iter().map(RunReport::skip_count).sum::<usize>();
    let error = reports.iter().map(RunReport::error_count).sum::<usize>();
    out.push_str(&format!(
        "\nSummary: {} contracts, {} tests: {} pass, {} fail, {} skip, {} error\n",
        contracts, tests, pass, fail, skip, error
    ));
    out
}

pub fn to_table_all(reports: &[RunReport]) -> String {
    let mut out = String::new();
    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&format!("GROUP: {}\n", report.domain));
        out.push_str(&to_table(report));
    }
    let contracts = reports.iter().map(RunReport::total_contracts).sum::<usize>();
    let tests = reports.iter().map(RunReport::total_tests).sum::<usize>();
    let pass = reports.iter().map(RunReport::pass_count).sum::<usize>();
    let fail = reports.iter().map(RunReport::fail_count).sum::<usize>();
    let skip = reports.iter().map(RunReport::skip_count).sum::<usize>();
    let error = reports.iter().map(RunReport::error_count).sum::<usize>();
    out.push_str(&format!(
        "\nSUMMARY | {} | {} | {} contracts, {} pass, {} fail, {} skip, {} error\n",
        if fail == 0 && error == 0 { "PASS" } else { "FAIL" },
        tests,
        contracts,
        pass,
        fail,
        skip,
        error
    ));
    out
}

pub fn to_github(reports: &[RunReport]) -> String {
    let mut out = to_pretty_all(reports);
    for report in reports {
        for case in &report.cases {
            match case.status {
                CaseStatus::Fail => {
                    for violation in &case.violations {
                        let file = violation.file.clone().unwrap_or_default();
                        let line = violation.line.unwrap_or(1);
                        out.push_str(&format!(
                            "::error file={},line={},title={}::{}\n",
                            file, line, case.test_id, violation.message
                        ));
                    }
                }
                CaseStatus::Error => {
                    out.push_str(&format!(
                        "::error title={}::{}\n",
                        case.test_id,
                        case.note.clone().unwrap_or_else(|| "error".to_string())
                    ));
                }
                CaseStatus::Skip => {
                    out.push_str(&format!(
                        "::notice title={}::{}\n",
                        case.test_id,
                        case.note.clone().unwrap_or_else(|| "skipped".to_string())
                    ));
                }
                CaseStatus::Pass => {}
            }
        }
    }
    out
}

pub fn to_junit_all(reports: &[RunReport]) -> Result<String, String> {
    let tests = reports.iter().map(RunReport::total_tests).sum::<usize>();
    let failures = reports.iter().map(RunReport::fail_count).sum::<usize>();
    let errors = reports.iter().map(RunReport::error_count).sum::<usize>();
    let skipped = reports.iter().map(RunReport::skip_count).sum::<usize>();
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuites tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\">",
        tests, failures, errors, skipped
    ));
    for report in reports {
        let suite = to_junit(report)?;
        let start = suite
            .find("<testsuite")
            .ok_or_else(|| "invalid junit suite".to_string())?;
        let end = suite
            .rfind("</testsuite>")
            .ok_or_else(|| "invalid junit suite".to_string())?;
        out.push_str(&suite[start..end + "</testsuite>".len()]);
    }
    out.push_str("</testsuites>\n");
    Ok(out)
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn to_junit(report: &RunReport) -> Result<String, String> {
    let tests = report.total_tests();
    let failures = report.fail_count();
    let errors = report.error_count();
    let skipped = report.skip_count();
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuites><testsuite name=\"contracts.{}\" tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\">",
        xml_escape(&report.domain),
        tests,
        failures,
        errors,
        skipped
    ));
    for case in &report.cases {
        out.push_str(&format!(
            "<testcase classname=\"{}\" name=\"{}\">",
            xml_escape(&case.contract_id),
            xml_escape(&case.test_id)
        ));
        match case.status {
            CaseStatus::Pass => {}
            CaseStatus::Skip => {
                let note = case.note.as_deref().unwrap_or("skipped");
                out.push_str(&format!("<skipped message=\"{}\"/>", xml_escape(note)));
            }
            CaseStatus::Error => {
                let note = case.note.as_deref().unwrap_or("error");
                out.push_str(&format!(
                    "<error message=\"{}\">{}</error>",
                    xml_escape(note),
                    xml_escape(note)
                ));
            }
            CaseStatus::Fail => {
                let detail = case
                    .violations
                    .iter()
                    .map(|v| {
                        let location = match (&v.file, v.line) {
                            (Some(file), Some(line)) => format!("{file}:{line}"),
                            (Some(file), None) => file.clone(),
                            _ => "unknown-location".to_string(),
                        };
                        format!("{}: {}", location, v.message)
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                out.push_str(&format!(
                    "<failure message=\"{}\">{}</failure>",
                    xml_escape(&detail),
                    xml_escape(&detail)
                ));
            }
        }
        out.push_str("</testcase>");
    }
    out.push_str("</testsuite></testsuites>\n");
    Ok(out)
}
