// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) fn invalid_format(name: &str, value: String, message: String) -> RuntimeConfigError {
    RuntimeConfigError::InvalidFormat {
        name: name.to_string(),
        value,
        message,
    }
}

pub(super) fn env_bool(name: &str, default: bool) -> Result<bool, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    match value.as_str() {
        "1" | "true" | "TRUE" | "yes" | "YES" => Ok(true),
        "0" | "false" | "FALSE" | "no" | "NO" => Ok(false),
        _ => Err(invalid_format(
            name,
            value.clone(),
            format!(
                "invalid boolean value for {name}: {value} (expected one of 1/0/true/false/yes/no)"
            ),
        )),
    }
}

pub(super) fn env_u64(name: &str, default: u64) -> Result<u64, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    value.parse::<u64>().map_err(|err| {
        invalid_format(
            name,
            value.clone(),
            format!("invalid u64 value for {name}: {value} ({err})"),
        )
    })
}

pub(super) fn env_usize(name: &str, default: usize) -> Result<usize, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    value.parse::<usize>().map_err(|err| {
        invalid_format(
            name,
            value.clone(),
            format!("invalid usize value for {name}: {value} ({err})"),
        )
    })
}

pub(super) fn env_f64(name: &str, default: f64) -> Result<f64, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(default);
    };
    value.parse::<f64>().map_err(|err| {
        invalid_format(
            name,
            value.clone(),
            format!("invalid f64 value for {name}: {value} ({err})"),
        )
    })
}

pub(super) fn env_duration_ms(
    name: &str,
    default_ms: u64,
) -> Result<Duration, RuntimeConfigError> {
    Ok(Duration::from_millis(env_u64(name, default_ms)?))
}

pub(super) fn env_list(name: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn env_dataset_list(
    name: &str,
) -> Result<Vec<bijux_atlas_model::DatasetId>, RuntimeConfigError> {
    let Some(value) = std::env::var(name).ok() else {
        return Ok(Vec::new());
    };
    let mut datasets = Vec::new();
    for item in value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let parts: Vec<_> = item.split('/').collect();
        if parts.len() != 3 {
            return Err(invalid_format(
                name,
                value.clone(),
                format!(
                    "invalid dataset list entry for {name}: {item} (expected release/species/assembly)"
                ),
            ));
        }
        let dataset =
            bijux_atlas_model::DatasetId::new(parts[0], parts[1], parts[2]).map_err(|err| {
                invalid_format(
                    name,
                    value.clone(),
                    format!("invalid dataset list entry for {name}: {item} ({err})"),
                )
            })?;
        datasets.push(dataset);
    }
    Ok(datasets)
}

pub(super) fn env_map(name: &str) -> Result<HashMap<String, String>, RuntimeConfigError> {
    let Some(raw) = std::env::var(name).ok() else {
        return Ok(HashMap::new());
    };
    let mut entries = HashMap::new();
    for item in raw
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let (key, value) = item.split_once('=').ok_or_else(|| {
            invalid_format(
                name,
                raw.clone(),
                format!("invalid key=value entry for {name}: {item}"),
            )
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(invalid_format(
                name,
                raw.clone(),
                format!("invalid key=value entry for {name}: {item}"),
            ));
        }
        entries.insert(key.to_string(), value.to_string());
    }
    Ok(entries)
}

pub(super) fn validate_url(
    name: &str,
    value: &str,
    required: bool,
) -> Result<(), RuntimeConfigError> {
    if value.trim().is_empty() {
        if required {
            return Err(RuntimeConfigError::MissingRequiredEnv {
                name: name.to_string(),
                message: format!("{name} must not be empty"),
            });
        }
        return Ok(());
    }
    reqwest::Url::parse(value).map_err(|err| {
        invalid_format(
            name,
            value.to_string(),
            format!("invalid url value for {name}: {value} ({err})"),
        )
    })?;
    Ok(())
}

pub(super) fn parse_registry_source_specs(
    retry: &StoreRetryConfig,
) -> Result<Vec<RegistrySourceSpec>, RuntimeConfigError> {
    let raw = std::env::var("ATLAS_REGISTRY_SOURCES").unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    let signatures = env_map("ATLAS_REGISTRY_SIGNATURES")?;
    let ttl = env_u64("ATLAS_REGISTRY_TTL_MS", 15_000)?;
    let max_sources = env_usize("ATLAS_REGISTRY_MAX_SOURCES", 8)?;
    let mut sources = Vec::new();
    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let (name, spec) = piece.split_once('=').ok_or_else(|| {
            invalid_format(
                "ATLAS_REGISTRY_SOURCES",
                raw.clone(),
                format!("invalid ATLAS_REGISTRY_SOURCES entry: {piece}"),
            )
        })?;
        let name = name.trim();
        let spec = spec.trim();
        let (scheme, endpoint) = if let Some(path) = spec.strip_prefix("local:") {
            ("local".to_string(), path.to_string())
        } else if let Some(url) = spec.strip_prefix("s3:") {
            validate_url("ATLAS_REGISTRY_SOURCES", url, true)?;
            ("s3".to_string(), url.to_string())
        } else if let Some(url) = spec.strip_prefix("http:") {
            validate_url("ATLAS_REGISTRY_SOURCES", url, true)?;
            ("http".to_string(), url.to_string())
        } else {
            return Err(RuntimeConfigError::InvalidValue {
                message: format!(
                    "unsupported registry source scheme in {piece}; use local:/path, s3:https://..., or http:https://..."
                ),
            });
        };
        sources.push(RegistrySourceSpec {
            name: name.to_string(),
            scheme,
            endpoint,
            signature: signatures.get(name).cloned(),
            ttl_ms: ttl,
        });
    }
    if sources.len() > max_sources {
        return Err(RuntimeConfigError::InvalidValue {
            message: format!(
                "ATLAS_REGISTRY_SOURCES exceeds max allowed sources: {} > {}",
                sources.len(),
                max_sources
            ),
        });
    }
    let priority = std::env::var("ATLAS_REGISTRY_PRIORITY").unwrap_or_default();
    if !priority.trim().is_empty() {
        let mut by_name: HashMap<String, RegistrySourceSpec> = sources
            .into_iter()
            .map(|row| (row.name.clone(), row))
            .collect();
        let mut ordered = Vec::new();
        for name in priority.split(',').map(str::trim).filter(|x| !x.is_empty()) {
            if let Some(row) = by_name.remove(name) {
                ordered.push(row);
            }
        }
        let mut rest: Vec<RegistrySourceSpec> = by_name.into_values().collect();
        rest.sort_by(|a, b| a.name.cmp(&b.name));
        ordered.extend(rest);
        sources = ordered;
    }
    for row in &sources {
        if row.scheme != "local" {
            validate_url("ATLAS_REGISTRY_SOURCES", &row.endpoint, true)?;
        }
    }
    let _ = retry;
    Ok(sources)
}
