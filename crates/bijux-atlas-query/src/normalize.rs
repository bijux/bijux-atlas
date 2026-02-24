use crate::filters::GeneQueryRequest;
use crate::parser::{GeneQueryAst, Predicate};
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

#[must_use]
pub fn normalized_ast_format(ast: &GeneQueryAst) -> String {
    let mut predicates = ast.predicates.clone();
    predicates.sort_by_key(predicate_sort_key);
    let ordered = GeneQueryAst {
        predicates,
        limit: ast.limit,
        dataset_key: ast.dataset_key.clone(),
        allow_full_scan: ast.allow_full_scan,
        has_cursor: ast.has_cursor,
        sort_key: ast.sort_key,
    };
    serde_json::to_string(&ordered).expect("ast serialization must succeed")
}

fn predicate_sort_key(predicate: &Predicate) -> String {
    match predicate {
        Predicate::GeneId(v) => format!("0:{v}"),
        Predicate::NameEquals(v) => format!("1:{v}"),
        Predicate::NamePrefix(v) => format!("2:{v}"),
        Predicate::Biotype(v) => format!("3:{v}"),
        Predicate::Region { seqid, start, end } => format!("4:{seqid}:{start}:{end}"),
    }
}
