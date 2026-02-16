use crate::filters::GeneQueryRequest;

#[must_use]
pub fn estimate_prefix_match_cost(req: &GeneQueryRequest) -> u64 {
    let Some(prefix) = req.filter.name_prefix.as_ref() else {
        return 0;
    };
    let len = prefix.len() as u64;
    if len == 0 {
        return u64::MAX;
    }
    // Short prefixes can match huge sets; model this superlinearly.
    let inverse_selectivity = 256_u64.saturating_sub((len * 16).min(240));
    inverse_selectivity.saturating_mul(req.limit as u64)
}
