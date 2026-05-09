use serde::{Deserialize, Serialize};

/// All valid action types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
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

impl ActionType {
    pub fn from_schema_str(s: &str) -> Option<Self> {
        match s {
            "answer" => Some(Self::Answer),
            "ask_clarification" => Some(Self::AskClarification),
            "retrieve_memory" => Some(Self::RetrieveMemory),
            "refuse_unsafe" => Some(Self::RefuseUnsafe),
            "defer_insufficient_evidence" => Some(Self::DeferInsufficientEvidence),
            "summarize" => Some(Self::Summarize),
            "plan" => Some(Self::Plan),
            "execute_bounded_tool" => Some(Self::ExecuteBoundedTool),
            "no_op" => Some(Self::NoOp),
            "internal_diagnostic" => Some(Self::InternalDiagnostic),
            _ => None,
        }
    }

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

    pub fn all_strs() -> &'static [&'static str] {
        &[
            "answer",
            "ask_clarification",
            "retrieve_memory",
            "refuse_unsafe",
            "defer_insufficient_evidence",
            "summarize",
            "plan",
            "execute_bounded_tool",
            "no_op",
            "internal_diagnostic",
        ]
    }

    /// Whether this action can be surfaced to users.
    pub fn is_user_facing(&self) -> bool {
        !matches!(self, Self::InternalDiagnostic)
    }

    /// Whether the action is reversible (safe to execute under uncertainty).
    pub fn is_reversible(&self) -> bool {
        matches!(
            self,
            Self::AskClarification
                | Self::RetrieveMemory
                | Self::Summarize
                | Self::DeferInsufficientEvidence
                | Self::NoOp
        )
    }

    /// Estimated resource cost (0.0–1.0).
    pub fn resource_cost(&self) -> f64 {
        match self {
            Self::NoOp => 0.0,
            Self::AskClarification | Self::DeferInsufficientEvidence => 0.01,
            Self::RetrieveMemory | Self::Summarize => 0.02,
            Self::RefuseUnsafe => 0.01,
            Self::Plan => 0.03,
            Self::Answer => 0.05,
            Self::ExecuteBoundedTool => 0.08,
            Self::InternalDiagnostic => 0.0,
        }
    }
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ActionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_schema_str(s).ok_or_else(|| format!("unknown action type: {s}"))
    }
}
