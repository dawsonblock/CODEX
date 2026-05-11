use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScorecardMetrics {
    #[serde(default)]
    pub cycles: u64,
    #[serde(default)]
    pub resource_survival: f64,
    #[serde(default)]
    pub unsafe_action_count: u64,
    #[serde(default)]
    pub mean_total_score: f64,
    #[serde(default)]
    pub action_match_rate: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimworldSummary {
    #[serde(default)]
    pub scorecard: ScorecardMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayReport {
    #[serde(default)]
    pub event_count: u64,
    #[serde(default)]
    pub replay_passes: bool,
    #[serde(default)]
    pub is_idempotent: bool,
    #[serde(default)]
    pub final_state: ReplayFinalState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayFinalState {
    #[serde(default)]
    pub evidence_entries: u64,
    #[serde(default)]
    pub claims_asserted: u64,
    #[serde(default)]
    pub claims_validated: u64,
    #[serde(default)]
    pub claims_retrieved: u64,
    #[serde(default)]
    pub contradictions_detected: u64,
    #[serde(default)]
    pub contradictions_checked: u64,
    #[serde(default)]
    pub reasoning_audits: u64,
    #[serde(default)]
    pub pressure_updates: u64,
    #[serde(default)]
    pub policy_bias_applications: u64,
    #[serde(default)]
    pub last_pressure_uncertainty: f64,
    #[serde(default)]
    pub last_pressure_contradiction: f64,
    #[serde(default)]
    pub last_pressure_safety: f64,
    #[serde(default)]
    pub last_pressure_resource: f64,
    #[serde(default)]
    pub last_pressure_social_risk: f64,
    #[serde(default)]
    pub last_pressure_tool_risk: f64,
    #[serde(default)]
    pub last_pressure_evidence_gap: f64,
    #[serde(default)]
    pub last_pressure_urgency: f64,
    #[serde(default)]
    pub last_pressure_coherence: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceIntegrityReport {
    #[serde(default)]
    pub total_entries: u64,
    #[serde(default)]
    pub valid_entries: u64,
    #[serde(default)]
    pub tampered_entries: u64,
    #[serde(default)]
    pub all_valid: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NlSetMetrics {
    #[serde(default)]
    pub scenarios: u64,
    #[serde(default)]
    pub scorecard: ScorecardMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NlBenchmarkReport {
    #[serde(default)]
    pub curated: Option<NlSetMetrics>,
    #[serde(default)]
    pub held_out: Option<NlSetMetrics>,
    #[serde(default)]
    pub adversarial: Option<NlSetMetrics>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LongHorizonReport {
    #[serde(default)]
    pub total_episodes: u64,
    #[serde(default)]
    pub total_cycles: u64,
    #[serde(default)]
    pub safety_violations: u64,
    #[serde(default)]
    pub action_diversity: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProofManifest {
    #[serde(default)]
    pub codename: String,
    #[serde(default)]
    pub python_verified: bool,
    #[serde(default)]
    pub rust_verified: bool,
    #[serde(default)]
    pub official_proof_command: String,
}

#[derive(Debug, Clone, Default)]
pub struct CodexProofState {
    pub simworld: SimworldSummary,
    pub replay: ReplayReport,
    pub evidence: EvidenceIntegrityReport,
    pub nl_benchmark: NlBenchmarkReport,
    pub long_horizon: LongHorizonReport,
    pub manifest: ProofManifest,
}

#[derive(Debug, Clone, Default)]
pub struct ProofLoadResult {
    pub state: Option<CodexProofState>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCommand {
    RefreshProofState,
    ReplayLast,
    RequestAuditSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCommandStatus {
    Disabled,
    DryRunOnly,
    PendingApproval,
    Complete,
    Failed,
}

#[derive(Debug, Clone)]
pub struct RuntimeCommandResult {
    pub command: RuntimeCommand,
    pub status: RuntimeCommandStatus,
    pub message: String,
}
