// SPDX-License-Identifier: Apache-2.0

use crate::cli::{PerfCommand, PerfRunArgs, PerfValidateArgs};
use crate::{emit_payload, resolve_repo_root};
use reqwest::blocking::Client;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
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

fn percentile_ms(samples: &[f64], quantile: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let idx = (((samples.len() as f64) * quantile).ceil() as usize).saturating_sub(1);
    samples[idx.min(samples.len() - 1)]
}

fn start_fixture_server(expected_path: String, response_body: String) -> Result<String, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|err| format!("failed to bind perf server: {err}"))?;
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
    let slo_path = root.join("configs/perf/slo.yaml");
    let slo = read_yaml(&slo_path)?;
    let validate_ok = slo
        .get("schema_version")
        .and_then(serde_yaml::Value::as_i64)
        == Some(1)
        && slo
            .get("canonical_scenario")
            .and_then(serde_yaml::Value::as_str)
            .is_some()
        && slo.get("targets").is_some();

    let report_path = root.join("artifacts/perf/perf-slo.json");
    let report = serde_json::json!({
        "schema_version": 1,
        "status": if validate_ok { "ok" } else { "failed" },
        "slo_path": "configs/perf/slo.yaml",
        "contracts": {
            "PERF-SLO-001": validate_ok
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

    let scenario_path = root.join(format!("tools/perf/{}.json", args.scenario));
    let scenario = read_json(&scenario_path)?;
    let slo = read_yaml(&root.join("configs/perf/slo.yaml"))?;

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
        .ok_or_else(|| "scenario is missing requests_per_thread".to_string())? as usize;
    let warmup_requests = scenario
        .get("warmup_requests")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "scenario is missing warmup_requests".to_string())? as usize;
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
    let samples = Arc::new(Mutex::new(Vec::with_capacity(threads * requests_per_thread)));
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
    let max_p99 = slo
        .get("targets")
        .and_then(|value| value.get("latency"))
        .and_then(|value| value.get("p99_ms"))
        .and_then(serde_yaml::Value::as_f64)
        .ok_or_else(|| "SLO file is missing targets.latency.p99_ms".to_string())?;

    let report_valid = scenario
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        == Some(1)
        && !samples.is_empty();
    let perf_load_002 = p99 <= max_p99;

    let report = serde_json::json!({
        "schema_version": 1,
        "scenario": args.scenario,
        "seed": seed,
        "configuration": {
            "warmup_requests": warmup_requests,
            "threads": threads,
            "requests_per_thread": requests_per_thread,
            "request_path": expected_path
        },
        "latency_ms": {
            "p50": percentile_ms(&samples, 0.50),
            "p95": percentile_ms(&samples, 0.95),
            "p99": p99
        },
        "error_rate_percent": error_rate_percent,
        "throughput_rps": throughput_rps,
        "contracts": {
            "PERF-LOAD-001": report_valid,
            "PERF-LOAD-002": perf_load_002
        }
    });

    let report_path = root.join(format!("artifacts/perf/{}-load.json", args.scenario));
    write_json(&report_path, &report)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": if report_valid && perf_load_002 { "ok" } else { "failed" },
            "text": if report_valid && perf_load_002 { "perf run passed" } else { "perf run failed" },
            "rows": [{
                "report_path": format!("artifacts/perf/{}-load.json", args.scenario),
                "scenario_path": format!("tools/perf/{}.json", args.scenario),
                "contracts": report["contracts"].clone(),
                "latency_ms": report["latency_ms"].clone(),
                "throughput_rps": report["throughput_rps"].clone()
            }],
            "summary": {
                "total": 1,
                "errors": if report_valid && perf_load_002 { 0 } else { 1 },
                "warnings": 0
            }
        }),
    )?;
    Ok((rendered, if report_valid && perf_load_002 { 0 } else { 1 }))
}

pub(crate) fn run_perf_command(
    _quiet: bool,
    command: PerfCommand,
) -> Result<(String, i32), String> {
    match command {
        PerfCommand::Validate(args) => run_perf_validate(args),
        PerfCommand::Run(args) => run_perf(args),
    }
}
