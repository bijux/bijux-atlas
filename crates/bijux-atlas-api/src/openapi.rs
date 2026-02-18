use serde_json::{json, Value};

#[must_use]
pub fn openapi_v1_spec() -> Value {
    let error_codes = crate::generated::error_codes::API_ERROR_CODES;
    json!({
      "openapi": "3.0.3",
      "info": {
        "title": "bijux-atlas API",
        "version": "v1",
        "x-api-contract-version": "v1"
      },
      "paths": {
        "/healthz": {
          "get": {
            "responses": {"200": {"description": "ok"}}
          }
        },
        "/healthz/overload": {
          "get": {
            "responses": {"200": {"description": "overload status"}}
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
        "/v1/openapi.json": {
          "get": {
            "responses": {"200": {"description": "generated OpenAPI v1 spec"}}
          }
        },
        "/v1/version": {
          "get": {
            "responses": {"200": {"description": "plugin and service version metadata"}}
          }
        },
        "/v1/datasets": {
          "get": {
            "parameters": [
              {"name": "include_bom", "in": "query", "schema": {"type": "boolean"}}
            ],
            "responses": {
              "200": {"description": "dataset list"},
              "304": {"description": "not modified"},
              "429": {"description": "rate limited", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/releases/{release}/species/{species}/assemblies/{assembly}": {
          "get": {
            "parameters": [
              {"name": "release", "in": "path", "required": true, "schema": {"type": "string"}},
              {"name": "species", "in": "path", "required": true, "schema": {"type": "string"}},
              {"name": "assembly", "in": "path", "required": true, "schema": {"type": "string"}},
              {"name": "include_bom", "in": "query", "schema": {"type": "boolean"}}
            ],
            "responses": {
              "200": {"description": "dataset metadata and qc summary"},
              "400": {"description": "invalid dataset dimensions", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}},
              "404": {"description": "dataset missing in catalog", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}},
              "503": {"description": "manifest unavailable", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
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
              {"name": "name_like", "in": "query", "schema": {"type": "string"}},
              {"name": "biotype", "in": "query", "schema": {"type": "string"}},
              {"name": "contig", "in": "query", "schema": {"type": "string"}},
              {"name": "range", "in": "query", "schema": {"type": "string", "pattern": "^[^:]+:[0-9]+-[0-9]+$"}},
              {"name": "min_transcripts", "in": "query", "schema": {"type": "integer", "minimum": 0}},
              {"name": "max_transcripts", "in": "query", "schema": {"type": "integer", "minimum": 0}},
              {"name": "sort", "in": "query", "schema": {"type": "string", "enum": ["gene_id:asc", "region:asc"]}},
              {"name": "limit", "in": "query", "schema": {"type": "integer", "minimum": 1, "maximum": 500}},
              {"name": "cursor", "in": "query", "schema": {"type": "string", "maxLength": 4096}},
              {"name": "include", "in": "query", "schema": {
                "type": "string",
                "description": "comma-separated include flags; base response is minimal (gene_id,name)",
                "anyOf": [
                  {"type": "string", "enum": ["coords"]},
                  {"type": "string", "enum": ["biotype"]},
                  {"type": "string", "enum": ["counts"]},
                  {"type": "string", "enum": ["length"]}
                ]
              }},
              {"name": "pretty", "in": "query", "schema": {"type": "boolean"}},
              {"name": "explain", "in": "query", "schema": {"type": "boolean", "description": "embed extraction policy details"}}
            ],
            "responses": {
              "200": {
                "description": "gene page",
                "content": {
                  "application/json": {
                    "examples": {
                      "ok": {
                        "value": {
                          "api_version": "v1",
                          "contract_version": "v1",
                          "dataset": {"release":"110","species":"homo_sapiens","assembly":"GRCh38"},
                          "page": {"next_cursor":"v1.opaque.cursor"},
                          "data": {"rows": [{"gene_id": "ENSG000001", "name": "BRCA1"}]},
                          "links": {"next_cursor":"v1.opaque.cursor"}
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
            "deprecated": true,
            "responses": {
              "200": {"description": "count response"},
              "400": {"description": "invalid query", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/query/validate": {
          "post": {
            "responses": {
              "200": {"description": "query classification and cost-only validation"},
              "400": {"description": "invalid query", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/diff/genes": {
          "get": {
            "parameters": [
              {"name":"from_release","in":"query","required":true,"schema":{"type":"string","description":"explicit release number or literal latest alias"}},
              {"name":"to_release","in":"query","required":true,"schema":{"type":"string","description":"explicit release number or literal latest alias"}},
              {"name":"species","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"assembly","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"limit","in":"query","schema":{"type":"integer","minimum":1,"maximum":500}},
              {"name":"cursor","in":"query","schema":{"type":"string"}}
            ],
            "responses": {
              "200": {
                "description":"gene-level cross-release diff page",
                "content":{"application/json":{"examples":{"ok":{"value":{"diff":{"from_release":"110","to_release":"111","species":"homo_sapiens","assembly":"GRCh38","scope":"genes","rows":[{"gene_id":"gA","status":"removed"},{"gene_id":"gB","status":"changed"},{"gene_id":"gC","status":"added"}]}}}}}}
              },
              "304": {"description":"not modified"},
              "400": {"description":"invalid query/cursor", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/diff/region": {
          "get": {
            "parameters": [
              {"name":"from_release","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"to_release","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"species","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"assembly","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"region","in":"query","required":true,"schema":{"type":"string","pattern":"^[^:]+:[0-9]+-[0-9]+$"}},
              {"name":"limit","in":"query","schema":{"type":"integer","minimum":1,"maximum":500}},
              {"name":"cursor","in":"query","schema":{"type":"string"}}
            ],
            "responses": {
              "200": {"description":"region-scoped cross-release diff page"},
              "304": {"description":"not modified"},
              "400": {"description":"invalid query/cursor", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/sequence/region": {
          "get": {
            "parameters": [
              {"name":"release","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"species","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"assembly","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"region","in":"query","required":true,"schema":{"type":"string","pattern":"^[^:]+:[0-9]+-[0-9]+$"}},
              {"name":"include_stats","in":"query","schema":{"type":"boolean"}}
            ],
            "responses": {
              "200": {"description":"sequence payload"},
              "304": {"description":"not modified"},
              "400": {"description":"invalid query", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "401": {"description":"api key required for large sequence request", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "422": {"description":"region policy rejection", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "429": {"description":"rate limited", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/genes/{gene_id}/sequence": {
          "get": {
            "parameters": [
              {"name":"gene_id","in":"path","required":true,"schema":{"type":"string"}},
              {"name":"release","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"species","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"assembly","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"flank","in":"query","schema":{"type":"integer","minimum":0}},
              {"name":"include_stats","in":"query","schema":{"type":"boolean"}}
            ],
            "responses": {
              "200": {"description":"gene sequence payload"},
              "404": {"description":"gene not found", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "422": {"description":"region policy rejection", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/genes/{gene_id}/transcripts": {
          "get": {
            "parameters": [
              {"name":"gene_id","in":"path","required":true,"schema":{"type":"string"}},
              {"name":"release","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"species","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"assembly","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"limit","in":"query","schema":{"type":"integer","minimum":1,"maximum":500}},
              {"name":"cursor","in":"query","schema":{"type":"string"}},
              {"name":"biotype","in":"query","schema":{"type":"string"}},
              {"name":"type","in":"query","schema":{"type":"string"}},
              {"name":"region","in":"query","schema":{"type":"string","pattern":"^[^:]+:[0-9]+-[0-9]+$"}}
            ],
            "responses": {
              "200": {"description":"transcript page"},
              "400": {"description":"invalid query", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "429": {"description":"bulkhead saturated", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "503": {"description":"dataset unavailable", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/v1/transcripts/{tx_id}": {
          "get": {
            "parameters": [
              {"name":"tx_id","in":"path","required":true,"schema":{"type":"string"}},
              {"name":"release","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"species","in":"query","required":true,"schema":{"type":"string"}},
              {"name":"assembly","in":"query","required":true,"schema":{"type":"string"}}
            ],
            "responses": {
              "200": {"description":"transcript summary"},
              "400": {"description":"invalid query", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "404": {"description":"transcript not found", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}},
              "503": {"description":"dataset unavailable", "content":{"application/json":{"schema":{"$ref":"#/components/schemas/ApiError"}}}}
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
        },
        "/debug/dataset-health": {
          "get": {
            "parameters": [
              {"name": "release", "in": "query", "required": true, "schema": {"type": "string"}},
              {"name": "species", "in": "query", "required": true, "schema": {"type": "string"}},
              {"name": "assembly", "in": "query", "required": true, "schema": {"type": "string"}}
            ],
            "responses": {
              "200": {"description": "dataset cache/verification health"},
              "400": {"description": "invalid dataset dimensions", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}},
              "404": {"description": "disabled"},
              "503": {"description": "health evaluation failed", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ApiError"}}}}
            }
          }
        },
        "/debug/registry-health": {
          "get": {
            "responses": {
              "200": {"description": "registry health and merge status"},
              "404": {"description": "disabled"}
            }
          }
        },
        "/v1/_debug/echo": {
          "get": {
            "responses": {
              "200": {"description": "echo query params when debug is enabled"},
              "404": {"description": "disabled"}
            }
          }
        }
      },
      "components": {
        "schemas": {
          "ApiErrorCode": {
            "type": "string",
            "enum": error_codes
          },
          "ApiError": {
            "type": "object",
            "required": ["code", "message", "details", "request_id"],
            "additionalProperties": false,
            "properties": {
              "code": {"$ref": "#/components/schemas/ApiErrorCode"},
              "message": {"type": "string"},
              "details": {"type": "object", "additionalProperties": true},
              "request_id": {"type": "string"}
            },
            "examples": {
              "missingDataset": {
                "value": {
                  "code": "MissingDatasetDimension",
                  "message": "missing dataset dimension: release",
                  "details": {"dimension": "release"},
                  "request_id": "req-0000000000000001"
                }
              },
              "invalidCursor": {
                "value": {
                  "code": "InvalidCursor",
                  "message": "invalid cursor",
                  "details": {"cursor": "bad.cursor"},
                  "request_id": "req-0000000000000002"
                }
              }
            }
          }
        }
      }
    })
}
