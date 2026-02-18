use crate::filters::GeneQueryRequest;
use bijux_atlas_core::canonical;

pub fn normalized_query_hash(req: &GeneQueryRequest) -> Result<String, String> {
    let normalized = normalize_request(req);
    let bytes = canonical::stable_json_bytes(&normalized).map_err(|e| e.to_string())?;
    Ok(canonical::stable_hash_hex(&bytes))
}

#[must_use]
pub fn normalize_request(req: &GeneQueryRequest) -> GeneQueryRequest {
    let mut normalized = req.clone();
    normalized.cursor = None;
    normalized.fields = crate::filters::GeneFields::default();
    normalized
}
