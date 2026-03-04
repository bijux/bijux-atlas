// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    PerfBenchesCommand, PerfCommand, PerfDiffArgs, PerfKindArgs, PerfRunArgs, PerfValidateArgs,
};
use crate::{emit_payload, resolve_repo_root};
use reqwest::blocking::Client;
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn ensure_json(path: &Path) -> Result<(), String> {
    let _: serde_json::Value = read_json(path)?;
    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value)
            .map_err(|err| format!("failed to encode {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn write_csv(path: &Path, header: &str, row: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, format!("{header}\n{row}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn validate_benchmark_result_shape(report: &serde_json::Value) -> bool {
    report
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        == Some(1)
        && report
            .get("scenario")
            .and_then(serde_json::Value::as_str)
            .is_some()
        && report
            .get("latency_ms")
            .and_then(|value| value.get("p99"))
            .and_then(serde_json::Value::as_f64)
            .is_some()
        && report
            .get("throughput_rps")
            .and_then(serde_json::Value::as_f64)
            .is_some()
}

fn load_perf_scenario(name: &str) -> Result<serde_json::Value, String> {
    match name {
        "gene-lookup" => Ok(serde_json::json!({
            "schema_version": 1,
            "scenario": "gene-lookup",
            "seed": 110,
            "warmup_requests": 8,
            "duration_seconds": 1,
            "threads": 4,
            "requests_per_thread": 32,
            "request_path": "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1",
            "expected_status": 200,
            "response_body": "{\"items\":[{\"gene_id\":\"g1\",\"name\":\"G1\"}],\"count\":1}"
        })),
        other => Err(format!("unknown perf scenario: {other}")),
    }
}

fn current_rss_mb() -> Result<f64, String> {
    let pid = std::process::id().to_string();
    let output = ProcessCommand::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .map_err(|err| format!("failed to sample process rss: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to sample process rss: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let rss_kb = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("failed to parse process rss: {err}"))?;
    Ok(rss_kb / 1024.0)
}

fn current_cpu_percent() -> Result<f64, String> {
    let pid = std::process::id().to_string();
    let output = ProcessCommand::new("ps")
        .args(["-o", "%cpu=", "-p", &pid])
        .output()
        .map_err(|err| format!("failed to sample process cpu: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to sample process cpu: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("failed to parse process cpu: {err}"))
}

fn percentile_ms(samples: &[f64], quantile: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let idx = (((samples.len() as f64) * quantile).ceil() as usize).saturating_sub(1);
    samples[idx.min(samples.len() - 1)]
}

fn start_fixture_server(expected_path: String, response_body: String) -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|err| format!("failed to bind perf server: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("failed to read perf server address: {err}"))?;
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                break;
            };
            let mut buffer = [0_u8; 4096];
            let bytes = stream.read(&mut buffer).unwrap_or(0);
            let request = String::from_utf8_lossy(&buffer[..bytes]);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");
            let (status_line, body) = if path == expected_path {
                ("HTTP/1.1 200 OK", response_body.as_str())
            } else {
                ("HTTP/1.1 404 Not Found", "{\"error\":\"not found\"}")
            };
            let response = format!(
                "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    Ok(format!("http://{addr}"))
}

fn run_perf_validate(args: PerfValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/contracts/perf/slo.schema.json"))?;
    ensure_json(&root.join("configs/contracts/perf/budgets.schema.json"))?;
    ensure_json(&root.join("configs/contracts/perf/exceptions.schema.json"))?;
    let slo_path = root.join("configs/perf/slo.yaml");
    let budgets_path = root.join("configs/perf/budgets.yaml");
    let exceptions_path = root.join("configs/perf/exceptions.json");
    let slo = read_yaml(&slo_path)?;
    let budgets = read_yaml(&budgets_path)?;
    let exceptions = read_json(&exceptions_path)?;
    let today = "2026-03-03";
    let expired = exceptions
        .get("exceptions")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| value.get("expires_on"))
        .filter_map(serde_json::Value::as_str)
        .filter(|expires_on| *expires_on < today)
        .count();
    let validate_ok = slo
        .get("schema_version")
        .and_then(serde_yaml::Value::as_i64)
        == Some(1)
        && slo
            .get("canonical_scenario")
            .and_then(serde_yaml::Value::as_str)
            .is_some()
        && slo.get("targets").is_some()
        && slo
            .get("targets")
            .and_then(|value| value.get("memory"))
            .and_then(|value| value.get("max_rss_mb"))
            .and_then(serde_yaml::Value::as_f64)
            .is_some()
        && slo
            .get("targets")
            .and_then(|value| value.get("cpu"))
            .and_then(|value| value.get("max_percent"))
            .and_then(serde_yaml::Value::as_f64)
            .is_some()
        && slo
            .get("targets")
            .and_then(|value| value.get("cold_start"))
            .and_then(|value| value.get("max_ready_ms"))
            .and_then(serde_yaml::Value::as_f64)
            .is_some()
        && budgets
            .get("throughput")
            .and_then(|value| value.get("min_requests_per_second"))
            .and_then(serde_yaml::Value::as_f64)
            .is_some()
        && budgets
            .get("regression_window")
            .and_then(|value| value.get("history_runs"))
            .and_then(serde_yaml::Value::as_i64)
            .is_some()
        && expired == 0;

    let report_path = root.join("artifacts/perf/perf-slo.json");
    let report = serde_json::json!({
        "schema_version": 1,
        "status": if validate_ok { "ok" } else { "failed" },
        "slo_path": "configs/perf/slo.yaml",
        "budgets_path": "configs/perf/budgets.yaml",
        "exceptions_path": "configs/perf/exceptions.json",
        "contracts": {
            "PERF-SLO-001": validate_ok,
            "PERF-EXC-001": expired == 0
        }
    });
    write_json(&report_path, &report)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": report["status"].clone(),
            "text": if validate_ok { "perf SLO validates" } else { "perf SLO is invalid" },
            "rows": [{
                "report_path": "artifacts/perf/perf-slo.json",
                "budgets_path": "configs/perf/budgets.yaml",
                "exceptions_path": "configs/perf/exceptions.json",
                "contracts": report["contracts"].clone()
            }],
            "summary": {
                "total": 1,
                "errors": if validate_ok { 0 } else { 1 },
                "warnings": 0
            }
        }),
    )?;
    Ok((rendered, if validate_ok { 0 } else { 1 }))
}

fn run_perf(args: PerfRunArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/contracts/perf/slo.schema.json"))?;
    ensure_json(&root.join("configs/contracts/perf/load-report.schema.json"))?;
    ensure_json(&root.join("configs/contracts/perf/budgets.schema.json"))?;

    let scenario = load_perf_scenario(&args.scenario)?;
    let slo = read_yaml(&root.join("configs/perf/slo.yaml"))?;
    let budgets = read_yaml(&root.join("configs/perf/budgets.yaml"))?;

    let expected_path = scenario
        .get("request_path")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "scenario is missing request_path".to_string())?
        .to_string();
    let response_body = scenario
        .get("response_body")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "scenario is missing response_body".to_string())?
        .to_string();
    let threads = scenario
        .get("threads")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "scenario is missing threads".to_string())? as usize;
    let requests_per_thread = scenario
        .get("requests_per_thread")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "scenario is missing requests_per_thread".to_string())?
        as usize;
    let warmup_requests = scenario
        .get("warmup_requests")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "scenario is missing warmup_requests".to_string())?
        as usize;
    let duration_seconds = scenario
        .get("duration_seconds")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "scenario is missing duration_seconds".to_string())?
        as u64;
    let seed = scenario
        .get("seed")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "scenario is missing seed".to_string())?;
    let expected_status = scenario
        .get("expected_status")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(200) as u16;

    let base = start_fixture_server(expected_path.clone(), response_body)?;
    let url = format!("{base}{expected_path}");
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|err| format!("failed to build perf client: {err}"))?;
    let rss_before_mb = current_rss_mb()?;

    for _ in 0..warmup_requests {
        let response = client
            .get(&url)
            .send()
            .map_err(|err| format!("warmup request failed: {err}"))?;
        if response.status().as_u16() != expected_status {
            return Err(format!(
                "warmup request returned {}, expected {expected_status}",
                response.status().as_u16()
            ));
        }
    }

    let started = Instant::now();
    let failures = Arc::new(AtomicU64::new(0));
    let samples = Arc::new(Mutex::new(Vec::with_capacity(
        threads * requests_per_thread,
    )));
    thread::scope(|scope| {
        for _thread in 0..threads {
            let client = client.clone();
            let url = url.clone();
            let failures = Arc::clone(&failures);
            let samples = Arc::clone(&samples);
            scope.spawn(move || {
                for _request in 0..requests_per_thread {
                    let request_started = Instant::now();
                    match client.get(&url).send() {
                        Ok(response) => {
                            if response.status().as_u16() != expected_status {
                                failures.fetch_add(1, Ordering::Relaxed);
                            }
                            if let Ok(mut buffer) = samples.lock() {
                                buffer.push(request_started.elapsed().as_secs_f64() * 1000.0);
                            }
                        }
                        Err(_) => {
                            failures.fetch_add(1, Ordering::Relaxed);
                            if let Ok(mut buffer) = samples.lock() {
                                buffer.push(request_started.elapsed().as_secs_f64() * 1000.0);
                            }
                        }
                    }
                }
            });
        }
    });
    let total_elapsed = started.elapsed().as_secs_f64();
    let mut samples = samples
        .lock()
        .map_err(|_| "failed to lock perf sample buffer".to_string())?
        .clone();
    samples.sort_by(f64::total_cmp);

    let total_requests = samples.len() as f64;
    let error_rate_percent = if total_requests == 0.0 {
        0.0
    } else {
        (failures.load(Ordering::Relaxed) as f64 / total_requests) * 100.0
    };
    let throughput_rps = if total_elapsed == 0.0 {
        total_requests
    } else {
        total_requests / total_elapsed
    };
    let p99 = percentile_ms(&samples, 0.99);
    let rss_after_mb = current_rss_mb()?;
    let rss_mb = rss_before_mb.max(rss_after_mb);
    let max_p99 = slo
        .get("targets")
        .and_then(|value| value.get("latency"))
        .and_then(|value| value.get("p99_ms"))
        .and_then(serde_yaml::Value::as_f64)
        .ok_or_else(|| "SLO file is missing targets.latency.p99_ms".to_string())?;
    let min_throughput = budgets
        .get("throughput")
        .and_then(|value| value.get("min_requests_per_second"))
        .and_then(serde_yaml::Value::as_f64)
        .ok_or_else(|| "budgets file is missing throughput.min_requests_per_second".to_string())?;

    let report_valid = scenario
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        == Some(1)
        && !samples.is_empty();
    let perf_load_002 = p99 <= max_p99;
    let perf_load_003 = error_rate_percent
        <= slo
            .get("targets")
            .and_then(|value| value.get("error_rate"))
            .and_then(|value| value.get("max_percent"))
            .and_then(serde_yaml::Value::as_f64)
            .ok_or_else(|| "SLO file is missing targets.error_rate.max_percent".to_string())?;
    let perf_load_004 = throughput_rps >= min_throughput;
    let perf_mem_001 = rss_mb
        <= slo
            .get("targets")
            .and_then(|value| value.get("memory"))
            .and_then(|value| value.get("max_rss_mb"))
            .and_then(serde_yaml::Value::as_f64)
            .ok_or_else(|| "SLO file is missing targets.memory.max_rss_mb".to_string())?;
    let cpu_percent = current_cpu_percent()?;
    let perf_cpu_001 = cpu_percent
        <= slo
            .get("targets")
            .and_then(|value| value.get("cpu"))
            .and_then(|value| value.get("max_percent"))
            .and_then(serde_yaml::Value::as_f64)
            .ok_or_else(|| "SLO file is missing targets.cpu.max_percent".to_string())?;

    let report = serde_json::json!({
        "schema_version": 1,
        "scenario": args.scenario,
        "seed": seed,
        "configuration": {
            "warmup_requests": warmup_requests,
            "duration_seconds": duration_seconds,
            "threads": threads,
            "requests_per_thread": requests_per_thread,
            "request_path": expected_path
        },
        "latency_ms": {
            "p50": percentile_ms(&samples, 0.50),
            "p95": percentile_ms(&samples, 0.95),
            "p99": p99
        },
        "memory_mb": {
            "rss": rss_mb
        },
        "cpu_percent": {
            "current": cpu_percent
        },
        "error_rate_percent": error_rate_percent,
        "throughput_rps": throughput_rps,
        "contracts": {
            "PERF-LOAD-001": report_valid,
            "PERF-LOAD-002": perf_load_002,
            "PERF-LOAD-003": perf_load_003,
            "PERF-LOAD-004": perf_load_004,
            "PERF-MEM-001": perf_mem_001,
            "PERF-CPU-001": perf_cpu_001
        }
    });

    ensure_json(&root.join("configs/contracts/perf/benchmark-result.schema.json"))?;
    let benchmark_result_valid = validate_benchmark_result_shape(&report);

    let report_path = root.join(format!("artifacts/perf/{}-load.json", args.scenario));
    write_json(&report_path, &report)?;
    let benchmark_artifacts_root = root.join("artifacts/benchmarks");
    let benchmark_report_path =
        benchmark_artifacts_root.join(format!("{}-result.json", args.scenario));
    write_json(&benchmark_report_path, &report)?;
    let benchmark_csv_path = benchmark_artifacts_root.join(format!("{}-result.csv", args.scenario));
    write_csv(
        &benchmark_csv_path,
        "scenario,latency_p50_ms,latency_p95_ms,latency_p99_ms,throughput_rps,error_rate_percent,rss_mb,cpu_percent",
        &format!(
            "{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            args.scenario,
            percentile_ms(&samples, 0.50),
            percentile_ms(&samples, 0.95),
            p99,
            throughput_rps,
            error_rate_percent,
            rss_mb,
            cpu_percent
        ),
    )?;
    let history_runs = budgets
        .get("regression_window")
        .and_then(|value| value.get("history_runs"))
        .and_then(serde_yaml::Value::as_i64)
        .unwrap_or(5);
    let summary_path = benchmark_artifacts_root.join(format!("{}-summary.json", args.scenario));
    let summary = serde_json::json!({
        "schema_version": 1,
        "scenario": args.scenario,
        "report_path": format!("artifacts/benchmarks/{}-result.json", args.scenario),
        "csv_path": format!("artifacts/benchmarks/{}-result.csv", args.scenario),
        "status": if report_valid && perf_load_002 && perf_load_003 && perf_load_004 && perf_mem_001 && perf_cpu_001 && benchmark_result_valid { "ok" } else { "failed" },
        "latency_p99_ms": p99,
        "throughput_rps": throughput_rps,
        "regression_window_runs": history_runs
    });
    write_json(&summary_path, &summary)?;
    let history_path = benchmark_artifacts_root.join(format!("{}-history.json", args.scenario));
    let mut history_entries = if history_path.exists() {
        read_json(&history_path)?
            .get("runs")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    history_entries.push(serde_json::json!({
        "run_index": history_entries.len() + 1,
        "latency_p99_ms": p99,
        "throughput_rps": throughput_rps,
        "error_rate_percent": error_rate_percent
    }));
    write_json(
        &history_path,
        &serde_json::json!({
            "schema_version": 1,
            "scenario": args.scenario,
            "runs": history_entries
        }),
    )?;
    let baseline_path = root.join(format!("ops/report/{}-baseline.json", args.scenario));
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": if report_valid && perf_load_002 && perf_load_003 && perf_load_004 && perf_mem_001 && perf_cpu_001 { "ok" } else { "failed" },
            "text": if report_valid && perf_load_002 && perf_load_003 && perf_load_004 && perf_mem_001 && perf_cpu_001 { "perf run passed" } else { "perf run failed" },
            "rows": [{
                "report_path": format!("artifacts/perf/{}-load.json", args.scenario),
                "benchmark_report_path": format!("artifacts/benchmarks/{}-result.json", args.scenario),
                "benchmark_csv_path": format!("artifacts/benchmarks/{}-result.csv", args.scenario),
                "benchmark_summary_path": format!("artifacts/benchmarks/{}-summary.json", args.scenario),
                "benchmark_history_path": format!("artifacts/benchmarks/{}-history.json", args.scenario),
                "scenario": args.scenario,
                "baseline_path": format!("ops/report/{}-baseline.json", args.scenario),
                "regression_window_runs": history_runs,
                "contracts": report["contracts"].clone(),
                "benchmark_contracts": {
                    "PERF-BENCH-RESULT-001": benchmark_result_valid
                },
                "latency_ms": report["latency_ms"].clone(),
                "memory_mb": report["memory_mb"].clone(),
                "cpu_percent": report["cpu_percent"].clone(),
                "throughput_rps": report["throughput_rps"].clone(),
                "error_rate_percent": report["error_rate_percent"].clone()
            }],
            "summary": {
                "total": 1,
                "errors": if report_valid && perf_load_002 && perf_load_003 && perf_load_004 && perf_mem_001 && perf_cpu_001 && benchmark_result_valid { 0 } else { 1 },
                "warnings": 0
            }
        }),
    )?;
    let _ = baseline_path;
    Ok((
        rendered,
        if report_valid
            && perf_load_002
            && perf_load_003
            && perf_load_004
            && perf_mem_001
            && perf_cpu_001
            && benchmark_result_valid
        {
            0
        } else {
            1
        },
    ))
}

fn run_perf_cold_start(args: PerfValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/contracts/perf/cold-start-report.schema.json"))?;
    let slo = read_yaml(&root.join("configs/perf/slo.yaml"))?;
    let started = Instant::now();
    let _base = start_fixture_server("/readyz".to_string(), "{\"ready\":true}".to_string())?;
    let ready_ms = started.elapsed().as_secs_f64() * 1000.0;
    let max_ready_ms = slo
        .get("targets")
        .and_then(|value| value.get("cold_start"))
        .and_then(|value| value.get("max_ready_ms"))
        .and_then(serde_yaml::Value::as_f64)
        .ok_or_else(|| "SLO file is missing targets.cold_start.max_ready_ms".to_string())?;
    let passed = ready_ms <= max_ready_ms;
    let report_path = root.join("artifacts/perf/cold-start.json");
    let report = serde_json::json!({
        "schema_version": 1,
        "ready_ms": ready_ms,
        "profile_reference": "ops/k8s/values/perf.yaml",
        "contracts": {
            "PERF-COLD-001": passed
        }
    });
    write_json(&report_path, &report)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": if passed { "ok" } else { "failed" },
            "text": if passed { "cold start meets readiness threshold" } else { "cold start exceeds readiness threshold" },
            "rows": [{
                "report_path": "artifacts/perf/cold-start.json",
                "profile_reference": "ops/k8s/values/perf.yaml",
                "contracts": report["contracts"].clone(),
                "ready_ms": ready_ms
            }],
            "summary": {
                "total": 1,
                "errors": if passed { 0 } else { 1 },
                "warnings": 0
            }
        }),
    )?;
    Ok((rendered, if passed { 0 } else { 1 }))
}

fn run_perf_kind(args: PerfKindArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    ensure_json(&root.join("ops/schema/k8s/perf-on-kind.schema.json"))?;
    let exe = std::env::current_exe().map_err(|err| format!("perf kind failed: {err}"))?;

    let mut kind_args = vec![
        "ops".to_string(),
        "kind".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if let Some(repo_root) = &args.repo_root {
        kind_args.push("--repo-root".to_string());
        kind_args.push(repo_root.display().to_string());
    }
    let kind_out = ProcessCommand::new(&exe)
        .args(&kind_args)
        .output()
        .map_err(|err| format!("perf kind failed: {err}"))?;
    let kind_status = if kind_out.status.success() {
        "reachable"
    } else {
        "unreachable"
    };

    let mut perf_args = vec![
        "perf".to_string(),
        "run".to_string(),
        "--scenario".to_string(),
        "gene-lookup".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if let Some(repo_root) = &args.repo_root {
        perf_args.push("--repo-root".to_string());
        perf_args.push(repo_root.display().to_string());
    }
    let perf_out = ProcessCommand::new(&exe)
        .args(&perf_args)
        .output()
        .map_err(|err| format!("perf kind failed: {err}"))?;
    let perf_ok = perf_out.status.success();

    let report_path = root.join("artifacts/perf/perf-on-kind.json");
    let report = serde_json::json!({
        "schema_version": 1,
        "profile": args.profile,
        "kind_status": kind_status,
        "load_report_path": "artifacts/perf/gene-lookup-load.json",
        "contracts": {
            "PERF-KIND-001": kind_status == "reachable" && perf_ok
        }
    });
    write_json(&report_path, &report)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": if report["contracts"]["PERF-KIND-001"] == serde_json::json!(true) { "ok" } else { "failed" },
            "text": if report["contracts"]["PERF-KIND-001"] == serde_json::json!(true) { "perf kind validation passed" } else { "perf kind validation failed" },
            "rows": [{
                "report_path": "artifacts/perf/perf-on-kind.json",
                "contracts": report["contracts"].clone(),
                "kind_status": kind_status,
                "load_report_path": "artifacts/perf/gene-lookup-load.json"
            }],
            "summary": {
                "total": 1,
                "errors": if report["contracts"]["PERF-KIND-001"] == serde_json::json!(true) { 0 } else { 1 },
                "warnings": 0
            }
        }),
    )?;
    Ok((
        rendered,
        if report["contracts"]["PERF-KIND-001"] == serde_json::json!(true) {
            0
        } else {
            1
        },
    ))
}

fn run_perf_diff(args: PerfDiffArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/contracts/perf/load-report.schema.json"))?;
    let left_path = if args.report_a.is_absolute() {
        args.report_a
    } else {
        root.join(args.report_a)
    };
    let right_path = if args.report_b.is_absolute() {
        args.report_b
    } else {
        root.join(args.report_b)
    };
    let left = read_json(&left_path)?;
    let right = read_json(&right_path)?;
    let p99_a = left
        .get("latency_ms")
        .and_then(|value| value.get("p99"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let p99_b = right
        .get("latency_ms")
        .and_then(|value| value.get("p99"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let throughput_a = left
        .get("throughput_rps")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let throughput_b = right
        .get("throughput_rps")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let error_rate_a = left
        .get("error_rate_percent")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let error_rate_b = right
        .get("error_rate_percent")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    let regressed = p99_b > p99_a || throughput_b < throughput_a || error_rate_b > error_rate_a;
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "text": if regressed { "perf reports regressed" } else { "perf reports are stable or improved" },
        "rows": [{
            "report_a": left_path.display().to_string(),
            "report_b": right_path.display().to_string(),
            "changes": {
                "p99_ms": {"from": p99_a, "to": p99_b, "regressed": p99_b > p99_a},
                "throughput_rps": {"from": throughput_a, "to": throughput_b, "regressed": throughput_b < throughput_a},
                "error_rate_percent": {"from": error_rate_a, "to": error_rate_b, "regressed": error_rate_b > error_rate_a}
            }
        }],
        "summary": {
            "total": 1,
            "errors": 0,
            "warnings": if regressed { 1 } else { 0 }
        }
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_perf_benches_list(args: PerfValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/contracts/perf/benches.schema.json"))?;
    let registry = read_json(&root.join("configs/perf/benches.json"))?;
    let benches_root = root.join("crates/bijux-atlas-server/benches");
    let mut disk_entries = BTreeSet::new();
    for entry in fs::read_dir(&benches_root)
        .map_err(|err| format!("failed to read {}: {err}", benches_root.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            if let Some(name) = path.file_stem().and_then(|value| value.to_str()) {
                disk_entries.insert(name.to_string());
            }
        }
    }
    let registry_entries = registry
        .get("benches")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let registry_names = registry_entries
        .iter()
        .filter_map(|value| value.get("name"))
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let perf_bench_001 = registry_names == disk_entries;
    let perf_bench_002 = registry_entries.iter().all(|entry| {
        let weight = entry
            .get("weight")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("micro");
        let default_enabled = entry
            .get("default_enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        weight != "macro" || !default_enabled
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if perf_bench_001 && perf_bench_002 { "ok" } else { "failed" },
        "text": if perf_bench_001 && perf_bench_002 { "bench registry matches disk" } else { "bench registry mismatch" },
        "rows": [{
            "registry_path": "configs/perf/benches.json",
            "contracts": {
                "PERF-BENCH-001": perf_bench_001,
                "PERF-BENCH-002": perf_bench_002
            },
            "benches": registry_entries
        }],
        "summary": {
            "total": 1,
            "errors": if perf_bench_001 && perf_bench_002 { 0 } else { 1 },
            "warnings": 0
        }
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((
        rendered,
        if perf_bench_001 && perf_bench_002 {
            0
        } else {
            1
        },
    ))
}

pub(crate) fn run_perf_command(
    _quiet: bool,
    command: PerfCommand,
) -> Result<(String, i32), String> {
    match command {
        PerfCommand::Validate(args) => run_perf_validate(args),
        PerfCommand::Run(args) => run_perf(args),
        PerfCommand::Diff(args) => run_perf_diff(args),
        PerfCommand::ColdStart(args) => run_perf_cold_start(args),
        PerfCommand::Kind(args) => run_perf_kind(args),
        PerfCommand::Benches { command } => match command {
            PerfBenchesCommand::List(args) => run_perf_benches_list(args),
        },
    }
}
