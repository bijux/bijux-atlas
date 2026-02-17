use crate::dataset::ValidationError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum StrictnessMode {
    Strict,
    Lenient,
    ReportOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum GeneIdentifierPolicy {
    Gff3Id,
    PreferEnsemblStableId {
        attribute_keys: Vec<String>,
        fallback_to_gff3_id: bool,
    },
}

impl Default for GeneIdentifierPolicy {
    fn default() -> Self {
        Self::Gff3Id
    }
}

impl GeneIdentifierPolicy {
    pub fn resolve(
        &self,
        attrs: &BTreeMap<String, String>,
        gff3_id: &str,
        strict: bool,
    ) -> Result<String, ValidationError> {
        match self {
            Self::Gff3Id => Ok(gff3_id.to_string()),
            Self::PreferEnsemblStableId {
                attribute_keys,
                fallback_to_gff3_id,
            } => {
                for key in attribute_keys {
                    if let Some(value) = attrs.get(key) {
                        let v = value.trim();
                        if !v.is_empty() {
                            if strict && !v.starts_with("ENS") {
                                return Err(ValidationError(
                                    "strict mode requires ENS* stable ID when using Ensembl policy"
                                        .to_string(),
                                ));
                            }
                            return Ok(v.to_string());
                        }
                    }
                }
                if *fallback_to_gff3_id {
                    Ok(gff3_id.to_string())
                } else {
                    Err(ValidationError(
                        "no Ensembl stable ID found and fallback disabled".to_string(),
                    ))
                }
            }
        }
    }
}
