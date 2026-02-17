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
              {"name": "name_prefix", "in": "query", "schema": {"type": "string"}},
              {"name": "biotype", "in": "query", "schema": {"type": "string"}},
              {"name": "region", "in": "query", "schema": {"type": "string", "pattern": "^[^:]+:[0-9]+-[0-9]+$"}},
              {"name": "limit", "in": "query", "schema": {"type": "integer", "minimum": 1, "maximum": 500}},
              {"name": "cursor", "in": "query", "schema": {"type": "string", "maxLength": 4096}},
              {"name": "fields", "in": "query", "schema": {"type": "string", "description": "comma-separated projection fields"}},
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
