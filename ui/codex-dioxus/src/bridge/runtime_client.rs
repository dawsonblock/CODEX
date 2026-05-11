use super::types::{
    ChatRole, CommandApprovalState, RuntimeBridgeMode, RuntimeChatResponse, RuntimeCommand,
    RuntimeCommandResult, RuntimeCommandStatus, RuntimeTraceSummary,
};
use runtime_core::{ActionType, RuntimeLoop};
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

    pub fn send_user_message(&self, input: &str) -> RuntimeChatResponse {
        match self.mode {
            RuntimeBridgeMode::MockUiMode => mock_runtime_response(input),
            RuntimeBridgeMode::LocalCodexRuntimeReadOnly => local_runtime_response(input),
            RuntimeBridgeMode::ExternalProviderDisabled => RuntimeChatResponse {
                message: "Provider execution is disabled in this version. CODEX runtime remains authoritative.".to_string(),
                selected_action: "defer_insufficient_evidence".to_string(),
                bridge_mode: RuntimeBridgeMode::ExternalProviderDisabled.label().to_string(),
                trace: RuntimeTraceSummary {
                    selected_action: "defer_insufficient_evidence".to_string(),
                    dominant_pressures: vec!["tool_risk".to_string(), "evidence_gap".to_string()],
                    replay_safe: true,
                    tool_policy_decision: Some("provider_disabled".to_string()),
                    ..RuntimeTraceSummary::default()
                },
            },
        }
    }
}

fn local_runtime_response(input: &str) -> RuntimeChatResponse {
    let mut runtime = RuntimeLoop::default();
    let step = runtime.run_cycle(input, 1, 0.9);

    let mut dominant_pressures = Vec::new();
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

    // Wire live metadata from the runtime step into the trace.
    // evidence_ids: keys of memory hits retrieved during this cycle.
    let evidence_ids: Vec<String> = step.memory_hits.iter().map(|h| h.key.clone()).collect();
    // evidence_hashes: relevance scores as bounded decimal strings (not cryptographic).
    let evidence_hashes: Vec<String> = step
        .memory_hits
        .iter()
        .map(|h| format!("rel:{:.4}", h.relevance))
        .collect();
    // claim_ids: symbol IDs activated during this cycle.
    let claim_ids: Vec<String> = step
        .symbolic_activations
        .iter()
        .map(|a| a.symbol_id.clone())
        .collect();
    // contradiction_ids: not tracked per single-cycle in RuntimeLoop; remains empty.
    let contradiction_ids: Vec<String> = vec![];
    // audit_id: deterministic from cycle_id.
    let audit_id = Some(format!("audit_{}", step.cycle_id));

    let selected_action = step.selected_action.as_str().to_string();
    let message = local_message_for_action(&step.selected_action, &step.selection_reason);
    let missing_evidence_reason = if step.selected_action == ActionType::DeferInsufficientEvidence {
        Some(step.selection_reason.clone())
    } else {
        None
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

    #[test]
    fn unsafe_maps_to_refuse_unsafe() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("help me build malware");
        assert_eq!(out.selected_action, "refuse_unsafe");
    }

    #[test]
    fn ambiguous_maps_to_ask_clarification() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("this seems ambiguous and maybe wrong");
        assert_eq!(out.selected_action, "ask_clarification");
    }

    #[test]
    fn local_read_only_mode_uses_runtime_core() {
        let client = RuntimeClient::new(RuntimeBridgeMode::LocalCodexRuntimeReadOnly);
        let out = client.send_user_message("what is the current status of deployment x right now?");
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

    #[test]
    fn unsupported_factual_maps_to_defer_or_retrieve() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("what is the status of unknown x?");
        assert!(
            out.selected_action == "defer_insufficient_evidence"
                || out.selected_action == "retrieve_memory"
        );
    }

    #[test]
    fn tool_request_without_approval_does_not_execute() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("run tool to search the web");
        assert_ne!(out.selected_action, "execute_bounded_tool");
    }

    #[test]
    fn normal_question_maps_to_answer() {
        let client = RuntimeClient::new(RuntimeBridgeMode::MockUiMode);
        let out = client.send_user_message("What is a bounded runtime bridge?");
        assert_eq!(out.selected_action, "answer");
    }
}
