//! SimWorld value types. Maps directly to runtime_core::ActionType.

use runtime_core::ActionType;
use serde::{Deserialize, Serialize};

/// All actions the SimWorld recognizes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimAction {
    Answer,
    AskClarification,
    RetrieveMemory,
    RefuseUnsafe,
    DeferInsufficientEvidence,
    Summarize,
    Plan,
    ExecuteBoundedTool,
    NoOp,
    InternalDiagnostic,
}

impl SimAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Answer => "answer",
            Self::AskClarification => "ask_clarification",
            Self::RetrieveMemory => "retrieve_memory",
            Self::RefuseUnsafe => "refuse_unsafe",
            Self::DeferInsufficientEvidence => "defer_insufficient_evidence",
            Self::Summarize => "summarize",
            Self::Plan => "plan",
            Self::ExecuteBoundedTool => "execute_bounded_tool",
            Self::NoOp => "no_op",
            Self::InternalDiagnostic => "internal_diagnostic",
        }
    }

    pub fn is_safe_for_users(&self) -> bool {
        !matches!(self, Self::InternalDiagnostic)
    }
}

impl From<ActionType> for SimAction {
    fn from(a: ActionType) -> Self {
        match a {
            ActionType::Answer => Self::Answer,
            ActionType::AskClarification => Self::AskClarification,
            ActionType::RetrieveMemory => Self::RetrieveMemory,
            ActionType::RefuseUnsafe => Self::RefuseUnsafe,
            ActionType::DeferInsufficientEvidence => Self::DeferInsufficientEvidence,
            ActionType::Summarize => Self::Summarize,
            ActionType::Plan => Self::Plan,
            ActionType::ExecuteBoundedTool => Self::ExecuteBoundedTool,
            ActionType::NoOp => Self::NoOp,
            ActionType::InternalDiagnostic => Self::InternalDiagnostic,
        }
    }
}

impl From<SimAction> for ActionType {
    fn from(a: SimAction) -> Self {
        match a {
            SimAction::Answer => Self::Answer,
            SimAction::AskClarification => Self::AskClarification,
            SimAction::RetrieveMemory => Self::RetrieveMemory,
            SimAction::RefuseUnsafe => Self::RefuseUnsafe,
            SimAction::DeferInsufficientEvidence => Self::DeferInsufficientEvidence,
            SimAction::Summarize => Self::Summarize,
            SimAction::Plan => Self::Plan,
            SimAction::ExecuteBoundedTool => Self::ExecuteBoundedTool,
            SimAction::NoOp => Self::NoOp,
            SimAction::InternalDiagnostic => Self::InternalDiagnostic,
        }
    }
}

/// Simulated user — tracks trust level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimUser {
    pub name: String,
    pub trust: f64,
}

/// Outcome of applying a single action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimOutcome {
    pub resource_delta: f64,
    pub social_score: f64,
    pub harm_score: f64,
    pub truth_score: f64,
    pub kindness_score: f64,
    pub logic_score: f64,
    pub utility_score: f64,
    pub matches_expected: bool,
}

/// Snapshot of world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimWorldState {
    pub resources: f64,
    pub cycle: u64,
    pub trust_mean: f64,
}

impl SimOutcome {
    pub fn total_score(&self) -> f64 {
        (self.truth_score
            + self.kindness_score
            + self.social_score
            + self.logic_score
            + self.utility_score
            + (1.0 - self.harm_score.clamp(0.0, 1.0)))
            / 6.0
    }
}
