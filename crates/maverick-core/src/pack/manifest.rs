use super::{invalid, validate_id, Capability, PackError};
use crate::{ReviewResult, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const PACK_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    pub detection: DetectionRules,
    pub commands: CommandSet,
    pub agents: AgentSlots,
    pub reviewers: Vec<ReviewerDefinition>,
    #[serde(default)]
    pub red_team: Option<RedTeamGate>,
    pub review_policy: ReviewPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectionRules {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub any_files: Vec<String>,
}

impl DetectionRules {
    /// All `files` must exist; when nonempty, `any_files` requires at least one.
    /// Empty rules never auto-select a pack. Paths are exact, relative file names.
    pub fn matches(&self, files: &HashSet<String>) -> bool {
        (!self.files.is_empty() || !self.any_files.is_empty())
            && self.files.iter().all(|f| files.contains(f))
            && (self.any_files.is_empty() || self.any_files.iter().any(|f| files.contains(f)))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandSet {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub build: Option<String>,
    #[serde(default)]
    pub test: Option<String>,
    #[serde(default)]
    pub lint: Option<String>,
    #[serde(default)]
    pub typecheck: Option<String>,
}

impl CommandSet {
    pub fn validate(&self) -> Result<(), PackError> {
        for (name, value) in [
            ("format", &self.format),
            ("build", &self.build),
            ("test", &self.test),
            ("lint", &self.lint),
            ("typecheck", &self.typecheck),
        ] {
            if let Some(value) = value {
                validate_command(name, value)?;
            }
        }
        Ok(())
    }

    /// An absent override inherits its default. No fallback command is invented.
    pub fn with_overrides(&self, overrides: &Self) -> Self {
        Self {
            format: overrides.format.clone().or_else(|| self.format.clone()),
            build: overrides.build.clone().or_else(|| self.build.clone()),
            test: overrides.test.clone().or_else(|| self.test.clone()),
            lint: overrides.lint.clone().or_else(|| self.lint.clone()),
            typecheck: overrides
                .typecheck
                .clone()
                .or_else(|| self.typecheck.clone()),
        }
    }
}

pub(super) fn validate_command(name: &str, command: &str) -> Result<(), PackError> {
    if command.trim().is_empty() || command.contains('\0') {
        return Err(invalid(&format!("command {name}"), command));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSlots {
    pub planner: String,
    pub implementer: String,
    #[serde(default)]
    pub red_team: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewerDefinition {
    pub id: String,
    pub agent: String,
    pub always: bool,
    pub capabilities: Vec<Capability>,
    pub blocking: BlockingPolicy,
    /// Transitional labels for the Markdown consumer; Rust aggregates typed results.
    #[serde(default)]
    pub legacy_verdicts: BTreeMap<String, ReviewStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockingPolicy {
    pub statuses: Vec<ReviewStatus>,
    pub severities: Vec<Severity>,
}

impl BlockingPolicy {
    pub fn is_blocking(&self, review: &ReviewResult) -> bool {
        review.is_blocking()
            || self.statuses.contains(&review.status)
            || review
                .findings
                .iter()
                .any(|finding| self.severities.contains(&finding.severity))
    }

    fn validate(&self) -> Result<(), PackError> {
        if !self.statuses.contains(&ReviewStatus::Fail)
            || !self.severities.contains(&Severity::Blocker)
            || !self.severities.contains(&Severity::High)
        {
            return Err(invalid(
                "blocking policy",
                "must include fail, blocker and high",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedTeamGate {
    pub always: bool,
    pub capabilities: Vec<Capability>,
    pub blocking: BlockingPolicy,
    #[serde(default)]
    pub legacy_verdicts: BTreeMap<String, ReviewStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewPolicy {
    pub max_fix_rounds: u32,
    pub rerun_blocked_only: bool,
    pub resolve_warnings: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewSummary {
    pub blocked: Vec<String>,
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
}

impl ReviewSummary {
    pub fn is_blocking(&self) -> bool {
        !self.blocked.is_empty() || !self.missing.is_empty()
    }
}

impl PackManifest {
    pub fn from_json(input: &str) -> Result<Self, PackError> {
        let manifest: Self = serde_json::from_str(input)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), PackError> {
        if self.schema_version != PACK_SCHEMA_VERSION {
            return Err(PackError::SchemaVersion(self.schema_version));
        }
        validate_id("pack id", &self.id)?;
        if self.display_name.trim().is_empty() {
            return Err(invalid("display_name", &self.display_name));
        }
        self.commands.validate()?;
        for file in self.detection.files.iter().chain(&self.detection.any_files) {
            if file.is_empty()
                || file.split('/').any(|part| {
                    part.is_empty()
                        || part == "."
                        || part == ".."
                        || !part
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                })
            {
                return Err(invalid(
                    "detection file (relative path without globs)",
                    file,
                ));
            }
        }
        for agent in self.agent_references() {
            validate_id("agent id", agent)?;
        }
        let mut ids = HashSet::new();
        let mut agents = HashSet::new();
        for reviewer in &self.reviewers {
            validate_id("reviewer id", &reviewer.id)?;
            if !ids.insert(&reviewer.id) {
                return Err(PackError::Duplicate {
                    field: "reviewer id".into(),
                    id: reviewer.id.clone(),
                });
            }
            if !agents.insert(&reviewer.agent) {
                return Err(PackError::Duplicate {
                    field: "reviewer agent".into(),
                    id: reviewer.agent.clone(),
                });
            }
            if !reviewer.always && reviewer.capabilities.is_empty() {
                return Err(invalid("conditional reviewer capabilities", &reviewer.id));
            }
            reviewer.blocking.validate()?;
            validate_verdicts(&reviewer.legacy_verdicts)?;
        }
        if self.agents.red_team.is_some() != self.red_team.is_some() {
            return Err(invalid(
                "red_team",
                "slot and gate must be configured together",
            ));
        }
        if let Some(gate) = &self.red_team {
            if !gate.always && gate.capabilities.is_empty() {
                return Err(invalid("red_team capabilities", "empty conditional gate"));
            }
            gate.blocking.validate()?;
            validate_verdicts(&gate.legacy_verdicts)?;
        }
        if self.review_policy.max_fix_rounds == 0 {
            return Err(invalid("max_fix_rounds", "must be positive"));
        }
        Ok(())
    }

    pub fn agent_references(&self) -> Vec<&str> {
        let mut agents = vec![
            self.agents.planner.as_str(),
            self.agents.implementer.as_str(),
        ];
        agents.extend(self.agents.red_team.as_deref());
        agents.extend(self.reviewers.iter().map(|r| r.agent.as_str()));
        agents
    }

    pub fn select_reviewers(
        &self,
        capabilities: &HashSet<Capability>,
    ) -> Result<Vec<&ReviewerDefinition>, PackError> {
        self.validate()?;
        Ok(self
            .reviewers
            .iter()
            .filter(|r| r.always || r.capabilities.iter().any(|c| capabilities.contains(c)))
            .collect())
    }

    pub fn requires_red_team(&self, capabilities: &HashSet<Capability>) -> Result<bool, PackError> {
        self.validate()?;
        Ok(self
            .red_team
            .as_ref()
            .is_some_and(|g| g.always || g.capabilities.iter().any(|c| capabilities.contains(c))))
    }

    /// Results use reviewer IDs scoped to this pack. Missing reviews fail closed;
    /// duplicates and results outside the selected panel are rejected.
    pub fn aggregate_reviews(
        &self,
        capabilities: &HashSet<Capability>,
        results: &[ReviewResult],
    ) -> Result<ReviewSummary, PackError> {
        let selected = self.select_reviewers(capabilities)?;
        let mut seen = HashSet::new();
        for result in results {
            if !seen.insert(&result.reviewer) {
                return Err(PackError::Duplicate {
                    field: "review result".into(),
                    id: result.reviewer.clone(),
                });
            }
            if !selected.iter().any(|r| r.id == result.reviewer) {
                return Err(PackError::UnexpectedReview(result.reviewer.clone()));
            }
        }
        let mut summary = ReviewSummary {
            blocked: vec![],
            missing: vec![],
            warnings: vec![],
        };
        for reviewer in selected {
            match results.iter().find(|r| r.reviewer == reviewer.id) {
                None => summary.missing.push(reviewer.id.clone()),
                Some(result) => {
                    if reviewer.blocking.is_blocking(result) {
                        summary.blocked.push(reviewer.id.clone());
                    }
                    if result.status == ReviewStatus::Warn {
                        summary.warnings.push(reviewer.id.clone());
                    }
                }
            }
        }
        Ok(summary)
    }
}

fn validate_verdicts(verdicts: &BTreeMap<String, ReviewStatus>) -> Result<(), PackError> {
    if verdicts.keys().any(|label| label.trim().is_empty()) {
        return Err(invalid("legacy verdict", "empty label"));
    }
    Ok(())
}
