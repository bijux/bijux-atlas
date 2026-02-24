use bijux_atlas_model::DatasetId;

#[must_use]
pub(crate) fn dataset_route_path(dataset: &DatasetId) -> String {
    format!(
        "/v1/datasets/{}/{}/{}",
        dataset.release, dataset.species, dataset.assembly
    )
}
