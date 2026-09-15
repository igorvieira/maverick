use crate::{ReviewResult, RunId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use thiserror::Error;

pub const ARTIFACT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    Task,
    Context,
    Plan,
    Classification,
    Review,
    Qa,
    Delivery,
}

impl FromStr for ArtifactKind {
    type Err = ArtifactError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(value.into()))
            .map_err(|_| ArtifactError::UnknownKind(value.into()))
    }
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("unknown artifact kind: {0}")]
    UnknownKind(String),
    #[error(
        "invalid artifact name: {0}; use an ASCII filename ending in .json, without directories"
    )]
    InvalidName(String),
    #[error("artifact name {name} does not match kind {kind:?}")]
    KindNameMismatch { kind: ArtifactKind, name: String },
    #[error("invalid structured review: {0}")]
    InvalidReview(#[source] serde_json::Error),
}

impl ArtifactKind {
    /// Non-review artifacts have fixed names; reviews use a single safe filename.
    pub fn validate_name(self, name: &str) -> Result<(), ArtifactError> {
        let stem = name.strip_suffix(".json").unwrap_or("");
        if stem.is_empty()
            || name.len() > 128
            || !stem
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(ArtifactError::InvalidName(name.into()));
        }
        let expected = match self {
            Self::Task => "task.json",
            Self::Context => "context.json",
            Self::Plan => "plan.json",
            Self::Classification => "classification.json",
            Self::Qa => "qa.json",
            Self::Delivery => "delivery.json",
            Self::Review => return Ok(()),
        };
        if name != expected {
            return Err(ArtifactError::KindNameMismatch {
                kind: self,
                name: name.into(),
            });
        }
        Ok(())
    }

    pub fn validate_payload(self, payload: &Value) -> Result<(), ArtifactError> {
        if self == Self::Review {
            serde_json::from_value::<ReviewResult>(payload.clone())
                .map_err(ArtifactError::InvalidReview)?;
        }
        Ok(())
    }
}

/// Schema version describes the envelope; revision counts writes to this artifact.
/// Payloads remain project/language agnostic, except the typed review contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema_version: u32,
    pub run_id: RunId,
    pub kind: ArtifactKind,
    pub name: String,
    pub revision: u64,
    pub payload: Value,
}
