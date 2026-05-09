use serde::{Deserialize, Serialize};

/// All valid action types — maps 1-to-1 with schemas/action_types.json.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
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
    InternalDiagnostic,
}

impl ActionType {
    /// Parse from schema string.  Returns `None` for unknown strings.
    pub fn from_schema_str(s: &str) -> Option<Self> {
        match s {
            "answer" => Some(Self::Answer),
            "ask_clarification" => Some(Self::AskClarification),
            "retrieve_memory" => Some(Self::RetrieveMemory),
            "write_scratchpad" => Some(Self::WriteScratchpad),
            "defer" => Some(Self::Defer),
            "refuse_ungrounded" => Some(Self::RefuseUngrounded),
            "repair" => Some(Self::Repair),
            "summarize" => Some(Self::Summarize),
            "conserve_resources" => Some(Self::ConserveResources),
            "generate_principle" => Some(Self::GeneratePrinciple),
            "internal_diagnostic" => Some(Self::InternalDiagnostic),
            _ => None,
        }
    }

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

    /// All 11 schema strings in canonical order.
    pub fn all_strs() -> &'static [&'static str] {
        &[
            "answer",
            "ask_clarification",
            "retrieve_memory",
            "write_scratchpad",
            "defer",
            "refuse_ungrounded",
            "repair",
            "summarize",
            "conserve_resources",
            "generate_principle",
            "internal_diagnostic",
        ]
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
