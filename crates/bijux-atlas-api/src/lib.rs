#![forbid(unsafe_code)]

use bijux_atlas_model::{DatasetId, LATEST_ALIAS_POLICY, NO_IMPLICIT_DEFAULT_DATASET_POLICY};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[cfg(feature = "allow-raw-genome-io")]
compile_error!(
    "Feature `allow-raw-genome-io` is forbidden in API layer: raw GFF3/FASTA reads are disallowed"
);

pub const CRATE_NAME: &str = "bijux-atlas-api";
pub const API_POLICY_LATEST_ALIAS: &str = LATEST_ALIAS_POLICY;
pub const API_POLICY_NO_IMPLICIT_DEFAULT_DATASET: &str = NO_IMPLICIT_DEFAULT_DATASET_POLICY;

#[must_use]
pub fn dataset_route_key(dataset: &DatasetId) -> String {
    format!(
        "release={}/species={}/assembly={}",
        dataset.release, dataset.species, dataset.assembly
    )
}

#[must_use]
pub fn openapi_v1_spec() -> Value {
    json!({
      "openapi": "3.0.3",
      "info": {
        "title": "bijux-atlas API",
        "version": "v1"
      },
      "paths": {
        "/healthz": {"get": {"responses": {"200": {"description": "ok"}}}},
        "/readyz": {"get": {"responses": {"200": {"description": "ready"}, "503": {"description": "not ready"}}}},
        "/metrics": {"get": {"responses": {"200": {"description": "prometheus metrics"}}}},
        "/v1/datasets": {
          "get": {
            "responses": {
              "200": {"description": "dataset list"},
              "304": {"description": "not modified"},
              "429": {"description": "rate limited", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/genes": {
          "get": {
            "parameters": [
              {"name": "release", "in": "query", "required": true, "schema": {"type": "string"}},
              {"name": "species", "in": "query", "required": true, "schema": {"type": "string"}},
              {"name": "assembly", "in": "query", "required": true, "schema": {"type": "string"}},
              {"name": "limit", "in": "query", "schema": {"type": "integer", "minimum": 1, "maximum": 500}},
              {"name": "cursor", "in": "query", "schema": {"type": "string"}},
              {"name": "fields", "in": "query", "schema": {"type": "string", "description": "comma-separated projection fields"}}
            ],
            "responses": {
              "200": {"description": "gene page"},
              "304": {"description": "not modified"},
              "400": {"description": "invalid query", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}},
              "413": {"description": "response too large", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}},
              "422": {"description": "query rejected by policy", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}},
              "429": {"description": "rate limited", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}},
              "503": {"description": "not ready / upstream unavailable", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/genes/count": {
          "get": {
            "responses": {
              "200": {"description": "count response"},
              "400": {"description": "invalid query", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/debug/datasets": {
          "get": {
            "responses": {
              "200": {"description": "debug cache inventory"},
              "404": {"description": "disabled"}
            }
          }
        }
      },
      "components": {
        "schemas": {
          "ApiErrorCode": {
            "type": "string",
            "enum": [
              "InvalidQueryParameter",
              "MissingDatasetDimension",
              "InvalidCursor",
              "QueryRejectedByPolicy",
              "RateLimited",
              "Timeout",
              "PayloadTooLarge",
              "ResponseTooLarge",
              "NotReady",
              "Internal"
            ]
          },
          "ApiError": {
            "type": "object",
            "required": ["code", "message", "details"],
            "additionalProperties": false,
            "properties": {
              "code": {"$ref": "#/components/schemas/ApiErrorCode"},
              "message": {"type": "string"},
              "details": {"type": "object"}
            }
          }
        }
      }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ApiErrorCode {
    InvalidQueryParameter,
    MissingDatasetDimension,
    InvalidCursor,
    QueryRejectedByPolicy,
    RateLimited,
    Timeout,
    PayloadTooLarge,
    ResponseTooLarge,
    NotReady,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
    pub details: Value,
}

impl ApiError {
    #[must_use]
    pub fn invalid_param(name: &str, value: &str) -> Self {
        Self {
            code: ApiErrorCode::InvalidQueryParameter,
            message: format!("invalid query parameter: {name}"),
            details: json!({"parameter": name, "value": value}),
        }
    }

    #[must_use]
    pub fn missing_dataset_dim(name: &str) -> Self {
        Self {
            code: ApiErrorCode::MissingDatasetDimension,
            message: format!("missing dataset dimension: {name}"),
            details: json!({"dimension": name}),
        }
    }
}

pub mod params {
    use super::ApiError;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    pub const ALLOWED_FIELDS: [&str; 6] = [
        "gene_id",
        "name",
        "coords",
        "biotype",
        "transcript_count",
        "sequence_length",
    ];

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ListGenesParams {
        pub release: String,
        pub species: String,
        pub assembly: String,
        pub limit: usize,
        pub cursor: Option<String>,
        pub gene_id: Option<String>,
        pub name: Option<String>,
        pub name_prefix: Option<String>,
        pub biotype: Option<String>,
        pub region: Option<String>,
        pub fields: Option<Vec<String>>,
        pub pretty: bool,
    }

    pub fn parse_list_genes_params(
        query: &BTreeMap<String, String>,
    ) -> Result<ListGenesParams, ApiError> {
        parse_list_genes_params_with_limit(query, 100, 500)
    }

    pub fn parse_list_genes_params_with_limit(
        query: &BTreeMap<String, String>,
        default_limit: usize,
        max_limit: usize,
    ) -> Result<ListGenesParams, ApiError> {
        let release = query
            .get("release")
            .cloned()
            .ok_or_else(|| ApiError::missing_dataset_dim("release"))?;
        let species = query
            .get("species")
            .cloned()
            .ok_or_else(|| ApiError::missing_dataset_dim("species"))?;
        let assembly = query
            .get("assembly")
            .cloned()
            .ok_or_else(|| ApiError::missing_dataset_dim("assembly"))?;

        let limit = if let Some(raw) = query.get("limit") {
            let value = raw
                .parse::<usize>()
                .map_err(|_| ApiError::invalid_param("limit", raw))?;
            if value == 0 || value > max_limit {
                return Err(ApiError::invalid_param("limit", raw));
            }
            value
        } else {
            default_limit
        };

        let fields = if let Some(raw_fields) = query.get("fields") {
            let mut out = Vec::new();
            let mut seen = BTreeSet::new();
            for raw in raw_fields.split(',') {
                let f = raw.trim();
                if f.is_empty() || !ALLOWED_FIELDS.contains(&f) {
                    return Err(ApiError::invalid_param("fields", raw_fields));
                }
                if seen.insert(f.to_string()) {
                    out.push(f.to_string());
                }
            }
            Some(out)
        } else {
            None
        };

        Ok(ListGenesParams {
            release,
            species,
            assembly,
            limit,
            cursor: query.get("cursor").cloned(),
            gene_id: query.get("gene_id").cloned(),
            name: query.get("name").cloned(),
            name_prefix: query.get("name_prefix").cloned(),
            biotype: query.get("biotype").cloned(),
            region: query.get("region").cloned(),
            fields,
            pretty: query
                .get("pretty")
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::params::parse_list_genes_params;
    use super::{ApiError, ApiErrorCode};
    use std::collections::BTreeMap;

    #[test]
    fn parse_params_success_exhaustive() {
        let mut q = BTreeMap::new();
        q.insert("release".to_string(), "110".to_string());
        q.insert("species".to_string(), "homo_sapiens".to_string());
        q.insert("assembly".to_string(), "GRCh38".to_string());
        q.insert("limit".to_string(), "42".to_string());
        q.insert("name_prefix".to_string(), "BR".to_string());

        let parsed = parse_list_genes_params(&q).expect("params parse");
        assert_eq!(parsed.limit, 42);
        assert_eq!(parsed.name_prefix.as_deref(), Some("BR"));
        assert!(!parsed.pretty);
    }

    #[test]
    fn parse_params_missing_dimensions() {
        let q = BTreeMap::new();
        let err = parse_list_genes_params(&q).expect_err("expected error");
        assert_eq!(err.code, ApiErrorCode::MissingDatasetDimension);
    }

    #[test]
    fn parse_params_invalid_limit() {
        let mut q = BTreeMap::new();
        q.insert("release".to_string(), "110".to_string());
        q.insert("species".to_string(), "homo_sapiens".to_string());
        q.insert("assembly".to_string(), "GRCh38".to_string());
        q.insert("limit".to_string(), "nope".to_string());

        let err = parse_list_genes_params(&q).expect_err("expected invalid limit");
        assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    }

    #[test]
    fn api_error_details_schema_stable() {
        let e = ApiError::invalid_param("limit", "nope");
        assert!(e.details.get("parameter").is_some());
        assert!(e.details.get("value").is_some());
    }

    #[test]
    fn parse_params_invalid_fields() {
        let mut q = BTreeMap::new();
        q.insert("release".to_string(), "110".to_string());
        q.insert("species".to_string(), "homo_sapiens".to_string());
        q.insert("assembly".to_string(), "GRCh38".to_string());
        q.insert("fields".to_string(), "name,nope".to_string());
        let err = parse_list_genes_params(&q).expect_err("invalid fields");
        assert_eq!(err.code, ApiErrorCode::InvalidQueryParameter);
    }
}
