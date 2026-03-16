// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::domain::canonical::sha256_hex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestIdentity {
    pub request_id: String,
    pub client_ip: String,
    pub user_agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticationContext {
    pub principal: String,
    pub subject: String,
    pub issuer: Option<String>,
    pub scopes: Vec<String>,
    pub method: String,
    pub request: RequestIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub key_id: String,
    pub owner: String,
    pub key_hash: String,
    pub expires_at_unix_s: Option<u64>,
    pub revoked_at_unix_s: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyStore {
    records: BTreeMap<String, ApiKeyRecord>,
}

impl ApiKeyStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, record: ApiKeyRecord) {
        self.records.insert(record.key_id.clone(), record);
    }

    pub fn revoke(&mut self, key_id: &str, revoked_at_unix_s: u64) -> bool {
        if let Some(record) = self.records.get_mut(key_id) {
            record.revoked_at_unix_s = Some(revoked_at_unix_s);
            return true;
        }
        false
    }

    #[must_use]
    pub fn list_active_ids(&self, now_unix_s: u64) -> Vec<String> {
        self.records
            .values()
            .filter(|record| {
                record.revoked_at_unix_s.is_none()
                    && record
                        .expires_at_unix_s
                        .is_none_or(|expiry| now_unix_s <= expiry)
            })
            .map(|record| record.key_id.clone())
            .collect()
    }

    pub fn validate_raw_key(
        &self,
        raw_key: &str,
        now_unix_s: u64,
    ) -> Result<&ApiKeyRecord, AuthValidationError> {
        let hashed = hash_api_key(raw_key);
        let Some(record) = self
            .records
            .values()
            .find(|record| record.key_hash == hashed)
        else {
            return Err(AuthValidationError::ApiKeyInvalid);
        };
        if record.revoked_at_unix_s.is_some() {
            return Err(AuthValidationError::ApiKeyRevoked);
        }
        if record
            .expires_at_unix_s
            .is_some_and(|expiry| now_unix_s > expiry)
        {
            return Err(AuthValidationError::ApiKeyExpired);
        }
        Ok(record)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub nbf: Option<u64>,
    pub iat: Option<u64>,
    pub jti: Option<String>,
    pub scope: Option<String>,
}

impl TokenClaims {
    #[must_use]
    pub fn scopes(&self) -> Vec<String> {
        self.scope
            .as_deref()
            .unwrap_or_default()
            .split_whitespace()
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenValidationPolicy {
    pub required_issuer: String,
    pub required_audience: String,
    pub required_scopes: Vec<String>,
    pub revoked_token_ids: BTreeSet<String>,
}

impl TokenValidationPolicy {
    #[must_use]
    pub fn new(required_issuer: &str, required_audience: &str) -> Self {
        Self {
            required_issuer: required_issuer.to_string(),
            required_audience: required_audience.to_string(),
            required_scopes: Vec::new(),
            revoked_token_ids: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthValidationError {
    ApiKeyInvalid,
    ApiKeyExpired,
    ApiKeyRevoked,
    TokenMalformed,
    TokenSignatureInvalid,
    TokenExpired,
    TokenNotYetValid,
    TokenIssuerInvalid,
    TokenAudienceInvalid,
    TokenScopeDenied,
    TokenRevoked,
}

#[must_use]
pub fn hash_api_key(raw_key: &str) -> String {
    sha256_hex(raw_key.as_bytes())
}

#[must_use]
pub fn generate_api_key(key_id: &str, owner: &str) -> String {
    let now = unix_now_s();
    let material = format!("atlas-key:{key_id}:{owner}:{now}");
    let digest = sha256_hex(material.as_bytes());
    format!("atlas_{digest}")
}

pub fn rotate_api_key(
    store: &mut ApiKeyStore,
    current_key_id: &str,
    next_key_id: &str,
    owner: &str,
    overlap_ttl_secs: u64,
    now_unix_s: u64,
) -> String {
    let next_raw = generate_api_key(next_key_id, owner);
    store.upsert(ApiKeyRecord {
        key_id: next_key_id.to_string(),
        owner: owner.to_string(),
        key_hash: hash_api_key(&next_raw),
        expires_at_unix_s: None,
        revoked_at_unix_s: None,
    });
    if let Some(current) = store.records.get_mut(current_key_id) {
        current.expires_at_unix_s = Some(now_unix_s.saturating_add(overlap_ttl_secs));
    }
    next_raw
}

pub fn mint_signed_token(claims: &TokenClaims, signing_secret: &str) -> Result<String, String> {
    let payload =
        serde_json::to_vec(claims).map_err(|err| format!("token encode failed: {err}"))?;
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let signature = sign_token_payload(&payload_b64, signing_secret);
    Ok(format!("atlas.v1.{payload_b64}.{signature}"))
}

pub fn validate_signed_token(
    token: &str,
    signing_secret: &str,
    policy: &TokenValidationPolicy,
    now_unix_s: u64,
) -> Result<TokenClaims, AuthValidationError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 4 || parts[0] != "atlas" || parts[1] != "v1" {
        return Err(AuthValidationError::TokenMalformed);
    }
    let payload_b64 = parts[2];
    let signature = parts[3];
    let expected = sign_token_payload(payload_b64, signing_secret);
    if signature != expected {
        return Err(AuthValidationError::TokenSignatureInvalid);
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| AuthValidationError::TokenMalformed)?;
    let claims: TokenClaims =
        serde_json::from_slice(&payload).map_err(|_| AuthValidationError::TokenMalformed)?;
    if now_unix_s > claims.exp {
        return Err(AuthValidationError::TokenExpired);
    }
    if claims.nbf.is_some_and(|nbf| now_unix_s < nbf) {
        return Err(AuthValidationError::TokenNotYetValid);
    }
    if claims.iss != policy.required_issuer {
        return Err(AuthValidationError::TokenIssuerInvalid);
    }
    if claims.aud != policy.required_audience {
        return Err(AuthValidationError::TokenAudienceInvalid);
    }
    let scopes = claims.scopes();
    if !policy.required_scopes.is_empty()
        && policy
            .required_scopes
            .iter()
            .any(|scope| !scopes.iter().any(|actual| actual == scope))
    {
        return Err(AuthValidationError::TokenScopeDenied);
    }
    if claims
        .jti
        .as_ref()
        .is_some_and(|jti| policy.revoked_token_ids.contains(jti))
    {
        return Err(AuthValidationError::TokenRevoked);
    }
    Ok(claims)
}

#[must_use]
pub fn authentication_context_from_api_key(
    record: &ApiKeyRecord,
    request: RequestIdentity,
) -> AuthenticationContext {
    AuthenticationContext {
        principal: "service-account".to_string(),
        subject: record.owner.clone(),
        issuer: Some("atlas-api-key".to_string()),
        scopes: vec!["dataset.read".to_string()],
        method: "api-key".to_string(),
        request,
    }
}

#[must_use]
pub fn authentication_context_from_token(
    claims: &TokenClaims,
    request: RequestIdentity,
) -> AuthenticationContext {
    AuthenticationContext {
        principal: "user".to_string(),
        subject: claims.sub.clone(),
        issuer: Some(claims.iss.clone()),
        scopes: claims.scopes(),
        method: "token".to_string(),
        request,
    }
}

#[must_use]
pub fn extract_request_identity(headers: &BTreeMap<String, String>) -> RequestIdentity {
    let request_id = headers
        .get("x-request-id")
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            let client_ip = headers
                .get("x-forwarded-for")
                .and_then(|value| value.split(',').next())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or_default();
            let user_agent = headers
                .get("user-agent")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_default();
            let fingerprint = sha256_hex(format!("{client_ip}\n{user_agent}").as_bytes());
            format!("req-{}", &fingerprint[..16])
        });
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_default();
    let user_agent = headers.get("user-agent").cloned().unwrap_or_default();
    RequestIdentity {
        request_id,
        client_ip,
        user_agent,
    }
}

fn sign_token_payload(payload_b64: &str, signing_secret: &str) -> String {
    let material = format!("atlas.v1.{payload_b64}.{signing_secret}");
    sha256_hex(material.as_bytes())
}

fn unix_now_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{
        authentication_context_from_token, extract_request_identity, hash_api_key,
        mint_signed_token, rotate_api_key, validate_signed_token, ApiKeyRecord, ApiKeyStore,
        AuthValidationError, TokenClaims, TokenValidationPolicy,
    };
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn api_key_store_supports_validation_expiration_and_revocation() {
        let mut store = ApiKeyStore::new();
        store.upsert(ApiKeyRecord {
            key_id: "atlas-key-a".to_string(),
            owner: "svc:ingest".to_string(),
            key_hash: hash_api_key("atlas_secret"),
            expires_at_unix_s: Some(200),
            revoked_at_unix_s: None,
        });
        assert!(store.validate_raw_key("atlas_secret", 150).is_ok());
        assert_eq!(
            store.validate_raw_key("atlas_secret", 250),
            Err(AuthValidationError::ApiKeyExpired)
        );
        assert!(store.revoke("atlas-key-a", 260));
        assert_eq!(
            store.validate_raw_key("atlas_secret", 260),
            Err(AuthValidationError::ApiKeyRevoked)
        );
    }

    #[test]
    fn api_key_rotation_keeps_overlap_and_registers_new_key() {
        let mut store = ApiKeyStore::new();
        store.upsert(ApiKeyRecord {
            key_id: "atlas-key-a".to_string(),
            owner: "svc:query".to_string(),
            key_hash: hash_api_key("atlas_old"),
            expires_at_unix_s: None,
            revoked_at_unix_s: None,
        });
        let new_key = rotate_api_key(
            &mut store,
            "atlas-key-a",
            "atlas-key-b",
            "svc:query",
            300,
            1_000,
        );
        assert!(store.validate_raw_key(&new_key, 1_001).is_ok());
        let active = store.list_active_ids(1_100);
        assert!(active.iter().any(|id| id == "atlas-key-a"));
        assert!(active.iter().any(|id| id == "atlas-key-b"));
    }

    #[test]
    fn token_pipeline_enforces_signature_issuer_audience_scope_and_revocation() {
        let claims = TokenClaims {
            sub: "user:alice".to_string(),
            iss: "atlas-auth".to_string(),
            aud: "atlas-api".to_string(),
            exp: 2_000,
            nbf: Some(1_000),
            iat: Some(1_000),
            jti: Some("jti-1".to_string()),
            scope: Some("dataset.read profile".to_string()),
        };
        let token = mint_signed_token(&claims, "secret").expect("token");
        let mut policy = TokenValidationPolicy::new("atlas-auth", "atlas-api");
        policy.required_scopes = vec!["dataset.read".to_string()];
        assert!(validate_signed_token(&token, "secret", &policy, 1_500).is_ok());
        let mut revoked = BTreeSet::new();
        revoked.insert("jti-1".to_string());
        policy.revoked_token_ids = revoked;
        assert_eq!(
            validate_signed_token(&token, "secret", &policy, 1_500),
            Err(AuthValidationError::TokenRevoked)
        );
    }

    #[test]
    fn request_identity_extraction_and_context_binding_are_deterministic() {
        let mut headers = BTreeMap::new();
        headers.insert("x-request-id".to_string(), "req-1".to_string());
        headers.insert(
            "x-forwarded-for".to_string(),
            "203.0.113.4, 198.51.100.10".to_string(),
        );
        headers.insert("user-agent".to_string(), "atlas-client/1.0".to_string());
        let identity = extract_request_identity(&headers);
        assert_eq!(identity.request_id, "req-1");
        assert_eq!(identity.client_ip, "203.0.113.4");
        assert_eq!(identity.user_agent, "atlas-client/1.0");

        let claims = TokenClaims {
            sub: "user:alice".to_string(),
            iss: "atlas-auth".to_string(),
            aud: "atlas-api".to_string(),
            exp: 2_000,
            nbf: None,
            iat: None,
            jti: None,
            scope: Some("dataset.read".to_string()),
        };
        let context = authentication_context_from_token(&claims, identity);
        assert_eq!(context.principal, "user");
        assert_eq!(context.subject, "user:alice");
        assert_eq!(context.method, "token");
        assert_eq!(context.scopes, vec!["dataset.read".to_string()]);
    }

    #[test]
    fn request_identity_derives_a_concrete_id_when_header_is_missing() {
        let mut headers = BTreeMap::new();
        headers.insert(
            "x-forwarded-for".to_string(),
            "203.0.113.9, 198.51.100.10".to_string(),
        );
        headers.insert("user-agent".to_string(), "atlas-client/2.0".to_string());

        let first = extract_request_identity(&headers);
        let second = extract_request_identity(&headers);

        assert!(first.request_id.starts_with("req-"));
        assert_eq!(first.request_id, second.request_id);
        assert_eq!(first.client_ip, "203.0.113.9");
        assert_eq!(first.user_agent, "atlas-client/2.0");
    }

    #[test]
    fn request_identity_keeps_missing_client_fields_empty() {
        let headers = BTreeMap::new();

        let identity = extract_request_identity(&headers);

        assert!(identity.request_id.starts_with("req-"));
        assert!(identity.client_ip.is_empty());
        assert!(identity.user_agent.is_empty());
    }
}
