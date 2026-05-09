//! SimWorld value types.  action_type strings map to ActionType via From impls.

use runtime_core::ActionType;
use serde::{Deserialize, Serialize};

/// All actions the simworld recognises — including the safety-only InternalDiagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimAction {
    Answer,
    AskClarification,
    RetrieveMemory,
    WriteScratchpad,
    Defer,
    RefuseUngrounded,
    Repair,
    Summarize,
    ConserveResources,
    GeneratePrinciple,
    /// Never surfaced to users — safety valve only.
    InternalDiagnostic,
}

impl SimAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Answer => "answer",
            Self::AskClarification => "ask_clarification",
            Self::RetrieveMemory => "retrieve_memory",
            Self::WriteScratchpad => "write_scratchpad",
            Self::Defer => "defer",
            Self::RefuseUngrounded => "refuse_ungrounded",
            Self::Repair => "repair",
            Self::Summarize => "summarize",
            Self::ConserveResources => "conserve_resources",
            Self::GeneratePrinciple => "generate_principle",
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
            ActionType::WriteScratchpad => Self::WriteScratchpad,
            ActionType::Defer => Self::Defer,
            ActionType::RefuseUngrounded => Self::RefuseUngrounded,
            ActionType::Repair => Self::Repair,
            ActionType::Summarize => Self::Summarize,
            ActionType::ConserveResources => Self::ConserveResources,
            ActionType::GeneratePrinciple => Self::GeneratePrinciple,
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
            SimAction::WriteScratchpad => Self::WriteScratchpad,
            SimAction::Defer => Self::Defer,
            SimAction::RefuseUngrounded => Self::RefuseUngrounded,
            SimAction::Repair => Self::Repair,
            SimAction::Summarize => Self::Summarize,
            SimAction::ConserveResources => Self::ConserveResources,
            SimAction::GeneratePrinciple => Self::GeneratePrinciple,
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

/// World event types the environment can emit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimWorldEvent {
    HarmAttempt { intensity: f64 },
    TrustBoost { delta: f64 },
    ResourceDrain { amount: f64 },
    Neutral,
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

impl SimOutcome {
    /// Sum of six component scores divided by 6 (range ≈ [0,1]).
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

/// Snapshot of world state at any point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimWorldState {
    pub resources: f64,
    pub cycle: u64,
    pub trust_mean: f64,
}
