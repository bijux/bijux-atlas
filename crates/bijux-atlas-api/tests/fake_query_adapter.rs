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
        name_like: None,
        biotype: None,
        contig: None,
        range: None,
        min_transcripts: None,
        max_transcripts: None,
        sort: None,
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
        "{\"api_version\":\"v1\",\"contract_version\":\"v1\",\"data\":{\"rows\":[{\"biotype\":null,\"gene_id\":\"gene1\",\"name\":\"BRCA1\"}]},\"dataset\":{\"assembly\":\"GRCh38\",\"release\":\"110\",\"species\":\"homo_sapiens\"},\"links\":{\"next_cursor\":\"v1.cursor\"},\"page\":{\"next_cursor\":\"v1.cursor\"}}"
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
        "{\"api_version\":\"v1\",\"contract_version\":\"v1\",\"data\":{\"rows\":[{\"gene_id\":\"gene1\",\"name\":\"BRCA1\"}]},\"dataset\":{\"assembly\":\"GRCh38\",\"release\":\"110\",\"species\":\"homo_sapiens\"},\"links\":null,\"page\":{\"next_cursor\":null}}"
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

#[test]
fn include_flags_are_additive_and_keep_base_fields() {
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
    let mut params = base_params();
    params.include = Some(vec![IncludeField::Length]);
    let payload = list_genes_v1(&adapter, &params).expect("wire response");
    let row = &payload["data"]["rows"][0];
    assert_eq!(row["gene_id"], "gene1");
    assert_eq!(row["name"], "BRCA1");
    assert_eq!(row["sequence_length"], 11);
}

#[test]
fn include_does_not_change_row_order_or_cursor() {
    let adapter = FakeAdapter {
        rows: vec![
            GeneRow {
                gene_id: "g1".to_string(),
                name: Some("A".to_string()),
                seqid: Some("chr1".to_string()),
                start: Some(1),
                end: Some(2),
                biotype: Some("pc".to_string()),
                transcript_count: Some(1),
                sequence_length: Some(2),
            },
            GeneRow {
                gene_id: "g2".to_string(),
                name: Some("B".to_string()),
                seqid: Some("chr1".to_string()),
                start: Some(3),
                end: Some(4),
                biotype: Some("pc".to_string()),
                transcript_count: Some(1),
                sequence_length: Some(2),
            },
        ],
        next_cursor: Some("v1.cursor.stable".to_string()),
    };

    let base = list_genes_v1(&adapter, &base_params()).expect("base response");
    let mut included = base_params();
    included.include = Some(vec![IncludeField::Coords, IncludeField::Biotype]);
    let projected = list_genes_v1(&adapter, &included).expect("include response");

    assert_eq!(
        base["page"]["next_cursor"],
        projected["page"]["next_cursor"]
    );
    assert_eq!(
        base["data"]["rows"][0]["gene_id"],
        projected["data"]["rows"][0]["gene_id"]
    );
    assert_eq!(
        base["data"]["rows"][1]["gene_id"],
        projected["data"]["rows"][1]["gene_id"]
    );
}
