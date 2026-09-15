use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewResult {
    pub reviewer: String,
    pub status: ReviewStatus,
    pub findings: Vec<Finding>,
}

impl ReviewResult {
    pub fn is_blocking(&self) -> bool {
        self.status.is_blocking() || self.findings.iter().any(Finding::is_blocking)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewStatus {
    Pass,
    Warn,
    Fail,
}

impl ReviewStatus {
    pub fn is_blocking(self) -> bool {
        self == Self::Fail
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub severity: Severity,
    pub rule: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub message: String,
    pub suggested_fix: Option<String>,
}

impl Finding {
    pub fn is_blocking(&self) -> bool {
        self.severity.is_blocking()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Blocker,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn is_blocking(self) -> bool {
        matches!(self, Self::Blocker | Self::High)
    }
}
