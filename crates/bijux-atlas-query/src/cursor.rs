use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;
const CURSOR_VERSION_V1: &str = "v1";
const MAX_CURSOR_DEPTH: u32 = 10_000;
const MAX_CURSOR_TOKEN_LEN: usize = 1024;
const MAX_CURSOR_PAYLOAD_PART_LEN: usize = 768;
const MAX_CURSOR_SIG_PART_LEN: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CursorErrorCode {
    InvalidFormat,
    UnsupportedVersion,
    InvalidSignature,
    InvalidPayload,
    DatasetMismatch,
    QueryHashMismatch,
    OrderMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorError {
    pub code: CursorErrorCode,
    pub message: String,
}

impl CursorError {
    #[must_use]
    pub fn new(code: CursorErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for CursorError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorLastSeen {
    pub gene_id: String,
    pub seqid: Option<String>,
    pub start: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorPayload {
    #[serde(default = "cursor_version_v1")]
    pub cursor_version: String,
    #[serde(default)]
    pub dataset_id: Option<String>,
    #[serde(default)]
    pub sort_key: Option<String>,
    #[serde(default)]
    pub last_seen: Option<CursorLastSeen>,
    pub order: String,
    pub last_seqid: Option<String>,
    pub last_start: Option<u64>,
    pub last_gene_id: String,
    pub query_hash: String,
    #[serde(default)]
    pub depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OrderMode {
    Region,
    GeneId,
}

pub fn encode_cursor(payload: &CursorPayload, secret: &[u8]) -> Result<String, CursorError> {
    let payload_bytes = serde_json::to_vec(payload)
        .map_err(|e| CursorError::new(CursorErrorCode::InvalidPayload, e.to_string()))?;
    let payload_part = URL_SAFE_NO_PAD.encode(payload_bytes);
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|e| CursorError::new(CursorErrorCode::InvalidPayload, e.to_string()))?;
    mac.update(payload_part.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_part = URL_SAFE_NO_PAD.encode(sig);
    Ok(format!(
        "{}.{}.{}",
        CURSOR_VERSION_V1, payload_part, sig_part
    ))
}

pub fn decode_cursor(
    token: &str,
    secret: &[u8],
    expected_hash: &str,
    mode: OrderMode,
    expected_dataset: Option<&str>,
) -> Result<CursorPayload, CursorError> {
    if token.len() > MAX_CURSOR_TOKEN_LEN {
        return Err(CursorError::new(
            CursorErrorCode::InvalidFormat,
            "cursor exceeds max length",
        ));
    }
    let (payload_part, sig_part) = parse_cursor_parts(token)?;
    if payload_part.len() > MAX_CURSOR_PAYLOAD_PART_LEN || sig_part.len() > MAX_CURSOR_SIG_PART_LEN
    {
        return Err(CursorError::new(
            CursorErrorCode::InvalidFormat,
            "cursor part exceeds max length",
        ));
    }

    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|e| CursorError::new(CursorErrorCode::InvalidPayload, e.to_string()))?;
    mac.update(payload_part.as_bytes());
    let expected = URL_SAFE_NO_PAD
        .decode(sig_part)
        .map_err(|e| CursorError::new(CursorErrorCode::InvalidFormat, e.to_string()))?;
    mac.verify_slice(&expected).map_err(|_| {
        CursorError::new(
            CursorErrorCode::InvalidSignature,
            "cursor signature mismatch",
        )
    })?;

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_part)
        .map_err(|e| CursorError::new(CursorErrorCode::InvalidFormat, e.to_string()))?;
    let payload: CursorPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| CursorError::new(CursorErrorCode::InvalidPayload, e.to_string()))?;

    if payload.cursor_version != CURSOR_VERSION_V1 {
        return Err(CursorError::new(
            CursorErrorCode::UnsupportedVersion,
            "cursor version unsupported",
        ));
    }
    if let Some(dataset) = expected_dataset {
        if payload.dataset_id.as_deref() != Some(dataset) {
            return Err(CursorError::new(
                CursorErrorCode::DatasetMismatch,
                "cursor dataset mismatch",
            ));
        }
    }
    if payload.query_hash != expected_hash {
        return Err(CursorError::new(
            CursorErrorCode::QueryHashMismatch,
            "cursor query hash mismatch",
        ));
    }

    match mode {
        OrderMode::Region if payload.order != "region" => Err(CursorError::new(
            CursorErrorCode::OrderMismatch,
            "cursor order mismatch for region query",
        )),
        OrderMode::GeneId if payload.order != "gene_id" => Err(CursorError::new(
            CursorErrorCode::OrderMismatch,
            "cursor order mismatch for gene_id query",
        )),
        _ if payload.depth > MAX_CURSOR_DEPTH => Err(CursorError::new(
            CursorErrorCode::InvalidPayload,
            "cursor depth exceeds max",
        )),
        _ => Ok(payload),
    }
}

fn parse_cursor_parts(token: &str) -> Result<(&str, &str), CursorError> {
    let parts: Vec<&str> = token.split('.').collect();
    match parts.as_slice() {
        // Current versioned format.
        [version, payload, sig] if *version == CURSOR_VERSION_V1 => Ok((payload, sig)),
        [version, _, _] => Err(CursorError::new(
            CursorErrorCode::UnsupportedVersion,
            format!("unsupported cursor version: {version}"),
        )),
        // Legacy unversioned format kept for backward-compatible decoding.
        [payload, sig] => Ok((payload, sig)),
        _ => Err(CursorError::new(
            CursorErrorCode::InvalidFormat,
            "invalid cursor format",
        )),
    }
}

fn cursor_version_v1() -> String {
    CURSOR_VERSION_V1.to_string()
}
