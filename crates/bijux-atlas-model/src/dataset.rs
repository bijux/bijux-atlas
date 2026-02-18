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

pub const RELEASE_MAX_LEN: usize = 16;
pub const SPECIES_MAX_LEN: usize = 64;
pub const ASSEMBLY_MAX_LEN: usize = 64;

pub fn parse_release(input: &str) -> Result<Release, ValidationError> {
    Release::parse(input)
}

pub fn parse_species(input: &str) -> Result<Species, ValidationError> {
    Species::parse(input)
}

pub fn parse_assembly(input: &str) -> Result<Assembly, ValidationError> {
    Assembly::parse(input)
}

pub fn parse_dataset_key(input: &str) -> Result<DatasetId, ValidationError> {
    DatasetId::parse_key(input)
}

pub fn parse_species_normalized(input: &str) -> Result<Species, ValidationError> {
    let normalized = input.trim().to_ascii_lowercase().replace('-', "_");
    Species::parse(&normalized)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
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
        if s.len() > RELEASE_MAX_LEN {
            return Err(ValidationError(format!(
                "release exceeds max length {RELEASE_MAX_LEN}"
            )));
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

impl Display for Release {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
#[non_exhaustive]
pub struct Species(String);

impl Species {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(ValidationError("species must not be empty".to_string()));
        }
        if s.len() > SPECIES_MAX_LEN {
            return Err(ValidationError(format!(
                "species exceeds max length {SPECIES_MAX_LEN}"
            )));
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

impl Display for Species {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
#[non_exhaustive]
pub struct Assembly(String);

impl Assembly {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ValidationError("assembly must not be empty".to_string()));
        }
        if trimmed.len() > ASSEMBLY_MAX_LEN {
            return Err(ValidationError(format!(
                "assembly exceeds max length {ASSEMBLY_MAX_LEN}"
            )));
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

impl Display for Assembly {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct DatasetId {
    pub release: Release,
    pub species: Species,
    pub assembly: Assembly,
}

impl DatasetId {
    pub fn new(release: &str, species: &str, assembly: &str) -> Result<Self, ValidationError> {
        Ok(Self {
            release: parse_release(release)?,
            species: parse_species(species)?,
            assembly: parse_assembly(assembly)?,
        })
    }

    pub fn from_normalized(
        release: &str,
        species: &str,
        assembly: &str,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            release: parse_release(release)?,
            species: parse_species_normalized(species)?,
            assembly: parse_assembly(assembly)?,
        })
    }

    #[must_use]
    pub fn canonical_string(&self) -> String {
        format!(
            "{}/{}/{}",
            self.release.as_str(),
            self.species.as_str(),
            self.assembly.as_str()
        )
    }

    pub fn from_canonical_string(input: &str) -> Result<Self, ValidationError> {
        let trimmed = input.trim();
        let mut parts = trimmed.split('/');
        let release = parts
            .next()
            .ok_or_else(|| ValidationError("dataset canonical form missing release".to_string()))?;
        let species = parts
            .next()
            .ok_or_else(|| ValidationError("dataset canonical form missing species".to_string()))?;
        let assembly = parts
            .next()
            .ok_or_else(|| ValidationError("dataset canonical form missing assembly".to_string()))?;
        if parts.next().is_some() {
            return Err(ValidationError(
                "dataset canonical form must be release/species/assembly".to_string(),
            ));
        }
        Self::new(release, species, assembly)
    }

    #[must_use]
    pub fn key_string(&self) -> String {
        format!(
            "release={}&species={}&assembly={}",
            self.release.as_str(),
            self.species.as_str(),
            self.assembly.as_str()
        )
    }

    pub fn parse_key(input: &str) -> Result<Self, ValidationError> {
        let trimmed = input.trim();
        let mut release: Option<&str> = None;
        let mut species: Option<&str> = None;
        let mut assembly: Option<&str> = None;
        for part in trimmed.split('&') {
            let (k, v) = part.split_once('=').ok_or_else(|| {
                ValidationError(
                    "dataset key must use release=<r>&species=<s>&assembly=<a>".to_string(),
                )
            })?;
            match k {
                "release" => release = Some(v),
                "species" => species = Some(v),
                "assembly" => assembly = Some(v),
                other => {
                    return Err(ValidationError(format!(
                        "unknown dataset key segment: {other}"
                    )))
                }
            }
        }
        let release =
            release.ok_or_else(|| ValidationError("dataset key missing release".to_string()))?;
        let species =
            species.ok_or_else(|| ValidationError("dataset key missing species".to_string()))?;
        let assembly =
            assembly.ok_or_else(|| ValidationError("dataset key missing assembly".to_string()))?;
        Self::new(release, species, assembly)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum DatasetSelector {
    Explicit(DatasetId),
}

pub fn normalize_species(input: &str) -> Result<String, ValidationError> {
    parse_species_normalized(input).map(Species::into_inner)
}

pub fn normalize_assembly(input: &str) -> Result<String, ValidationError> {
    parse_assembly(input).map(Assembly::into_inner)
}

pub fn normalize_release(input: &str) -> Result<String, ValidationError> {
    parse_release(input).map(Release::into_inner)
}
