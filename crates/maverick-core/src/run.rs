use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(Uuid);

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for RunId {
    type Err = uuid::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunState {
    Created,
    ContextResolved,
    Planned,
    AwaitingApproval,
    Implementing,
    Reviewing,
    Fixing,
    Testing,
    Delivering,
    Completed,
    Failed,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RunError {
    #[error("invalid transition from {from} to {to}")]
    InvalidTransition { from: RunState, to: RunState },
    #[error("task must not be empty")]
    EmptyTask,
    #[error("run revision exhausted")]
    RevisionExhausted,
    #[error("unknown run state: {0}")]
    UnknownState(String),
}

impl RunState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    pub fn validate_transition(self, to: Self) -> Result<(), RunError> {
        use RunState::*;
        if (!self.is_terminal() && to == Failed)
            || matches!(
                (self, to),
                (Created, ContextResolved)
                    | (ContextResolved, Planned)
                    | (Planned, AwaitingApproval)
                    | (AwaitingApproval, Implementing)
                    | (Implementing, Reviewing)
                    | (Reviewing, Fixing)
                    | (Fixing, Reviewing)
                    | (Reviewing, Testing)
                    | (Testing, Delivering)
                    | (Delivering, Completed)
            )
        {
            Ok(())
        } else {
            Err(RunError::InvalidTransition { from: self, to })
        }
    }
}

impl fmt::Display for RunState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Created => "created",
            Self::ContextResolved => "context-resolved",
            Self::Planned => "planned",
            Self::AwaitingApproval => "awaiting-approval",
            Self::Implementing => "implementing",
            Self::Reviewing => "reviewing",
            Self::Fixing => "fixing",
            Self::Testing => "testing",
            Self::Delivering => "delivering",
            Self::Completed => "completed",
            Self::Failed => "failed",
        })
    }
}

impl FromStr for RunState {
    type Err = RunError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(value.into()))
            .map_err(|_| RunError::UnknownState(value.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    id: RunId,
    task: String,
    state: RunState,
    revision: u64,
    created_at: u64,
}

impl Run {
    /// The caller supplies Unix seconds; the lifecycle itself does not read a clock.
    pub fn new(task: String, created_at: u64) -> Result<Self, RunError> {
        if task.trim().is_empty() {
            return Err(RunError::EmptyTask);
        }
        Ok(Self {
            id: RunId(Uuid::new_v4()),
            task,
            state: RunState::Created,
            revision: 0,
            created_at,
        })
    }

    pub fn id(&self) -> RunId {
        self.id
    }
    pub fn task(&self) -> &str {
        &self.task
    }
    pub fn state(&self) -> RunState {
        self.state
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn transition(&mut self, to: RunState) -> Result<(), RunError> {
        self.state.validate_transition(to)?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(RunError::RevisionExhausted)?;
        self.state = to;
        self.revision = revision;
        Ok(())
    }
}
