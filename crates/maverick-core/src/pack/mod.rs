//! Language-independent pack contracts. Commands are data, never executable here.
mod adapter;
mod manifest;

pub use adapter::*;
pub use manifest::*;

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackError {
    #[error("invalid pack JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported schema_version {0}; expected 1")]
    SchemaVersion(u32),
    #[error("invalid {field}: {value:?}")]
    Invalid { field: String, value: String },
    #[error("duplicate {field}: {id}")]
    Duplicate { field: String, id: String },
    #[error("unknown selected pack: {0}")]
    UnknownPack(String),
    #[error("unexpected review result: {0}")]
    UnexpectedReview(String),
}

pub(crate) fn invalid(field: &str, value: &str) -> PackError {
    PackError::Invalid {
        field: field.into(),
        value: value.into(),
    }
}

pub(crate) fn identifier(value: &str, separators: &[char]) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.split(separators).all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

pub(crate) fn validate_id(field: &str, value: &str) -> Result<(), PackError> {
    if !identifier(value, &['-', '_']) {
        return Err(invalid(field, value));
    }
    Ok(())
}

/// Extensible snake_case identifier, validated on construction and deserialization.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Capability(String);

impl TryFrom<String> for Capability {
    type Error = PackError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !identifier(&value, &['_']) {
            return Err(invalid("capability (snake_case)", &value));
        }
        Ok(Self(value))
    }
}

impl From<Capability> for String {
    fn from(value: Capability) -> Self {
        value.0
    }
}

impl FromStr for Capability {
    type Err = PackError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.to_owned().try_into()
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeClassification {
    pub capabilities: std::collections::HashSet<Capability>,
}
