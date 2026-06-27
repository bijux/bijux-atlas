// SPDX-License-Identifier: Apache-2.0

use super::{chrono_like_unix_secs, normalized_header_value};
use crate::app::server::observability::unix_time_millis;
use crate::sha256_hex;
use axum::http::HeaderMap;
use base64::Engine as _;
use hmac::{digest::KeyInit, Hmac, Mac};
use sha2::Sha256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthenticationContext {
    pub(super) principal: &'static str,
    pub(super) mechanism: &'static str,
    pub(super) subject: String,
    pub(super) issuer: Option<String>,
    pub(super) scopes: Vec<String>,
}

#[derive(Debug, Clone)]
struct ApiKeyRecord {
    key_hash: String,
    not_before_unix_s: Option<u64>,
    expires_unix_s: Option<u64>,
    revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ApiKeyValidationError {
    Unknown,
    NotYetValid,
    Expired,
    Revoked,
}

#[derive(Debug, Clone)]
pub(super) struct ApiKeyStore {
    records: Vec<ApiKeyRecord>,
}

impl ApiKeyStore {
    pub(super) fn from_allowed_entries(entries: &[String], expiration_days: u64) -> Self {
        let now = chrono_like_unix_secs();
        let expires_unix_s = now.saturating_add(expiration_days.saturating_mul(86_400));
        let mut records = Vec::new();
        for entry in entries {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parsed = parse_api_key_record_line(trimmed).unwrap_or_else(|| ApiKeyRecord {
                key_hash: hash_api_key(trimmed),
                not_before_unix_s: None,
                expires_unix_s: Some(expires_unix_s),
                revoked: false,
            });
            records.push(parsed);
        }
        Self { records }
    }

    pub(super) fn validate(
        &self,
        raw_key: &str,
        now_unix_s: u64,
    ) -> Result<(), ApiKeyValidationError> {
        let candidate_hash = hash_api_key(raw_key);
        let Some(record) = self
            .records
            .iter()
            .find(|item| item.key_hash == candidate_hash)
        else {
            return Err(ApiKeyValidationError::Unknown);
        };
        if record.revoked {
            return Err(ApiKeyValidationError::Revoked);
        }
        if let Some(not_before) = record.not_before_unix_s {
            if now_unix_s < not_before {
                return Err(ApiKeyValidationError::NotYetValid);
            }
        }
        if let Some(expires) = record.expires_unix_s {
            if now_unix_s > expires {
                return Err(ApiKeyValidationError::Expired);
            }
        }
        Ok(())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

pub(super) fn hash_api_key(raw_key: &str) -> String {
    sha256_hex(raw_key.as_bytes())
}

fn parse_api_key_record_line(input: &str) -> Option<ApiKeyRecord> {
    if !input.starts_with("hash=") {
        return None;
    }
    let mut hash = None;
    let mut not_before_unix_s = None;
    let mut expires_unix_s = None;
    let mut revoked = false;
    for part in input.split('|') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next()?;
        let value = kv.next().unwrap_or_default();
        match key {
            "hash" => hash = Some(value.to_string()),
            "not_before" => not_before_unix_s = value.parse::<u64>().ok(),
            "expires" => expires_unix_s = value.parse::<u64>().ok(),
            "revoked" => revoked = value.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }
    let key_hash = hash?;
    if key_hash.len() != 64 || !key_hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    Some(ApiKeyRecord {
        key_hash,
        not_before_unix_s,
        expires_unix_s,
        revoked,
    })
}

#[allow(dead_code)]
pub(super) fn generate_api_key(subject: &str) -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let now = unix_time_millis();
    let sequence = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let seed = format!("{subject}:{now}:{sequence}:{}", std::process::id());
    let material = sha256_hex(seed.as_bytes());
    format!("atlas_{material}")
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TokenClaims {
    sub: Option<String>,
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<u64>,
    nbf: Option<u64>,
    jti: Option<String>,
    scope: Option<String>,
    scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenValidationError {
    Malformed,
    Signature,
    Expired,
    NotYetValid,
    Issuer,
    Audience,
    Scope,
    Revoked,
}

impl TokenValidationError {
    pub(super) const fn as_code(&self) -> &'static str {
        match self {
            Self::Malformed => "token_malformed",
            Self::Signature => "token_signature_invalid",
            Self::Expired => "token_expired",
            Self::NotYetValid => "token_not_yet_valid",
            Self::Issuer => "token_issuer_invalid",
            Self::Audience => "token_audience_invalid",
            Self::Scope => "token_scope_missing",
            Self::Revoked => "token_revoked",
        }
    }
}

pub(super) fn token_header_value(headers: &HeaderMap) -> Option<String> {
    let raw = normalized_header_value(headers, "authorization", 4096)?;
    let mut parts = raw.splitn(2, ' ');
    let scheme = parts.next().unwrap_or_default();
    let token = parts.next().unwrap_or_default();
    if !scheme.eq_ignore_ascii_case("bearer") || token.trim().is_empty() {
        return None;
    }
    Some(token.trim().to_string())
}

pub(super) fn validate_signed_token(
    token: &str,
    api: &bijux_atlas_runtime::runtime::config::ApiConfig,
) -> Result<AuthenticationContext, TokenValidationError> {
    let Some(secret) = api.token_signing_secret.as_deref() else {
        return Err(TokenValidationError::Malformed);
    };
    let mut parts = token.split('.');
    let (Some(header_b64), Some(payload_b64), Some(sig_b64), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(TokenValidationError::Malformed);
    };
    let signed_content = format!("{header_b64}.{payload_b64}");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| TokenValidationError::Malformed)?;
    mac.update(signed_content.as_bytes());
    let expected = mac.finalize().into_bytes();
    let parsed_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| TokenValidationError::Malformed)?;
    if parsed_sig != expected.as_slice() {
        return Err(TokenValidationError::Signature);
    }
    let claims_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| TokenValidationError::Malformed)?;
    let claims: TokenClaims =
        serde_json::from_slice(&claims_json).map_err(|_| TokenValidationError::Malformed)?;
    let now = chrono_like_unix_secs();
    if let Some(nbf) = claims.nbf {
        if now < nbf {
            return Err(TokenValidationError::NotYetValid);
        }
    }
    if let Some(exp) = claims.exp {
        if now >= exp {
            return Err(TokenValidationError::Expired);
        }
    }
    if let Some(required) = api.token_required_issuer.as_deref() {
        if claims.iss.as_deref() != Some(required) {
            return Err(TokenValidationError::Issuer);
        }
    }
    if let Some(required) = api.token_required_audience.as_deref() {
        if claims.aud.as_deref() != Some(required) {
            return Err(TokenValidationError::Audience);
        }
    }
    if let Some(jti) = claims.jti.as_deref() {
        if api.token_revoked_ids.iter().any(|value| value == jti) {
            return Err(TokenValidationError::Revoked);
        }
    }
    let mut scopes = claims.scopes.unwrap_or_default();
    if let Some(scope_text) = claims.scope {
        for scope in scope_text.split(' ') {
            let normalized = scope.trim();
            if !normalized.is_empty() && !scopes.iter().any(|value| value == normalized) {
                scopes.push(normalized.to_string());
            }
        }
    }
    for required in &api.token_required_scopes {
        if !scopes.iter().any(|scope| scope == required) {
            return Err(TokenValidationError::Scope);
        }
    }
    let Some(subject) = claims.sub.filter(|value| !value.trim().is_empty()) else {
        return Err(TokenValidationError::Malformed);
    };
    Ok(AuthenticationContext {
        principal: "user",
        mechanism: "token",
        subject,
        issuer: claims.iss,
        scopes,
    })
}

pub(super) fn proxy_authenticated_principal(
    headers: &HeaderMap,
    auth_mode: bijux_atlas_runtime::runtime::config::AuthMode,
) -> Option<&'static str> {
    match auth_mode {
        bijux_atlas_runtime::runtime::config::AuthMode::Oidc => {
            normalized_header_value(headers, "x-forwarded-user", 256)
                .or_else(|| normalized_header_value(headers, "x-atlas-oidc-subject", 256))
                .map(|_| "user")
        }
        bijux_atlas_runtime::runtime::config::AuthMode::Mtls => {
            normalized_header_value(headers, "x-forwarded-client-cert", 512)
                .or_else(|| normalized_header_value(headers, "x-atlas-mtls-subject", 256))
                .map(|_| "service-account")
        }
        _ => None,
    }
}

pub(super) fn build_hmac_signature(
    secret: &str,
    method: &str,
    uri: &str,
    ts: &str,
) -> Option<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
    let payload = format!("{method}\n{uri}\n{ts}\n");
    mac.update(payload.as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}
