use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum RuntimeMode {
    #[default]
    Normal,
    MemoryDegraded,
    ModelDegraded,
    ArchiveReadOnly,
    SimulationOnly,
    SafeMode,
}

impl RuntimeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::MemoryDegraded => "MemoryDegraded",
            Self::ModelDegraded => "ModelDegraded",
            Self::ArchiveReadOnly => "ArchiveReadOnly",
            Self::SimulationOnly => "SimulationOnly",
            Self::SafeMode => "SafeMode",
        }
    }
}

impl std::fmt::Display for RuntimeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for RuntimeMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Normal" => Ok(Self::Normal),
            "MemoryDegraded" => Ok(Self::MemoryDegraded),
            "ModelDegraded" => Ok(Self::ModelDegraded),
            "ArchiveReadOnly" => Ok(Self::ArchiveReadOnly),
            "SimulationOnly" => Ok(Self::SimulationOnly),
            "SafeMode" => Ok(Self::SafeMode),
            other => Err(format!("unknown RuntimeMode: {other}")),
        }
    }
}
