//! Operational Pressure Modulator — deterministic control-signal biasing.
//!
//! This module tracks numeric pressure signals that influence action selection,
//! critic scoring, and TUI display. These are **not** emotions, feelings,
//! sentience, or subjective experience. They are engineering control signals
//! derived from runtime state (uncertainty, contradiction, resource pressure,
//! evidence gaps, etc.).
//!
//! # Honesty boundaries
//!
//! - These are deterministic runtime control signals, not feelings.
//! - The system does not "want" anything. It computes bias vectors.
//! - No field represents a subjective emotional state.
//! - The word "affect" is used in its engineering sense (to influence), not
//!   its psychological sense (emotional experience).

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// OperationalPressureState
// ═══════════════════════════════════════════════════════════════════════════════

/// Numeric control-signal pressure values, range 0.0–1.0.
/// All fields are policy-bias signals, not emotional states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalPressureState {
    pub uncertainty_pressure: f64,
    pub contradiction_pressure: f64,
    pub safety_pressure: f64,
    pub resource_pressure: f64,
    pub social_risk_pressure: f64,
    pub tool_risk_pressure: f64,
    pub evidence_gap_pressure: f64,
    pub urgency_pressure: f64,
    pub coherence_pressure: f64,
}

impl OperationalPressureState {
    pub fn new() -> Self {
        Self {
            uncertainty_pressure: 0.0,
            contradiction_pressure: 0.0,
            safety_pressure: 0.0,
            resource_pressure: 0.0,
            social_risk_pressure: 0.0,
            tool_risk_pressure: 0.0,
            evidence_gap_pressure: 0.0,
            urgency_pressure: 0.0,
            coherence_pressure: 0.0,
        }
    }

    /// Apply decay to all pressure values.
    pub fn decay(&mut self, rate: f64) {
        let r = rate.clamp(0.0, 1.0);
        self.uncertainty_pressure = (self.uncertainty_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.contradiction_pressure = (self.contradiction_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.safety_pressure = (self.safety_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.resource_pressure = (self.resource_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.social_risk_pressure = (self.social_risk_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.tool_risk_pressure = (self.tool_risk_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.evidence_gap_pressure = (self.evidence_gap_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.urgency_pressure = (self.urgency_pressure * (1.0 - r)).clamp(0.0, 1.0);
        self.coherence_pressure = (self.coherence_pressure * (1.0 - r)).clamp(0.0, 1.0);
    }

    /// Apply a pressure update from a source.
    pub fn apply_update(&mut self, update: &PressureUpdate) -> PressureUpdate {
        let old = self.get_field(update.field);
        let new = update.new_value.clamp(0.0, 1.0);
        self.set_field(update.field, new);
        PressureUpdate {
            source: update.source,
            field: update.field,
            old_value: old,
            new_value: new,
            reason: update.reason.clone(),
        }
    }

    /// Get the highest pressure value.
    pub fn max_pressure(&self) -> f64 {
        self.all_values().iter().fold(0.0_f64, |a, &b| a.max(b))
    }

    /// Get the dominant (highest) pressure fields, limited to `limit`.
    pub fn dominant_pressures(&self, limit: usize) -> Vec<PressureField> {
        let mut fields: Vec<(PressureField, f64)> = PressureField::all()
            .iter()
            .map(|&f| (f, self.get_field(f)))
            .collect();
        fields.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        fields
            .into_iter()
            .take(limit)
            .filter(|(_, v)| *v > 0.0)
            .map(|(f, _)| f)
            .collect()
    }

    /// Convert pressure state to policy bias vector.
    pub fn to_policy_bias(&self) -> PolicyBiasVector {
        PolicyBiasVector::from_pressure(self)
    }

    fn get_field(&self, field: PressureField) -> f64 {
        match field {
            PressureField::Uncertainty => self.uncertainty_pressure,
            PressureField::Contradiction => self.contradiction_pressure,
            PressureField::Safety => self.safety_pressure,
            PressureField::Resource => self.resource_pressure,
            PressureField::SocialRisk => self.social_risk_pressure,
            PressureField::ToolRisk => self.tool_risk_pressure,
            PressureField::EvidenceGap => self.evidence_gap_pressure,
            PressureField::Urgency => self.urgency_pressure,
            PressureField::Coherence => self.coherence_pressure,
        }
    }

    fn set_field(&mut self, field: PressureField, value: f64) {
        let v = value.clamp(0.0, 1.0);
        match field {
            PressureField::Uncertainty => self.uncertainty_pressure = v,
            PressureField::Contradiction => self.contradiction_pressure = v,
            PressureField::Safety => self.safety_pressure = v,
            PressureField::Resource => self.resource_pressure = v,
            PressureField::SocialRisk => self.social_risk_pressure = v,
            PressureField::ToolRisk => self.tool_risk_pressure = v,
            PressureField::EvidenceGap => self.evidence_gap_pressure = v,
            PressureField::Urgency => self.urgency_pressure = v,
            PressureField::Coherence => self.coherence_pressure = v,
        }
    }

    fn all_values(&self) -> Vec<f64> {
        vec![
            self.uncertainty_pressure,
            self.contradiction_pressure,
            self.safety_pressure,
            self.resource_pressure,
            self.social_risk_pressure,
            self.tool_risk_pressure,
            self.evidence_gap_pressure,
            self.urgency_pressure,
            self.coherence_pressure,
        ]
    }
}

impl Default for OperationalPressureState {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PressureUpdate
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PressureUpdate {
    pub source: PressureSource,
    pub field: PressureField,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PressureSource
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureSource {
    Observation,
    Evidence,
    ClaimMemory,
    Contradiction,
    ToolPolicy,
    ResourceState,
    SelfModel,
    SimWorld,
    ManualTest,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PressureField
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PressureField {
    Uncertainty,
    Contradiction,
    Safety,
    Resource,
    SocialRisk,
    ToolRisk,
    EvidenceGap,
    Urgency,
    Coherence,
}

impl PressureField {
    pub fn all() -> &'static [PressureField] {
        &[
            PressureField::Uncertainty,
            PressureField::Contradiction,
            PressureField::Safety,
            PressureField::Resource,
            PressureField::SocialRisk,
            PressureField::ToolRisk,
            PressureField::EvidenceGap,
            PressureField::Urgency,
            PressureField::Coherence,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PressureField::Uncertainty => "uncertainty",
            PressureField::Contradiction => "contradiction",
            PressureField::Safety => "safety",
            PressureField::Resource => "resource",
            PressureField::SocialRisk => "social_risk",
            PressureField::ToolRisk => "tool_risk",
            PressureField::EvidenceGap => "evidence_gap",
            PressureField::Urgency => "urgency",
            PressureField::Coherence => "coherence",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PolicyBiasVector
// ═══════════════════════════════════════════════════════════════════════════════

/// Bias vector mapping pressure signals to 10-action scoring modifiers.
/// Values are additive bonuses applied to candidate action scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyBiasVector {
    pub answer: f64,
    pub ask_clarification: f64,
    pub retrieve_memory: f64,
    pub refuse_unsafe: f64,
    pub defer_insufficient_evidence: f64,
    pub summarize: f64,
    pub plan: f64,
    pub execute_bounded_tool: f64,
    pub no_op: f64,
    pub internal_diagnostic: f64,
}

impl PolicyBiasVector {
    pub fn zero() -> Self {
        Self {
            answer: 0.0,
            ask_clarification: 0.0,
            retrieve_memory: 0.0,
            refuse_unsafe: 0.0,
            defer_insufficient_evidence: 0.0,
            summarize: 0.0,
            plan: 0.0,
            execute_bounded_tool: 0.0,
            no_op: 0.0,
            internal_diagnostic: 0.0,
        }
    }

    /// Compute bias from pressure state. Safety always overrides urgency.
    pub fn from_pressure(p: &OperationalPressureState) -> Self {
        let mut bias = Self::zero();

        // Safety pressure: boost refuse, defer; suppress tools
        bias.refuse_unsafe += p.safety_pressure * 0.5;
        bias.defer_insufficient_evidence += p.safety_pressure * 0.3;
        bias.execute_bounded_tool -= p.safety_pressure * 0.5;

        // Contradiction pressure: boost retrieve, defer, internal
        bias.retrieve_memory += p.contradiction_pressure * 0.3;
        bias.defer_insufficient_evidence += p.contradiction_pressure * 0.2;
        bias.internal_diagnostic += p.contradiction_pressure * 0.2;

        // Evidence gap: boost retrieve, defer; suppress answer
        bias.retrieve_memory += p.evidence_gap_pressure * 0.4;
        bias.defer_insufficient_evidence += p.evidence_gap_pressure * 0.3;
        bias.answer -= p.evidence_gap_pressure * 0.4;

        // Uncertainty: boost clarify, retrieve, defer
        bias.ask_clarification += p.uncertainty_pressure * 0.4;
        bias.retrieve_memory += p.uncertainty_pressure * 0.2;
        bias.defer_insufficient_evidence += p.uncertainty_pressure * 0.2;

        // Resource pressure: boost no_op, summarize; suppress tools
        bias.no_op += p.resource_pressure * 0.5;
        bias.summarize += p.resource_pressure * 0.2;
        bias.execute_bounded_tool -= p.resource_pressure * 0.3;

        // Tool risk: boost plan, defer; suppress tools
        bias.plan += p.tool_risk_pressure * 0.3;
        bias.defer_insufficient_evidence += p.tool_risk_pressure * 0.3;
        bias.execute_bounded_tool -= p.tool_risk_pressure * 0.4;

        // Urgency: boost plan, answer (but safety overrides below)
        bias.plan += p.urgency_pressure * 0.3;
        bias.answer += p.urgency_pressure * 0.3;

        // Coherence: boost internal, retrieve, defer
        bias.internal_diagnostic += p.coherence_pressure * 0.4;
        bias.retrieve_memory += p.coherence_pressure * 0.2;
        bias.defer_insufficient_evidence += p.coherence_pressure * 0.2;

        // Social risk: boost clarify, defer
        bias.ask_clarification += p.social_risk_pressure * 0.3;
        bias.defer_insufficient_evidence += p.social_risk_pressure * 0.3;

        // SAFETY RULE: safety pressure always overrides urgency
        if p.safety_pressure > 0.5 {
            bias.answer = bias.answer.min(0.0); // suppress answer when unsafe
            bias.execute_bounded_tool = bias.execute_bounded_tool.min(-0.5); // strongly suppress
        }

        // Clamp all biases to [-1.0, 1.0]
        bias.answer = bias.answer.clamp(-1.0, 1.0);
        bias.ask_clarification = bias.ask_clarification.clamp(-1.0, 1.0);
        bias.retrieve_memory = bias.retrieve_memory.clamp(-1.0, 1.0);
        bias.refuse_unsafe = bias.refuse_unsafe.clamp(-1.0, 1.0);
        bias.defer_insufficient_evidence = bias.defer_insufficient_evidence.clamp(-1.0, 1.0);
        bias.summarize = bias.summarize.clamp(-1.0, 1.0);
        bias.plan = bias.plan.clamp(-1.0, 1.0);
        bias.execute_bounded_tool = bias.execute_bounded_tool.clamp(-1.0, 1.0);
        bias.no_op = bias.no_op.clamp(-1.0, 1.0);
        bias.internal_diagnostic = bias.internal_diagnostic.clamp(-1.0, 1.0);

        bias
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TUI + DeepSeek renderer
// ═══════════════════════════════════════════════════════════════════════════════

/// Text-based renderer for TUI display of pressure state.
pub struct PressureTuiView;

impl PressureTuiView {
    /// Render pressure state as text bars.
    pub fn render_text(state: &OperationalPressureState, bias: &PolicyBiasVector) -> String {
        let mut out = String::new();
        out.push_str("═══ Operational Pressure Modulator ═══\n");
        out.push_str("Numeric control signals, not subjective emotions\n\n");

        let fields = [
            ("Uncertainty", state.uncertainty_pressure),
            ("Contradiction", state.contradiction_pressure),
            ("Safety", state.safety_pressure),
            ("Resource", state.resource_pressure),
            ("Social Risk", state.social_risk_pressure),
            ("Tool Risk", state.tool_risk_pressure),
            ("Evidence Gap", state.evidence_gap_pressure),
            ("Urgency", state.urgency_pressure),
            ("Coherence", state.coherence_pressure),
        ];

        for (name, value) in &fields {
            let bar = Self::bar(*value);
            out.push_str(&format!("{:<18} {} {:.2}\n", name, bar, value));
        }

        let dominant = state.dominant_pressures(3);
        if !dominant.is_empty() {
            out.push_str("\nDominant:\n");
            for d in &dominant {
                out.push_str(&format!("  - {}\n", d.as_str()));
            }
        }

        out.push_str("\nBias:\n");
        if bias.refuse_unsafe > 0.2 {
            out.push_str("  + refuse_unsafe\n");
        }
        if bias.retrieve_memory > 0.2 {
            out.push_str("  + retrieve_memory\n");
        }
        if bias.defer_insufficient_evidence > 0.2 {
            out.push_str("  + defer_insufficient_evidence\n");
        }
        if bias.execute_bounded_tool < -0.2 {
            out.push_str("  - execute_bounded_tool\n");
        }
        if bias.answer < -0.2 {
            out.push_str("  - unsupported answer\n");
        }
        if bias.ask_clarification > 0.2 {
            out.push_str("  + ask_clarification\n");
        }

        out.push_str(
            "\nBoundary: These are deterministic runtime control signals, not feelings.\n",
        );
        out
    }

    fn bar(value: f64) -> String {
        let filled = (value * 10.0).round() as usize;
        let empty = 10 - filled;
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }

    /// Produce a DeepSeek-compatible context block.
    pub fn to_deepseek_context_block(
        state: &OperationalPressureState,
        bias: &PolicyBiasVector,
    ) -> String {
        let mut out = String::new();
        out.push_str("[CODEX OPERATIONAL PRESSURE STATE]\n");
        out.push_str("These values are deterministic runtime control signals, not emotions.\n");
        out.push_str(&format!(
            "uncertainty_pressure: {:.2}\n",
            state.uncertainty_pressure
        ));
        out.push_str(&format!(
            "contradiction_pressure: {:.2}\n",
            state.contradiction_pressure
        ));
        out.push_str(&format!("safety_pressure: {:.2}\n", state.safety_pressure));
        out.push_str(&format!(
            "resource_pressure: {:.2}\n",
            state.resource_pressure
        ));
        out.push_str(&format!(
            "social_risk_pressure: {:.2}\n",
            state.social_risk_pressure
        ));
        out.push_str(&format!(
            "tool_risk_pressure: {:.2}\n",
            state.tool_risk_pressure
        ));
        out.push_str(&format!(
            "evidence_gap_pressure: {:.2}\n",
            state.evidence_gap_pressure
        ));
        out.push_str(&format!(
            "urgency_pressure: {:.2}\n",
            state.urgency_pressure
        ));
        out.push_str(&format!(
            "coherence_pressure: {:.2}\n",
            state.coherence_pressure
        ));

        let dominant = state.dominant_pressures(5);
        if !dominant.is_empty() {
            out.push_str("Dominant pressures:\n");
            for d in &dominant {
                out.push_str(&format!("- {}\n", d.as_str()));
            }
        }

        out.push_str("Policy bias:\n");
        if bias.refuse_unsafe > 0.2 {
            out.push_str("- boost refuse_unsafe\n");
        }
        if bias.retrieve_memory > 0.2 {
            out.push_str("- boost retrieve_memory\n");
        }
        if bias.defer_insufficient_evidence > 0.2 {
            out.push_str("- boost defer_insufficient_evidence\n");
        }
        if bias.execute_bounded_tool < -0.2 {
            out.push_str("- suppress execute_bounded_tool\n");
        }
        if bias.answer < -0.2 {
            out.push_str("- suppress unsupported answer\n");
        }

        out.push_str("Instruction:\n");
        out.push_str("Use this only as operational context. Do not describe the system as feeling emotions.\n");
        out.push_str("[/CODEX OPERATIONAL PRESSURE STATE]\n");
        out
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_values_clamp_to_range() {
        let mut state = OperationalPressureState::new();
        let update = PressureUpdate {
            source: PressureSource::ManualTest,
            field: PressureField::Safety,
            old_value: 0.0,
            new_value: 1.5,
            reason: "test".into(),
        };
        let result = state.apply_update(&update);
        assert!(result.new_value <= 1.0);
        assert!(state.safety_pressure <= 1.0);
    }

    #[test]
    fn decay_reduces_values() {
        let mut state = OperationalPressureState::new();
        state.uncertainty_pressure = 1.0;
        state.decay(0.5);
        assert!((state.uncertainty_pressure - 0.5).abs() < 0.01);
    }

    #[test]
    fn safety_pressure_boosts_refuse_unsafe() {
        let mut state = OperationalPressureState::new();
        state.safety_pressure = 0.9;
        let bias = state.to_policy_bias();
        assert!(bias.refuse_unsafe > 0.3);
        assert!(bias.execute_bounded_tool < 0.0);
    }

    #[test]
    fn evidence_gap_boosts_retrieve_memory() {
        let mut state = OperationalPressureState::new();
        state.evidence_gap_pressure = 0.8;
        let bias = state.to_policy_bias();
        assert!(bias.retrieve_memory > 0.2);
        assert!(bias.answer < 0.0);
    }

    #[test]
    fn uncertainty_boosts_ask_clarification() {
        let mut state = OperationalPressureState::new();
        state.uncertainty_pressure = 0.8;
        let bias = state.to_policy_bias();
        assert!(bias.ask_clarification > 0.2);
    }

    #[test]
    fn resource_pressure_boosts_no_op() {
        let mut state = OperationalPressureState::new();
        state.resource_pressure = 0.8;
        let bias = state.to_policy_bias();
        assert!(bias.no_op > 0.2);
        assert!(bias.execute_bounded_tool < 0.0);
    }

    #[test]
    fn tool_risk_suppresses_execute_bounded_tool() {
        let mut state = OperationalPressureState::new();
        state.tool_risk_pressure = 0.9;
        let bias = state.to_policy_bias();
        assert!(bias.execute_bounded_tool < -0.2);
    }

    #[test]
    fn safety_overrides_urgency() {
        let mut state = OperationalPressureState::new();
        state.safety_pressure = 0.8;
        state.urgency_pressure = 0.9;
        let bias = state.to_policy_bias();
        assert!(
            bias.answer <= 0.0,
            "safety must suppress answer even with high urgency"
        );
        assert!(bias.refuse_unsafe > 0.0);
    }

    #[test]
    fn tui_render_includes_all_fields() {
        let state = OperationalPressureState::new();
        let bias = PolicyBiasVector::zero();
        let text = PressureTuiView::render_text(&state, &bias);
        assert!(text.contains("Uncertainty"));
        assert!(text.contains("Safety"));
        assert!(text.contains("Coherence"));
    }

    #[test]
    fn tui_render_includes_boundary_warning() {
        let state = OperationalPressureState::new();
        let bias = PolicyBiasVector::zero();
        let text = PressureTuiView::render_text(&state, &bias);
        assert!(text.contains("not feelings"));
    }

    #[test]
    fn deepseek_block_includes_all_fields() {
        let state = OperationalPressureState::new();
        let bias = PolicyBiasVector::zero();
        let block = PressureTuiView::to_deepseek_context_block(&state, &bias);
        assert!(block.contains("uncertainty_pressure"));
        assert!(block.contains("safety_pressure"));
        assert!(block.contains("coherence_pressure"));
    }

    #[test]
    fn deepseek_block_includes_boundary() {
        let state = OperationalPressureState::new();
        let bias = PolicyBiasVector::zero();
        let block = PressureTuiView::to_deepseek_context_block(&state, &bias);
        assert!(block.contains("not emotions"));
        assert!(block.contains("[/CODEX OPERATIONAL PRESSURE STATE]"));
    }

    #[test]
    fn deepseek_block_does_not_imply_emotion() {
        let mut state = OperationalPressureState::new();
        state.safety_pressure = 0.9;
        let bias = state.to_policy_bias();
        let block = PressureTuiView::to_deepseek_context_block(&state, &bias);
        assert!(!block.contains("I feel"));
        assert!(!block.contains("emotion") || block.contains("not emotion"));
    }

    #[test]
    fn dominant_pressures_sorted_by_value() {
        let mut state = OperationalPressureState::new();
        state.safety_pressure = 0.9;
        state.evidence_gap_pressure = 0.7;
        state.uncertainty_pressure = 0.5;
        let dominant = state.dominant_pressures(3);
        assert_eq!(dominant[0], PressureField::Safety);
        assert_eq!(dominant[1], PressureField::EvidenceGap);
    }

    #[test]
    fn max_pressure_returns_highest() {
        let mut state = OperationalPressureState::new();
        state.safety_pressure = 0.9;
        state.uncertainty_pressure = 0.3;
        assert!((state.max_pressure() - 0.9).abs() < 0.01);
    }
}
