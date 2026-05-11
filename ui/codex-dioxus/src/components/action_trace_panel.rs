use crate::bridge::types::RuntimeTraceSummary;
use dioxus::prelude::*;

#[component]
pub fn ActionTracePanel(trace: Option<RuntimeTraceSummary>) -> Element {
    let Some(trace) = trace else {
        return rsx! {
            section { class: "card",
                h3 { "Action Trace" }
                p { class: "muted", "Not available in current runtime bridge." }
            }
        };
    };

    let is_grounded = matches!(trace.metadata_quality, crate::bridge::types::MetadataQuality::RuntimeGrounded);

    let evidence_ids = if trace.evidence_ids.is_empty() || !is_grounded {
        "not available in current runtime bridge".to_string()
    } else {
        trace.evidence_ids.join(", ")
    };
    let evidence_hashes = if trace.evidence_hashes.is_empty() || !is_grounded {
        "not available in current runtime bridge".to_string()
    } else {
        trace.evidence_hashes.join(", ")
    };
    let claim_ids = if trace.claim_ids.is_empty() || !is_grounded {
        "not available in current runtime bridge".to_string()
    } else {
        trace.claim_ids.join(", ")
    };
    let contradiction_ids = if trace.contradiction_ids.is_empty() || !is_grounded {
        "not available in current runtime bridge".to_string()
    } else {
        trace.contradiction_ids.join(", ")
    };
    
    let audit_label = if is_grounded { "runtime audit ID" } else { "UI trace ID" };
    let audit_id = trace.audit_id.unwrap_or_else(|| "none".to_string());

    let dominant_pressures = if trace.dominant_pressures.is_empty() {
        "not available in current runtime bridge".to_string()
    } else {
        trace.dominant_pressures.join(", ")
    };
    let tool_policy = trace
        .tool_policy_decision
        .unwrap_or_else(|| "not available in current runtime bridge".to_string());
    let missing_evidence = trace
        .missing_evidence_reason
        .unwrap_or_else(|| "not available in current runtime bridge".to_string());
    let metadata_quality = trace.metadata_quality.label();
    let show_partial_warning = matches!(
        trace.metadata_quality,
        crate::bridge::types::MetadataQuality::PartiallyGrounded
            | crate::bridge::types::MetadataQuality::Unavailable
    );

    rsx! {
        section { class: "card",
            h3 { "Action Trace" }
            if show_partial_warning {
                p {
                    class: "muted",
                    "This response is local-runtime routed, but evidence/claim/audit grounding is partial."
                }
            }
            ul { class: "list",
                li { "selected_action: {trace.selected_action}" }
                li { "metadata_quality: {metadata_quality}" }
                li { "replay_safe: {trace.replay_safe}" }
                li { "pressure_updates: {trace.pressure_updates}" }
                li { "policy_bias_applications: {trace.policy_bias_applications}" }
                li { "evidence_ids: {evidence_ids}" }
                li { "evidence_hashes: {evidence_hashes}" }
                li { "claim_ids: {claim_ids}" }
                li { "contradiction_ids: {contradiction_ids}" }
                li { "{audit_label}: {audit_id}" }
                li { "dominant_pressures: {dominant_pressures}" }
                li { "tool_policy_decision: {tool_policy}" }
                li { "missing_evidence_reason: {missing_evidence}" }
            }
        }
    }
}
