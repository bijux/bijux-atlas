use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Release(String);

impl Release {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(ValidationError("release must not be empty".to_string()));
        }
        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError(
                "release must be numeric string (e.g. 110)".to_string(),
            ));
        }
        Ok(Self(s.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Species(String);

impl Species {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let s = input.trim().to_ascii_lowercase().replace('-', "_");
        if s.is_empty() {
            return Err(ValidationError("species must not be empty".to_string()));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(ValidationError(
                "species must match [a-z0-9_]+ in snake_case".to_string(),
            ));
        }
        if s.starts_with('_') || s.ends_with('_') || s.contains("__") {
            return Err(ValidationError(
                "species must not start/end with '_' or contain '__'".to_string(),
            ));
        }
        Ok(Self(s))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Assembly(String);

impl Assembly {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ValidationError("assembly must not be empty".to_string()));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
        {
            return Err(ValidationError(
                "assembly must match [A-Za-z0-9._]+".to_string(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct DatasetId {
    pub release: String,
    pub species: String,
    pub assembly: String,
}

impl DatasetId {
    pub fn new(release: &str, species: &str, assembly: &str) -> Result<Self, ValidationError> {
        Ok(Self {
            release: Release::parse(release)?.into_inner(),
            species: Species::parse(species)?.into_inner(),
            assembly: Assembly::parse(assembly)?.into_inner(),
        })
    }

    #[must_use]
    pub fn canonical_string(&self) -> String {
        format!("{}/{}/{}", self.release, self.species, self.assembly)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum DatasetSelector {
    Explicit(DatasetId),
}

#[must_use]
pub fn normalize_species(input: &str) -> Result<String, ValidationError> {
    Species::parse(input).map(Species::into_inner)
}

#[must_use]
pub fn normalize_assembly(input: &str) -> Result<String, ValidationError> {
    Assembly::parse(input).map(Assembly::into_inner)
}

#[must_use]
pub fn normalize_release(input: &str) -> Result<String, ValidationError> {
    Release::parse(input).map(Release::into_inner)
}
