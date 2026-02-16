use serde_json::{json, Value};

#[must_use]
pub fn openapi_v1_spec() -> Value {
    json!({
      "openapi": "3.0.3",
      "info": {
        "title": "bijux-atlas API",
        "version": "v1"
      },
      "paths": {
        "/healthz": {
          "get": {
            "responses": {"200": {"description": "ok"}}
          }
        },
        "/readyz": {
          "get": {
            "responses": {
              "200": {"description": "ready"},
              "503": {"description": "not ready", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
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
              {"name": "gene_id", "in": "query", "schema": {"type": "string"}},
              {"name": "name", "in": "query", "schema": {"type": "string"}},
              {"name": "name_prefix", "in": "query", "schema": {"type": "string"}},
              {"name": "biotype", "in": "query", "schema": {"type": "string"}},
              {"name": "region", "in": "query", "schema": {"type": "string", "pattern": "^[^:]+:[0-9]+-[0-9]+$"}},
              {"name": "limit", "in": "query", "schema": {"type": "integer", "minimum": 1, "maximum": 500}},
              {"name": "cursor", "in": "query", "schema": {"type": "string", "maxLength": 4096}},
              {"name": "fields", "in": "query", "schema": {"type": "string", "description": "comma-separated projection fields"}},
              {"name": "pretty", "in": "query", "schema": {"type": "boolean"}}
            ],
            "responses": {
              "200": {
                "description": "gene page",
                "content": {
                  "application/json": {
                    "examples": {
                      "ok": {
                        "value": {
                          "rows": [{"gene_id": "ENSG000001", "name": "BRCA1"}],
                          "next_cursor": "v1.opaque.cursor"
                        }
                      }
                    }
                  }
                }
              },
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
              "details": {"type": "object", "additionalProperties": true}
            },
            "examples": {
              "missingDataset": {
                "value": {
                  "code": "MissingDatasetDimension",
                  "message": "missing dataset dimension: release",
                  "details": {"dimension": "release"}
                }
              },
              "invalidCursor": {
                "value": {
                  "code": "InvalidCursor",
                  "message": "invalid cursor",
                  "details": {"cursor": "bad.cursor"}
                }
              }
            }
          }
        }
      }
    })
}
