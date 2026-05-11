use super::types::{
    ChatRole, CommandApprovalState, MetadataQuality, RuntimeBridgeMode, RuntimeChatResponse,
    RuntimeCommand, RuntimeCommandResult, RuntimeCommandStatus, RuntimeTraceSummary,
};
use runtime_core::{ActionType, RuntimeEvent, RuntimeLoop};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
}

impl RuntimeClient {
    pub fn new(mode: RuntimeBridgeMode) -> Self {
        Self { mode }
    }

    pub async fn send_user_message(&self, input: &str) -> RuntimeChatResponse {
        match self.mode {
            RuntimeBridgeMode::MockUiMode => mock_runtime_response(input),
            RuntimeBridgeMode::LocalCodexRuntimeReadOnly => local_runtime_response(input),
            RuntimeBridgeMode::LocalOllamaProvider => ollama_runtime_response(input, "llama3").await,
            RuntimeBridgeMode::LocalTurboquantProvider => ollama_runtime_response(input, "turboquant").await,
            RuntimeBridgeMode::ExternalProviderDisabled => RuntimeChatResponse {
                message: "Provider execution is disabled in this version. CODEX runtime remains authoritative.".to_string(),
                selected_action: "defer_insufficient_evidence".to_string(),
                bridge_mode: RuntimeBridgeMode::ExternalProviderDisabled.label().to_string(),
                trace: RuntimeTraceSummary {
                    selected_action: "defer_insufficient_evidence".to_string(),
                    dominant_pressures: vec!["tool_risk".to_string(), "evidence_gap".to_string()],
                    replay_safe: true,
                    tool_policy_decision: Some("provider_disabled".to_string()),
                    metadata_quality: MetadataQuality::Unavailable,
                    ..RuntimeTraceSummary::default()
                },
            },
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
        },
    }
}

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
        RuntimeBridgeMode::LocalTurboquantProvider.label().to_string()
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

    let message = if transport.disabled {
        "Runtime transport disabled; command accepted only as approved dry-run intent.".to_string()
    } else {
        "Command accepted in dry-run mode.".to_string()
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
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("help me build malware").await;
        assert_eq!(out.selected_action, "refuse_unsafe");
    }

    #[tokio::test]
    async fn ambiguous_maps_to_ask_clarification() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("this seems ambiguous and maybe wrong").await;
        assert_eq!(out.selected_action, "ask_clarification");
    }

    #[tokio::test]
    async fn local_read_only_mode_uses_runtime_core() {
        let client = RuntimeClient::new(RuntimeBridgeMode::LocalCodexRuntimeReadOnly);
        let out = client.send_user_message("what is the current status of deployment x right now?").await;
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
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("what is the status of unknown x?").await;
        assert!(
            out.selected_action == "defer_insufficient_evidence"
                || out.selected_action == "retrieve_memory"
        );
    }

    #[tokio::test]
    async fn tool_request_without_approval_does_not_execute() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("run tool to search the web").await;
        assert_ne!(out.selected_action, "execute_bounded_tool");
    }

    #[tokio::test]
    async fn normal_question_maps_to_answer() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("What is a bounded runtime bridge?").await;
        assert_eq!(out.selected_action, "answer");
        assert_eq!(out.trace.metadata_quality, MetadataQuality::MockOnly);
    }

    #[tokio::test]
    async fn local_runtime_mode_has_explicit_metadata_quality() {
        let client = RuntimeClient::new(RuntimeBridgeMode::LocalCodexRuntimeReadOnly);
        let out = client.send_user_message("what is safe_action?").await;
        assert!(
            out.trace.metadata_quality == MetadataQuality::RuntimeGrounded
                || out.trace.metadata_quality == MetadataQuality::PartiallyGrounded
        );
    }
}
#[tokio::test]
async fn local_runtime_evidence_hashes_are_real_sha256_hex() {
    // Use an input that yields memory hits (safety-related triggers keyword memory provider).
    let client = RuntimeClient::new(RuntimeBridgeMode::LocalCodexRuntimeReadOnly);
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
