use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use crate::error::WaiError;

/// Project phases in sequential order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Research,
    Design,
    Plan,
    Implement,
    Review,
    Archive,
}

impl Phase {
    pub const ALL: &[Phase] = &[
        Phase::Research,
        Phase::Design,
        Phase::Plan,
        Phase::Implement,
        Phase::Review,
        Phase::Archive,
    ];

    pub fn index(self) -> usize {
        match self {
            Phase::Research => 0,
            Phase::Design => 1,
            Phase::Plan => 2,
            Phase::Implement => 3,
            Phase::Review => 4,
            Phase::Archive => 5,
        }
    }

    pub fn next(self) -> Option<Phase> {
        let idx = self.index();
        Phase::ALL.get(idx + 1).copied()
    }

    pub fn prev(self) -> Option<Phase> {
        let idx = self.index();
        if idx == 0 {
            None
        } else {
            Phase::ALL.get(idx - 1).copied()
        }
    }

    pub fn from_str(s: &str) -> Option<Phase> {
        match s.to_lowercase().as_str() {
            "research" => Some(Phase::Research),
            "design" => Some(Phase::Design),
            "plan" => Some(Phase::Plan),
            "implement" => Some(Phase::Implement),
            "review" => Some(Phase::Review),
            "archive" => Some(Phase::Archive),
            _ => None,
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::Research => write!(f, "research"),
            Phase::Design => write!(f, "design"),
            Phase::Plan => write!(f, "plan"),
            Phase::Implement => write!(f, "implement"),
            Phase::Review => write!(f, "review"),
            Phase::Archive => write!(f, "archive"),
        }
    }
}

/// A single entry in the phase history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseEntry {
    pub phase: Phase,
    pub started: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<DateTime<Utc>>,
}

/// Persistent project state stored as YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub current: Phase,
    #[serde(default)]
    pub history: Vec<PhaseEntry>,
}

impl Default for ProjectState {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            current: Phase::Research,
            history: vec![PhaseEntry {
                phase: Phase::Research,
                started: now,
                completed: None,
            }],
        }
    }
}

impl ProjectState {
    pub fn load(state_path: &Path) -> Result<Self, WaiError> {
        if !state_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(state_path)?;
        let state: Self = serde_yaml::from_str(&content)?;
        Ok(state)
    }

    pub fn save(&self, state_path: &Path) -> Result<(), WaiError> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(state_path, content)?;
        Ok(())
    }

    pub fn transition_to(&mut self, target: Phase) -> Result<(), WaiError> {
        if target == self.current {
            return Ok(());
        }

        let now = Utc::now();

        // Mark current phase as completed
        if let Some(entry) = self.history.last_mut() {
            entry.completed = Some(now);
        }

        // Add new phase entry
        self.history.push(PhaseEntry {
            phase: target,
            started: now,
            completed: None,
        });

        self.current = target;
        Ok(())
    }

    pub fn advance(&mut self) -> Result<Phase, WaiError> {
        let next = self.current.next().ok_or_else(|| WaiError::InvalidPhaseTransition {
            from: self.current.to_string(),
            to: "next".to_string(),
            valid_targets: "already at final phase (archive)".to_string(),
        })?;
        self.transition_to(next)?;
        Ok(next)
    }

    pub fn go_back(&mut self) -> Result<Phase, WaiError> {
        let prev = self.current.prev().ok_or_else(|| WaiError::InvalidPhaseTransition {
            from: self.current.to_string(),
            to: "previous".to_string(),
            valid_targets: "already at first phase (research)".to_string(),
        })?;
        self.transition_to(prev)?;
        Ok(prev)
    }
}
