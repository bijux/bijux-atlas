use bijux_atlas_api::{list_genes_v1, ApiError, IncludeField, ListGenesParams, QueryAdapter};
use bijux_atlas_core::ErrorCode;
use bijux_atlas_query::{GeneQueryResponse, GeneRow};

#[derive(Default)]
struct FakeAdapter {
    rows: Vec<GeneRow>,
    next_cursor: Option<String>,
}

impl QueryAdapter for FakeAdapter {
    fn list_genes(&self, _params: &ListGenesParams) -> Result<GeneQueryResponse, ApiError> {
        Ok(GeneQueryResponse {
            rows: self.rows.clone(),
            next_cursor: self.next_cursor.clone(),
        })
    }
}

fn base_params() -> ListGenesParams {
    ListGenesParams {
        release: "110".to_string(),
        species: "homo_sapiens".to_string(),
        assembly: "GRCh38".to_string(),
        limit: 10,
        cursor: None,
        gene_id: None,
        name: None,
        name_prefix: None,
        biotype: None,
        region: None,
        include: None,
        pretty: false,
    }
}

#[test]
fn wire_response_uses_omit_vs_null_policy() {
    let adapter = FakeAdapter {
        rows: vec![GeneRow {
            gene_id: "gene1".to_string(),
            name: Some("BRCA1".to_string()),
            seqid: Some("chr1".to_string()),
            start: Some(10),
            end: Some(20),
            biotype: None,
            transcript_count: Some(2),
            sequence_length: Some(11),
        }],
        next_cursor: Some("v1.cursor".to_string()),
    };

    let mut params = base_params();
    params.include = Some(vec![IncludeField::Biotype]);

    let payload = list_genes_v1(&adapter, &params).expect("wire response");
    let encoded = serde_json::to_string(&payload).expect("json");
    assert_eq!(
        encoded,
        "{\"next_cursor\":\"v1.cursor\",\"rows\":[{\"biotype\":null,\"gene_id\":\"gene1\",\"name\":\"BRCA1\"}]}"
    );
}

#[test]
fn wire_response_golden_for_default_projection() {
    let adapter = FakeAdapter {
        rows: vec![GeneRow {
            gene_id: "gene1".to_string(),
            name: Some("BRCA1".to_string()),
            seqid: Some("chr1".to_string()),
            start: Some(10),
            end: Some(20),
            biotype: Some("protein_coding".to_string()),
            transcript_count: Some(2),
            sequence_length: Some(11),
        }],
        next_cursor: None,
    };

    let payload = list_genes_v1(&adapter, &base_params()).expect("wire response");
    let encoded = serde_json::to_string(&payload).expect("json");
    assert_eq!(
        encoded,
        "{\"next_cursor\":null,\"rows\":[{\"gene_id\":\"gene1\",\"name\":\"BRCA1\"}]}"
    );
}

#[test]
fn fake_adapter_error_contract_is_stable() {
    let err = ApiError::new(
        ErrorCode::InvalidQueryParameter,
        "bad",
        serde_json::json!({"parameter":"limit"}),
    );
    let encoded = serde_json::to_string(&err).expect("json");
    assert_eq!(
        encoded,
        "{\"code\":\"InvalidQueryParameter\",\"message\":\"bad\",\"details\":{\"parameter\":\"limit\"}}"
    );
}
