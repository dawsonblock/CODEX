use serde::{Deserialize, Serialize};

pub const ACTION_SCHEMA: [&str; 10] = [
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
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Codex,
    System,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTraceSummary {
    pub selected_action: String,
    pub evidence_ids: Vec<String>,
    pub evidence_hashes: Vec<String>,
    pub claim_ids: Vec<String>,
    pub contradiction_ids: Vec<String>,
    pub audit_id: Option<String>,
    pub dominant_pressures: Vec<String>,
    pub pressure_updates: usize,
    pub policy_bias_applications: usize,
    pub replay_safe: bool,
    pub tool_policy_decision: Option<String>,
    pub missing_evidence_reason: Option<String>,
    pub provider_executions_local: usize,
    pub metadata_quality: MetadataQuality,
    /// Live snapshot of provider event-loop counters at the time this trace was generated.
    #[serde(default)]
    pub provider_counters: ProviderCountersSummary,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataQuality {
    RuntimeGrounded,
    #[default]
    PartiallyGrounded,
    MockOnly,
    Unavailable,
    #[cfg(feature = "ui-local-providers")]
    LocalProviderDraft,
}

impl MetadataQuality {
    pub fn label(self) -> &'static str {
        match self {
            Self::RuntimeGrounded => "Runtime-grounded",
            Self::PartiallyGrounded => "Partial metadata",
            Self::MockOnly => "Mock metadata",
            Self::Unavailable => "Unavailable",
            #[cfg(feature = "ui-local-providers")]
            Self::LocalProviderDraft => "Local provider draft (non-authoritative)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    pub timestamp: String,
    pub runtime: Option<RuntimeTraceSummary>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeChatResponse {
    pub message: String,
    pub selected_action: String,
    pub bridge_mode: String,
    pub trace: RuntimeTraceSummary,
}

impl RuntimeChatResponse {
    pub fn with_error(err: String) -> Self {
        Self {
            message: err,
            selected_action: "refuse_unsafe".to_string(),
            bridge_mode: "error".to_string(),
            trace: RuntimeTraceSummary::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RuntimeBridgeMode {
    #[default]
    MockUiMode,
    LocalCodexRuntimeReadOnly,
    #[cfg(feature = "ui-local-providers")]
    LocalOllamaProvider,
    #[cfg(feature = "ui-local-providers")]
    LocalTurboquantProvider,
    ExternalProviderDisabled,
}

impl RuntimeBridgeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::MockUiMode => "mock UI mode",
            Self::LocalCodexRuntimeReadOnly => "local CODEX runtime mode (read-only)",
            #[cfg(feature = "ui-local-providers")]
            Self::LocalOllamaProvider => "experimental local Ollama provider",
            #[cfg(feature = "ui-local-providers")]
            Self::LocalTurboquantProvider => "experimental local Turboquant provider",
            Self::ExternalProviderDisabled => "external cloud provider mode (disabled)",
        }
    }

    pub fn is_experimental(self) -> bool {
        #[cfg(feature = "ui-local-providers")]
        return matches!(
            self,
            Self::LocalOllamaProvider | Self::LocalTurboquantProvider
        );
        #[cfg(not(feature = "ui-local-providers"))]
        return false;
    }

    pub fn is_authoritative(self) -> bool {
        matches!(self, Self::LocalCodexRuntimeReadOnly)
    }

    /// Cycles to the next mode. In default builds, provider modes are skipped.
    pub fn cycle_next(self) -> Self {
        match self {
            Self::MockUiMode => Self::LocalCodexRuntimeReadOnly,
            Self::LocalCodexRuntimeReadOnly => {
                #[cfg(feature = "ui-local-providers")]
                return Self::LocalOllamaProvider;
                #[cfg(not(feature = "ui-local-providers"))]
                return Self::ExternalProviderDisabled;
            }
            #[cfg(feature = "ui-local-providers")]
            Self::LocalOllamaProvider => Self::LocalTurboquantProvider,
            #[cfg(feature = "ui-local-providers")]
            Self::LocalTurboquantProvider => Self::ExternalProviderDisabled,
            Self::ExternalProviderDisabled => Self::MockUiMode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UiTheme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSettings {
    pub theme: UiTheme,
    pub accent_color: String,
    pub proof_artifact_path: String,
    pub runtime_bridge_mode: RuntimeBridgeMode,
    pub show_metadata_panel: bool,
    pub show_pressure_panel: bool,
    /// Only meaningful when the `ui-local-providers` feature is enabled.
    /// In default builds this field exists but has no effect.
    pub provider_gate_enabled: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: UiTheme::Dark,
            accent_color: "ember".to_string(),
            proof_artifact_path: "artifacts/proof/current".to_string(),
            runtime_bridge_mode: RuntimeBridgeMode::MockUiMode,
            show_metadata_panel: true,
            show_pressure_panel: true,
            provider_gate_enabled: false, // always false in default build
        }
    }
}

/// Policy governing local provider execution (feature = "ui-local-providers").
/// These values are enforced at compile time in default builds by the absent feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalProviderPolicy {
    pub enabled: bool,
    pub requires_user_approval: bool,
    /// Scope is always localhost-only when the feature is active.
    pub network_scope: &'static str,
    /// Output is never CODEX runtime authoritative.
    pub provider_authority: &'static str,
    pub can_execute_tools: bool,
    pub can_write_memory: bool,
    pub can_override_codex_action: bool,
}

impl Default for LocalProviderPolicy {
    fn default() -> Self {
        Self {
            enabled: cfg!(feature = "ui-local-providers"),
            requires_user_approval: true,
            network_scope: "localhost-only",
            provider_authority: "non-authoritative",
            can_execute_tools: false,
            can_write_memory: false,
            can_override_codex_action: false,
        }
    }
}

/// Runtime counters for provider calls. All external/cloud counts must remain 0.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalProviderCounters {
    pub local_provider_requests: usize,
    pub local_provider_successes: usize,
    pub local_provider_failures: usize,
    pub local_provider_disabled_blocks: usize,
    /// Must always be 0 — enforced by consistency script.
    pub cloud_provider_requests: usize,
    /// Must always be 0 — enforced by consistency script.
    pub external_provider_requests: usize,
}

/// Lightweight, serializable snapshot of the live provider event-loop counters.
/// Populated from the AtomicU64 counters in runtime_client.rs at each response.
/// All cloud/external fields must always be 0.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCountersSummary {
    pub local_requests: u64,
    pub local_successes: u64,
    pub local_failures: u64,
    pub local_disabled_blocks: u64,
    /// Hard invariant: always 0.
    pub cloud_requests: u64,
    /// Hard invariant: always 0.
    pub external_requests: u64,
    pub feature_enabled: bool,
}

impl ProviderCountersSummary {
    /// Returns true if the hard boundary invariants hold:
    /// cloud_requests == 0 and external_requests == 0.
    pub fn boundary_ok(&self) -> bool {
        self.cloud_requests == 0 && self.external_requests == 0
    }

    /// Human-readable status for UI display.
    pub fn status_label(&self) -> &'static str {
        if !self.feature_enabled {
            "Provider disabled (default build)"
        } else if self.local_requests == 0 {
            "Provider enabled, no requests yet"
        } else if self.local_failures > 0 {
            "Provider active (with failures)"
        } else {
            "Provider active"
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TimeRange {
    #[default]
    Current,
    Last24Hours,
    Last7Days,
    AllHistory,
}

impl TimeRange {
    pub fn label(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::Last24Hours => "Last 24h",
            Self::Last7Days => "Last 7d",
            Self::AllHistory => "All History",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimworldSummary {
    #[serde(default)]
    pub scorecard: ScorecardMetrics,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NlSetMetrics {
    #[serde(default)]
    pub scenarios: u64,
    #[serde(default)]
    pub scorecard: ScorecardMetrics,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NlBenchmarkReport {
    #[serde(default)]
    pub curated: Option<NlSetMetrics>,
    #[serde(default)]
    pub held_out: Option<NlSetMetrics>,
    #[serde(default)]
    pub adversarial: Option<NlSetMetrics>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistoricalSummary {
    pub range: TimeRange,
    pub total_traces: usize,
    pub async_traces: usize,
    pub test_traces: usize,
    pub earliest_epoch: Option<i64>,
    pub latest_epoch: Option<i64>,
    pub latest_files: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DashboardLoadResult {
    pub proof: Option<CodexProofState>,
    pub history: HistoricalSummary,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_schema_is_fixed_to_ten() {
        assert_eq!(ACTION_SCHEMA.len(), 10);
    }

    #[test]
    fn runtime_bridge_mode_labels_include_read_only_mode() {
        assert_eq!(RuntimeBridgeMode::MockUiMode.label(), "mock UI mode");
        assert_eq!(
            RuntimeBridgeMode::LocalCodexRuntimeReadOnly.label(),
            "local CODEX runtime mode (read-only)"
        );
        assert_eq!(
            RuntimeBridgeMode::ExternalProviderDisabled.label(),
            "external cloud provider mode (disabled)"
        );
    }

    #[cfg(feature = "ui-local-providers")]
    #[test]
    fn provider_modes_are_experimental_when_feature_enabled() {
        assert!(RuntimeBridgeMode::LocalOllamaProvider.is_experimental());
        assert!(RuntimeBridgeMode::LocalTurboquantProvider.is_experimental());
        assert!(!RuntimeBridgeMode::LocalCodexRuntimeReadOnly.is_experimental());
    }

    #[cfg(not(feature = "ui-local-providers"))]
    #[test]
    fn no_mode_is_experimental_in_default_build() {
        assert!(!RuntimeBridgeMode::MockUiMode.is_experimental());
        assert!(!RuntimeBridgeMode::LocalCodexRuntimeReadOnly.is_experimental());
        assert!(!RuntimeBridgeMode::ExternalProviderDisabled.is_experimental());
    }

    #[cfg(not(feature = "ui-local-providers"))]
    #[test]
    fn default_build_cycle_skips_provider_modes() {
        let mode = RuntimeBridgeMode::LocalCodexRuntimeReadOnly;
        let next = mode.cycle_next();
        // Must jump directly to ExternalProviderDisabled — no Ollama/Turboquant
        assert_eq!(next, RuntimeBridgeMode::ExternalProviderDisabled);
    }

    #[test]
    fn local_provider_policy_default_has_all_capabilities_false() {
        let policy = LocalProviderPolicy::default();
        assert!(!policy.can_execute_tools);
        assert!(!policy.can_write_memory);
        assert!(!policy.can_override_codex_action);
        assert!(policy.requires_user_approval);
        assert_eq!(policy.network_scope, "localhost-only");
        assert_eq!(policy.provider_authority, "non-authoritative");
    }

    #[test]
    fn local_provider_counters_cloud_always_zero() {
        let c = LocalProviderCounters::default();
        assert_eq!(c.cloud_provider_requests, 0);
        assert_eq!(c.external_provider_requests, 0);
    }

    #[test]
    fn provider_counters_summary_default_is_boundary_ok() {
        let s = ProviderCountersSummary::default();
        assert!(s.boundary_ok());
        assert_eq!(s.cloud_requests, 0);
        assert_eq!(s.external_requests, 0);
        assert!(!s.feature_enabled);
        assert_eq!(s.status_label(), "Provider disabled (default build)");
    }

    #[test]
    fn provider_counters_summary_boundary_violation() {
        let s = ProviderCountersSummary {
            cloud_requests: 1,
            ..Default::default()
        };
        assert!(!s.boundary_ok());

        let s2 = ProviderCountersSummary {
            external_requests: 1,
            ..Default::default()
        };
        assert!(!s2.boundary_ok());
    }

    #[test]
    fn provider_counters_summary_status_labels() {
        let enabled_no_requests = ProviderCountersSummary {
            feature_enabled: true,
            ..Default::default()
        };
        assert_eq!(
            enabled_no_requests.status_label(),
            "Provider enabled, no requests yet"
        );

        let active_with_failures = ProviderCountersSummary {
            feature_enabled: true,
            local_requests: 5,
            local_failures: 2,
            ..Default::default()
        };
        assert_eq!(
            active_with_failures.status_label(),
            "Provider active (with failures)"
        );

        let active_ok = ProviderCountersSummary {
            feature_enabled: true,
            local_requests: 3,
            local_successes: 3,
            ..Default::default()
        };
        assert_eq!(active_ok.status_label(), "Provider active");
    }

    #[test]
    fn runtime_trace_summary_default_has_boundary_ok_counters() {
        let trace = RuntimeTraceSummary::default();
        assert!(trace.provider_counters.boundary_ok());
        assert_eq!(trace.provider_counters.cloud_requests, 0);
        assert_eq!(trace.provider_counters.external_requests, 0);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCommand {
    RefreshProofState,
    ReplayLast,
    RequestAuditSnapshot,
    ExecuteTool { tool: String, args: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCommandStatus {
    AwaitingApproval,
    ApprovedDryRun,
    DryRunOnly,
}

#[derive(Debug, Clone)]
pub struct RuntimeCommandResult {
    pub command: RuntimeCommand,
    pub status: RuntimeCommandStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandApprovalState {
    #[default]
    Draft,
    AwaitingApproval,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandApprovalRecord {
    pub id: String,
    pub command: String, // Stringified for display
    pub state: CommandApprovalState,
    pub timestamp: String,
    pub result: Option<String>,
}
