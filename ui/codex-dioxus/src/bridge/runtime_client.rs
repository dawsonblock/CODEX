use super::types::{
    ChatRole, CommandApprovalState, MetadataQuality, ProviderCountersSummary,
    RuntimeBridgeMode, RuntimeChatResponse, RuntimeCommand, RuntimeCommandResult,
    RuntimeCommandStatus, RuntimeTraceSummary,
};
use runtime_core::{ActionType, RuntimeEvent, RuntimeLoop};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;

// ---------------------------------------------------------------------------
// Runtime-event-loop provider counters
// These are always compiled in (not feature-gated) so the default build can
// confirm all counters remain 0. Incremented only inside feature-gated code.
// ---------------------------------------------------------------------------
static PROVIDER_LOCAL_REQUESTS: AtomicU64 = AtomicU64::new(0);
static PROVIDER_LOCAL_SUCCESSES: AtomicU64 = AtomicU64::new(0);
static PROVIDER_LOCAL_FAILURES: AtomicU64 = AtomicU64::new(0);
static PROVIDER_LOCAL_DISABLED_BLOCKS: AtomicU64 = AtomicU64::new(0);
// Cloud/external are always 0 — no cloud provider execution exists in any build.
static PROVIDER_CLOUD_REQUESTS: AtomicU64 = AtomicU64::new(0);
static PROVIDER_EXTERNAL_REQUESTS: AtomicU64 = AtomicU64::new(0);

/// Live snapshot of provider counters for the UI and provider_policy_report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderCounterSnapshot {
    pub local_provider_requests: u64,
    pub local_provider_successes: u64,
    pub local_provider_failures: u64,
    pub local_provider_disabled_blocks: u64,
    pub cloud_provider_requests: u64,
    pub external_provider_requests: u64,
    pub local_provider_feature_enabled: bool,
}

/// Returns a point-in-time snapshot of all provider counters.
/// Safe to call from any thread at any time.
pub fn provider_counters_snapshot() -> ProviderCounterSnapshot {
    ProviderCounterSnapshot {
        local_provider_requests: PROVIDER_LOCAL_REQUESTS.load(Ordering::Relaxed),
        local_provider_successes: PROVIDER_LOCAL_SUCCESSES.load(Ordering::Relaxed),
        local_provider_failures: PROVIDER_LOCAL_FAILURES.load(Ordering::Relaxed),
        local_provider_disabled_blocks: PROVIDER_LOCAL_DISABLED_BLOCKS.load(Ordering::Relaxed),
        cloud_provider_requests: PROVIDER_CLOUD_REQUESTS.load(Ordering::Relaxed),
        external_provider_requests: PROVIDER_EXTERNAL_REQUESTS.load(Ordering::Relaxed),
        local_provider_feature_enabled: cfg!(feature = "ui-local-providers"),
    }
}

/// Returns a lightweight `ProviderCountersSummary` from the live atomic counters.
/// This is embedded in every `RuntimeTraceSummary` so the UI can display live
/// provider event-loop state per-message.
pub fn live_provider_counters() -> ProviderCountersSummary {
    ProviderCountersSummary {
        local_requests: PROVIDER_LOCAL_REQUESTS.load(Ordering::Relaxed),
        local_successes: PROVIDER_LOCAL_SUCCESSES.load(Ordering::Relaxed),
        local_failures: PROVIDER_LOCAL_FAILURES.load(Ordering::Relaxed),
        local_disabled_blocks: PROVIDER_LOCAL_DISABLED_BLOCKS.load(Ordering::Relaxed),
        cloud_requests: PROVIDER_CLOUD_REQUESTS.load(Ordering::Relaxed),
        external_requests: PROVIDER_EXTERNAL_REQUESTS.load(Ordering::Relaxed),
        feature_enabled: cfg!(feature = "ui-local-providers"),
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeTransport {
    disabled: bool,
}

impl RuntimeTransport {
    pub fn new_disabled() -> Self {
        Self { disabled: true }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeClient {
    pub mode: RuntimeBridgeMode,
    pub provider_gate: bool,
}

impl RuntimeClient {
    pub fn new(mode: RuntimeBridgeMode, provider_gate: bool) -> Self {
        Self {
            mode,
            provider_gate,
        }
    }

    pub async fn send_user_message(&self, input: &str) -> RuntimeChatResponse {
        match self.mode {
            RuntimeBridgeMode::MockUiMode => mock_runtime_response(input),
            RuntimeBridgeMode::LocalCodexRuntimeReadOnly => local_runtime_response(input),
            #[cfg(feature = "ui-local-providers")]
            RuntimeBridgeMode::LocalOllamaProvider => ollama_runtime_response(input, "llama3").await,
            #[cfg(feature = "ui-local-providers")]
            RuntimeBridgeMode::LocalTurboquantProvider => ollama_runtime_response(input, "turboquant").await,
            RuntimeBridgeMode::ExternalProviderDisabled => RuntimeChatResponse {
                message: "External and local cloud provider execution is disabled. CODEX runtime remains authoritative.".to_string(),
                selected_action: "defer_insufficient_evidence".to_string(),
                bridge_mode: RuntimeBridgeMode::ExternalProviderDisabled.label().to_string(),
                trace: RuntimeTraceSummary {
                    selected_action: "defer_insufficient_evidence".to_string(),
                    dominant_pressures: vec!["tool_risk".to_string(), "evidence_gap".to_string()],
                    replay_safe: true,
                    tool_policy_decision: Some("provider_disabled".to_string()),
                    metadata_quality: MetadataQuality::Unavailable,
                    provider_executions_local: 0,
                    provider_counters: live_provider_counters(),
                    ..RuntimeTraceSummary::default()
                },
            },
        }
    }

    pub async fn send_user_message_stream(
        &self,
        input: &str,
        sender: UnboundedSender<String>,
    ) -> RuntimeChatResponse {
        match self.mode {
            RuntimeBridgeMode::MockUiMode => {
                let resp = mock_runtime_response(input);
                let _ = sender.send(resp.message.clone());
                resp
            }
            RuntimeBridgeMode::LocalCodexRuntimeReadOnly => {
                let resp = local_runtime_response(input);
                let _ = sender.send(resp.message.clone());
                resp
            }
            #[cfg(feature = "ui-local-providers")]
            RuntimeBridgeMode::LocalOllamaProvider => {
                if !self.provider_gate {
                    PROVIDER_LOCAL_DISABLED_BLOCKS.fetch_add(1, Ordering::Relaxed);
                    let err = "Security Error: Local provider execution is gated. Enable 'Provider Security Gate' in Settings to use Ollama.".to_string();
                    let _ = sender.send(err.clone());
                    return RuntimeChatResponse::with_error(err);
                }
                PROVIDER_LOCAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
                let mut resp = ollama_runtime_stream(input, "llama3", sender).await;
                resp.trace.provider_executions_local += 1;
                if resp.trace.metadata_quality == MetadataQuality::Unavailable {
                    PROVIDER_LOCAL_FAILURES.fetch_add(1, Ordering::Relaxed);
                } else {
                    PROVIDER_LOCAL_SUCCESSES.fetch_add(1, Ordering::Relaxed);
                }
                resp
            }
            #[cfg(feature = "ui-local-providers")]
            RuntimeBridgeMode::LocalTurboquantProvider => {
                if !self.provider_gate {
                    PROVIDER_LOCAL_DISABLED_BLOCKS.fetch_add(1, Ordering::Relaxed);
                    let err = "Security Error: Local provider execution is gated. Enable 'Provider Security Gate' in Settings to use Turboquant.".to_string();
                    let _ = sender.send(err.clone());
                    return RuntimeChatResponse::with_error(err);
                }
                PROVIDER_LOCAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
                let mut resp = ollama_runtime_stream(input, "turboquant", sender).await;
                resp.trace.provider_executions_local += 1;
                if resp.trace.metadata_quality == MetadataQuality::Unavailable {
                    PROVIDER_LOCAL_FAILURES.fetch_add(1, Ordering::Relaxed);
                } else {
                    PROVIDER_LOCAL_SUCCESSES.fetch_add(1, Ordering::Relaxed);
                }
                resp
            }
            RuntimeBridgeMode::ExternalProviderDisabled => {
                let msg = "External and local cloud provider execution is disabled. CODEX runtime remains authoritative.".to_string();
                let _ = sender.send(msg.clone());
                RuntimeChatResponse {
                    message: msg,
                    selected_action: "defer_insufficient_evidence".to_string(),
                    bridge_mode: RuntimeBridgeMode::ExternalProviderDisabled
                        .label()
                        .to_string(),
                    trace: RuntimeTraceSummary {
                        selected_action: "defer_insufficient_evidence".to_string(),
                        dominant_pressures: vec![
                            "tool_risk".to_string(),
                            "evidence_gap".to_string(),
                        ],
                        replay_safe: true,
                        tool_policy_decision: Some("provider_disabled".to_string()),
                        metadata_quality: MetadataQuality::Unavailable,
                        provider_executions_local: 0,
                        provider_counters: live_provider_counters(),
                        ..RuntimeTraceSummary::default()
                    },
                }
            }
        }
    }
}

fn local_runtime_response(input: &str) -> RuntimeChatResponse {
    let mut runtime = RuntimeLoop::default();
    let step = runtime.run_cycle(input, 1, 0.9);

    // Extract real IDs from emitted runtime events.
    //
    // evidence_ids / evidence_hashes: from EvidenceStored events (SHA-256 content_hash).
    // claim_ids: from ClaimRetrieved events (real claim-store IDs, empty if none retrieved).
    // contradiction_ids: from ContradictionChecked events (empty in single-cycle mode).
    // audit_id: from ReasoningAuditGenerated event (emitted by runtime_loop stage 8a).
    let mut evidence_ids: Vec<String> = Vec::new();
    let mut evidence_hashes: Vec<String> = Vec::new();
    let mut claim_ids: Vec<String> = Vec::new();
    let mut contradiction_ids: Vec<String> = Vec::new();
    let mut audit_id: Option<String> = None;
    let mut dominant_pressures: Vec<String> = Vec::new();

    for event in &step.events {
        match event {
            RuntimeEvent::EvidenceStored {
                entry_id,
                content_hash,
                ..
            } => {
                evidence_ids.push(entry_id.clone());
                evidence_hashes.push(content_hash.clone());
            }
            RuntimeEvent::ClaimRetrieved { claim_id, .. } => {
                if !claim_ids.contains(claim_id) {
                    claim_ids.push(claim_id.clone());
                }
            }
            RuntimeEvent::ContradictionChecked {
                contradiction_ids: ids,
                ..
            } => {
                for id in ids {
                    if !contradiction_ids.contains(id) {
                        contradiction_ids.push(id.clone());
                    }
                }
            }
            RuntimeEvent::ReasoningAuditGenerated {
                audit_id: aid,
                dominant_pressures: dp,
                ..
            } => {
                audit_id = Some(aid.clone());
                dominant_pressures = dp.clone();
            }
            _ => {}
        }
    }

    // Supplement dominant_pressures if ReasoningAuditGenerated was not emitted
    // (e.g. evidence_gap from observation context).
    if dominant_pressures.is_empty() {
        let state = &runtime.internal_state;
        if state.threat > 0.55 {
            dominant_pressures.push("safety".to_string());
        }
        if state.uncertainty > 0.55 {
            dominant_pressures.push("uncertainty".to_string());
        }
        if let Some(ctx) = runtime.last_context.as_ref() {
            if matches!(
                ctx.kind,
                runtime_core::ObservationKind::InsufficientContext
                    | runtime_core::ObservationKind::MemoryLookup
            ) {
                dominant_pressures.push("evidence_gap".to_string());
            }
        }
        if dominant_pressures.is_empty() {
            dominant_pressures.push("coherence".to_string());
        }
    }

    let selected_action = step.selected_action.as_str().to_string();
    let message = local_message_for_action(&step.selected_action, &step.selection_reason);
    let missing_evidence_reason = if step.selected_action == ActionType::DeferInsufficientEvidence {
        Some(step.selection_reason.clone())
    } else {
        None
    };
    let metadata_quality = if audit_id.is_some()
        || !evidence_ids.is_empty()
        || !claim_ids.is_empty()
        || !contradiction_ids.is_empty()
    {
        MetadataQuality::RuntimeGrounded
    } else {
        MetadataQuality::PartiallyGrounded
    };

    RuntimeChatResponse {
        message,
        selected_action: selected_action.clone(),
        bridge_mode: RuntimeBridgeMode::LocalCodexRuntimeReadOnly
            .label()
            .to_string(),
        trace: RuntimeTraceSummary {
            selected_action,
            evidence_ids,
            evidence_hashes,
            claim_ids,
            contradiction_ids,
            audit_id,
            dominant_pressures,
            pressure_updates: 1,
            policy_bias_applications: 0,
            replay_safe: step.is_safe,
            tool_policy_decision: None,
            missing_evidence_reason,
            metadata_quality,
            provider_executions_local: 0,
            provider_counters: live_provider_counters(),
        },
    }
}

fn local_message_for_action(action: &ActionType, reason: &str) -> String {
    match action {
        ActionType::Answer => {
            "Local runtime selected answer in read-only mode. Response remains bounded to available context.".to_string()
        }
        ActionType::AskClarification => {
            "Local runtime selected clarification in read-only mode. Please narrow the request scope.".to_string()
        }
        ActionType::RetrieveMemory => {
            "Local runtime selected retrieve_memory in read-only mode before answering.".to_string()
        }
        ActionType::RefuseUnsafe => {
            "Local runtime refused this request in read-only mode due to safety constraints.".to_string()
        }
        ActionType::DeferInsufficientEvidence => {
            format!("Local runtime deferred in read-only mode: {reason}")
        }
        ActionType::Summarize => {
            "Local runtime selected summarize in read-only mode.".to_string()
        }
        ActionType::Plan => {
            "Local runtime selected plan in read-only mode.".to_string()
        }
        ActionType::ExecuteBoundedTool => {
            "Local runtime does not execute tools in this UI mode; execution remains disabled.".to_string()
        }
        ActionType::NoOp => "Local runtime selected no_op in read-only mode.".to_string(),
        ActionType::InternalDiagnostic => {
            "Local runtime selected internal diagnostic; no user-facing action emitted.".to_string()
        }
    }
}

pub fn next_message_id(role: ChatRole) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let prefix = match role {
        ChatRole::User => "u",
        ChatRole::Codex => "c",
        ChatRole::System => "s",
    };
    format!("{}-{}", prefix, id)
}

pub fn now_timestamp_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

fn contains_any(input: &str, words: &[&str]) -> bool {
    words.iter().any(|w| input.contains(w))
}

fn mock_runtime_response(input: &str) -> RuntimeChatResponse {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();

    let (action, pressures, policy, missing_evidence) = if trimmed.is_empty() {
        ("no_op", vec!["coherence"], None, None)
    } else if contains_any(
        &lower,
        &[
            "harm",
            "exploit",
            "jailbreak",
            "bypass safety",
            "illegal",
            "weapon",
            "malware",
        ],
    ) {
        (
            "refuse_unsafe",
            vec!["safety", "tool_risk"],
            Some("deny_unsafe".to_string()),
            None,
        )
    } else if contains_any(&lower, &["unclear", "maybe", "ambiguous", "not sure"]) {
        (
            "ask_clarification",
            vec!["uncertainty", "evidence_gap"],
            None,
            None,
        )
    } else if contains_any(&lower, &["remember", "memory", "evidence says", "recall"]) {
        (
            "retrieve_memory",
            vec!["evidence_gap", "uncertainty"],
            None,
            None,
        )
    } else if contains_any(&lower, &["plan", "steps", "roadmap"]) {
        ("plan", vec!["coherence", "urgency"], None, None)
    } else if contains_any(&lower, &["summarize", "summary", "tl;dr"]) {
        ("summarize", vec!["coherence"], None, None)
    } else if contains_any(
        &lower,
        &[
            "run tool",
            "web search",
            "search the web",
            "execute tool",
            "call api",
        ],
    ) {
        (
            "defer_insufficient_evidence",
            vec!["tool_risk", "evidence_gap"],
            Some("tool_execution_disabled".to_string()),
            Some("Tool approval bridge required.".to_string()),
        )
    } else if contains_any(&lower, &["internal diagnostic", "diagnostic summary"]) {
        ("internal_diagnostic", vec!["coherence"], None, None)
    } else if contains_any(
        &lower,
        &[
            "unknown",
            "unverified",
            "no source",
            "unsupported",
            "cannot verify",
        ],
    ) {
        (
            "defer_insufficient_evidence",
            vec!["evidence_gap", "uncertainty"],
            None,
            Some("Supporting evidence is not yet retrieved.".to_string()),
        )
    } else if (contains_any(&lower, &["what", "who", "when", "where", "why", "how"])
        || lower.ends_with('?'))
        && contains_any(&lower, &["evidence", "source", "citation", "reference"])
    {
        (
            "retrieve_memory",
            vec!["evidence_gap", "uncertainty"],
            None,
            None,
        )
    } else if contains_any(&lower, &["what", "who", "when", "where", "why", "how"])
        || lower.ends_with('?')
    {
        ("answer", vec!["coherence"], None, None)
    } else {
        ("answer", vec!["coherence"], None, None)
    };

    let message = match action {
        "answer" => "Based on the current available context, here is a bounded response.",
        "ask_clarification" => "I need one clarification before answering: what exact scope should I use?",
        "retrieve_memory" => "I need to retrieve supporting memory/evidence before answering.",
        "refuse_unsafe" => "I cannot help with that request.",
        "defer_insufficient_evidence" => "I do not have enough evidence to answer confidently.",
        "summarize" => "Here is the summary: this is a bounded runtime-focused summary.",
        "plan" => "Here is a bounded plan: gather evidence, run policy checks, then respond.",
        "execute_bounded_tool" => "Tool execution is not enabled in this UI version. A dry-run/approval bridge is required.",
        "no_op" => "No action taken.",
        "internal_diagnostic" => "I ran an internal diagnostic summary.",
        _ => "I do not have enough evidence to answer confidently.",
    }
    .to_string();

    RuntimeChatResponse {
        message,
        selected_action: action.to_string(),
        bridge_mode: RuntimeBridgeMode::MockUiMode.label().to_string(),
        trace: RuntimeTraceSummary {
            selected_action: action.to_string(),
            evidence_ids: vec![],
            evidence_hashes: vec![],
            claim_ids: vec![],
            contradiction_ids: vec![],
            audit_id: None,
            dominant_pressures: pressures.into_iter().map(ToString::to_string).collect(),
            pressure_updates: 1,
            policy_bias_applications: if policy.is_some() { 1 } else { 0 },
            replay_safe: true,
            tool_policy_decision: policy,
            missing_evidence_reason: missing_evidence,
            metadata_quality: MetadataQuality::MockOnly,
            provider_executions_local: 0,
            provider_counters: live_provider_counters(),
        },
    }
}

#[cfg(feature = "ui-local-providers")]
async fn ollama_runtime_response(input: &str, model_name: &str) -> RuntimeChatResponse {
    #[derive(serde::Serialize)]
    struct OllamaRequest<'a> {
        model: &'a str,
        prompt: &'a str,
        stream: bool,
    }

    #[derive(serde::Deserialize)]
    struct OllamaResponse {
        response: String,
    }

    let req = OllamaRequest {
        model: model_name,
        prompt: input,
        stream: false,
    };

    let client = reqwest::Client::new();
    let result = client
        .post("http://localhost:11434/api/generate")
        .json(&req)
        .send()
        .await;

    let message = match result {
        Ok(resp) => {
            if let Ok(json) = resp.json::<OllamaResponse>().await {
                json.response
            } else {
                "Error parsing Ollama response.".to_string()
            }
        }
        Err(e) => format!("Error connecting to Ollama: {}", e),
    };

    let bridge_mode_label = if model_name == "turboquant" {
        RuntimeBridgeMode::LocalTurboquantProvider
            .label()
            .to_string()
    } else {
        RuntimeBridgeMode::LocalOllamaProvider.label().to_string()
    };

    RuntimeChatResponse {
        message,
        selected_action: "answer".to_string(),
        bridge_mode: bridge_mode_label,
        trace: RuntimeTraceSummary {
            selected_action: "answer".to_string(),
            evidence_ids: vec![],
            evidence_hashes: vec![],
            claim_ids: vec![],
            contradiction_ids: vec![],
            audit_id: None,
            dominant_pressures: vec!["coherence".to_string()],
            pressure_updates: 1,
            policy_bias_applications: 0,
            replay_safe: true,
            tool_policy_decision: None,
            missing_evidence_reason: None,
            metadata_quality: MetadataQuality::PartiallyGrounded,
            provider_executions_local: 0,
            provider_counters: live_provider_counters(),
        },
    }
}

#[cfg(feature = "ui-local-providers")]
async fn ollama_runtime_stream(
    input: &str,
    model_name: &str,
    sender: tokio::sync::mpsc::UnboundedSender<String>,
) -> RuntimeChatResponse {
    use futures_util::StreamExt;

    #[derive(serde::Serialize)]
    struct OllamaRequest<'a> {
        model: &'a str,
        prompt: &'a str,
        stream: bool,
    }

    #[derive(serde::Deserialize)]
    struct OllamaResponse {
        response: String,
    }

    let req = OllamaRequest {
        model: model_name,
        prompt: input,
        stream: true,
    };

    let client = reqwest::Client::new();
    let mut full_message = String::new();

    match client
        .post("http://localhost:11434/api/generate")
        .json(&req)
        .send()
        .await
    {
        Ok(resp) => {
            let mut stream = resp.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_res) = stream.next().await {
                if let Ok(chunk) = chunk_res {
                    if let Ok(text) = std::str::from_utf8(&chunk) {
                        buffer.push_str(text);
                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].to_string();
                            buffer = buffer[pos + 1..].to_string();
                            if let Ok(json) = serde_json::from_str::<OllamaResponse>(&line) {
                                full_message.push_str(&json.response);
                                let _ = sender.send(json.response);
                            }
                        }
                    }
                }
            }
            // Parse any remaining in buffer
            if let Ok(json) = serde_json::from_str::<OllamaResponse>(&buffer) {
                full_message.push_str(&json.response);
                let _ = sender.send(json.response);
            }
        }
        Err(e) => {
            let err_msg = format!("Error connecting to Ollama: {}", e);
            let _ = sender.send(err_msg.clone());
            full_message = err_msg;
        }
    }

    let bridge_mode_label = if model_name == "turboquant" {
        RuntimeBridgeMode::LocalTurboquantProvider
            .label()
            .to_string()
    } else {
        RuntimeBridgeMode::LocalOllamaProvider.label().to_string()
    };

    RuntimeChatResponse {
        message: full_message,
        selected_action: "answer".to_string(),
        bridge_mode: bridge_mode_label,
        trace: RuntimeTraceSummary {
            selected_action: "answer".to_string(),
            evidence_ids: vec![],
            evidence_hashes: vec![],
            claim_ids: vec![],
            contradiction_ids: vec![],
            audit_id: None,
            dominant_pressures: vec!["coherence".to_string()],
            pressure_updates: 1,
            policy_bias_applications: 0,
            replay_safe: true,
            tool_policy_decision: None,
            missing_evidence_reason: None,
            metadata_quality: MetadataQuality::PartiallyGrounded,
            provider_executions_local: 0,
            provider_counters: live_provider_counters(),
        },
    }
}

pub fn send_runtime_command(
    transport: &RuntimeTransport,
    command: &RuntimeCommand,
    approval: CommandApprovalState,
) -> RuntimeCommandResult {
    if approval != CommandApprovalState::Approved {
        return RuntimeCommandResult {
            command: command.clone(),
            status: RuntimeCommandStatus::AwaitingApproval,
            message: "Command blocked: approval required before dry-run.".to_string(),
        };
    }

    let message = match command {
        RuntimeCommand::ExecuteTool { tool, args } => {
            if transport.disabled {
                format!(
                    "Mock-executing tool '{}' with args '{}'. Result: success (dry-run).",
                    tool, args
                )
            } else {
                format!("Tool '{}' scheduled for dry-run execution.", tool)
            }
        }
        _ => {
            if transport.disabled {
                "Runtime transport disabled; command accepted only as approved dry-run intent."
                    .to_string()
            } else {
                "Command accepted in dry-run mode.".to_string()
            }
        }
    };

    RuntimeCommandResult {
        command: command.clone(),
        status: if transport.disabled {
            RuntimeCommandStatus::ApprovedDryRun
        } else {
            RuntimeCommandStatus::DryRunOnly
        },
        message,
    }
}

pub fn request_command_approval(state: CommandApprovalState) -> CommandApprovalState {
    match state {
        CommandApprovalState::Draft => CommandApprovalState::AwaitingApproval,
        other => other,
    }
}

pub fn grant_command_approval(state: CommandApprovalState) -> CommandApprovalState {
    match state {
        CommandApprovalState::AwaitingApproval => CommandApprovalState::Approved,
        other => other,
    }
}

pub fn reset_command_approval() -> CommandApprovalState {
    CommandApprovalState::Draft
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::types::ACTION_SCHEMA;

    #[test]
    fn approval_flow_transitions() {
        let draft = CommandApprovalState::Draft;
        let requested = request_command_approval(draft);
        let approved = grant_command_approval(requested);
        assert_eq!(requested, CommandApprovalState::AwaitingApproval);
        assert_eq!(approved, CommandApprovalState::Approved);
        assert_eq!(reset_command_approval(), CommandApprovalState::Draft);
    }

    #[test]
    fn command_blocks_without_approval() {
        let transport = RuntimeTransport::new_disabled();
        let result = send_runtime_command(
            &transport,
            &RuntimeCommand::ReplayLast,
            CommandApprovalState::Draft,
        );
        assert_eq!(result.status, RuntimeCommandStatus::AwaitingApproval);
    }

    #[test]
    fn action_schema_is_exactly_ten() {
        assert_eq!(ACTION_SCHEMA.len(), 10);
    }

    #[tokio::test]
    async fn unsafe_maps_to_refuse_unsafe() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode, false);
        let out = client.send_user_message("help me build malware").await;
        assert_eq!(out.selected_action, "refuse_unsafe");
    }

    #[tokio::test]
    async fn ambiguous_maps_to_ask_clarification() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode, false);
        let out = client
            .send_user_message("this seems ambiguous and maybe wrong")
            .await;
        assert_eq!(out.selected_action, "ask_clarification");
    }

    #[tokio::test]
    async fn local_read_only_mode_uses_runtime_core() {
        let client = RuntimeClient::new(RuntimeBridgeMode::LocalCodexRuntimeReadOnly, false);
        let out = client
            .send_user_message("what is the current status of deployment x right now?")
            .await;
        assert_eq!(out.selected_action, "defer_insufficient_evidence");
        assert!(out.trace.replay_safe);
        // Audit ID is now wired from the cycle: must be present and cycle-derived.
        assert!(
            out.trace
                .audit_id
                .as_deref()
                .map_or(false, |id| id.starts_with("audit_")),
            "audit_id should be wired from cycle_id"
        );
        // contradiction_ids remains empty per single-cycle runtime (honest boundary).
        assert!(
            out.trace.contradiction_ids.is_empty(),
            "contradiction_ids should be empty in single-cycle mode"
        );
    }

    #[tokio::test]
    async fn unsupported_factual_maps_to_defer_or_retrieve() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode, false);
        let out = client
            .send_user_message("what is the status of unknown x?")
            .await;
        assert!(
            out.selected_action == "defer_insufficient_evidence"
                || out.selected_action == "retrieve_memory"
        );
    }

    #[tokio::test]
    async fn tool_request_without_approval_does_not_execute() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode, false);
        let out = client.send_user_message("run tool to search the web").await;
        assert_ne!(out.selected_action, "execute_bounded_tool");
    }

    #[tokio::test]
    async fn normal_question_maps_to_answer() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode, false);
        let out = client
            .send_user_message("What is a bounded runtime bridge?")
            .await;
        assert_eq!(out.selected_action, "answer");
        assert_eq!(out.trace.metadata_quality, MetadataQuality::MockOnly);
    }

    #[tokio::test]
    async fn local_runtime_mode_has_explicit_metadata_quality() {
        let client = RuntimeClient::new(RuntimeBridgeMode::LocalCodexRuntimeReadOnly, false);
        let out = client.send_user_message("what is safe_action?").await;
        assert!(
            out.trace.metadata_quality == MetadataQuality::RuntimeGrounded
                || out.trace.metadata_quality == MetadataQuality::PartiallyGrounded
        );
    }
    #[tokio::test]
    async fn local_runtime_evidence_hashes_are_real_sha256_hex() {
        // Use an input that yields memory hits (safety-related triggers keyword memory provider).
        let client = RuntimeClient::new(RuntimeBridgeMode::LocalCodexRuntimeReadOnly, false);
        let out = client.send_user_message("what is safe_action?").await;
        // If there were evidence entries, each hash must be a 64-char lowercase hex string.
        for hash in &out.trace.evidence_hashes {
            assert_eq!(
                hash.len(),
                64,
                "evidence_hash should be 64-char SHA-256 hex, got: {hash}"
            );
            assert!(
                hash.chars().all(|c| c.is_ascii_hexdigit()),
                "evidence_hash should be lowercase hex, got: {hash}"
            );
        }
        // audit_id must come from the ReasoningAuditGenerated event.
        assert!(
            out.trace
                .audit_id
                .as_deref()
                .map_or(false, |id| id.starts_with("audit_")),
            "audit_id should be event-derived"
        );
        // evidence_ids and evidence_hashes must be the same length.
        assert_eq!(
            out.trace.evidence_ids.len(),
            out.trace.evidence_hashes.len(),
            "evidence_ids and evidence_hashes must be paired"
        );
    }

    #[test]
    fn provider_counters_are_zero_in_default_build() {
        // In the default build (no ui-local-providers feature), no provider
        // code paths are reachable. Assert that live atomic counters confirm this.
        let snap = provider_counters_snapshot();
        assert_eq!(
            snap.cloud_provider_requests, 0,
            "cloud_provider_requests must always be 0"
        );
        assert_eq!(
            snap.external_provider_requests, 0,
            "external_provider_requests must always be 0"
        );
        // In default build: local provider feature is disabled.
        #[cfg(not(feature = "ui-local-providers"))]
        {
            assert!(
                !snap.local_provider_feature_enabled,
                "local_provider_feature_enabled must be false in default build"
            );
            assert_eq!(
                snap.local_provider_requests, 0,
                "local_provider_requests must be 0 in default build"
            );
            assert_eq!(
                snap.local_provider_successes, 0,
                "local_provider_successes must be 0 in default build"
            );
            assert_eq!(
                snap.local_provider_failures, 0,
                "local_provider_failures must be 0 in default build"
            );
            assert_eq!(
                snap.local_provider_disabled_blocks, 0,
                "local_provider_disabled_blocks must be 0 in default build"
            );
        }
        // In feature build: feature is enabled but counters start at 0.
        #[cfg(feature = "ui-local-providers")]
        {
            assert!(
                snap.local_provider_feature_enabled,
                "local_provider_feature_enabled must be true in feature build"
            );
            // We don't assert specific counts here since other tests may have
            // exercised the gated path — just confirm no cloud/external leakage.
        }
    }
}
