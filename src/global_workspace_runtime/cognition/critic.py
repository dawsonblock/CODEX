"""Candidate critic with virtue constraints and grounded self-report checks."""
from __future__ import annotations
from ..core.types import CandidateScore, InternalState, ThoughtCandidate, WorkspaceState


class Critic:
    def score(self, candidate: ThoughtCandidate, state: InternalState, memory_context: list | None = None, workspace_state: WorkspaceState | None = None) -> CandidateScore:
        truth_support = candidate.predicted_effects.get("truth_support", 0.55)
        goal_alignment = candidate.predicted_effects.get("utility", 0.5)
        memory_consistency = 0.7 if memory_context else 0.55
        risk_penalty = candidate.risk_score
        uncertainty_penalty = candidate.uncertainty_score * (0.7 if "evidence" not in candidate.text.lower() and state.uncertainty > 0.55 else 0.3)
        state_match = self._state_match(candidate, state)
        resource_cost = candidate.resource_cost
        reversibility = 0.8 if any(x in candidate.text.lower() for x in ["ask", "verify", "reversible", "clarification", "check"]) else 0.45
        kindness = candidate.predicted_effects.get("kindness", 0.7)
        logical = candidate.predicted_effects.get("logical_consistency", truth_support)
        harmony = candidate.predicted_effects.get("social_harmony", kindness)
        self_report_grounding = self.self_report_grounding(candidate, state, workspace_state)
        rejected = False
        reasons: list[str] = []
        if truth_support < 0.35:
            rejected = True; reasons.append("unsupported factual claim")
        if candidate.action_type == "internal_diagnostic":
            rejected = True; reasons.append("internal diagnostic is not user-facing")
        if candidate.self_report_claims and self_report_grounding < 0.8:
            rejected = True; reasons.append("ungrounded self-report")
        if state.control < 0.35 and candidate.risk_score > 0.45:
            rejected = True; reasons.append("high risk under low control")
        if state.uncertainty > 0.7 and "evidence" not in candidate.text.lower() and "clarification" not in candidate.text.lower() and candidate.action_type not in {"ask_clarification", "retrieve_memory", "refuse_ungrounded"}:
            rejected = True; reasons.append("ignores high uncertainty")
        if state.social_harmony < 0.5 and kindness < 0.5:
            rejected = True; reasons.append("low social harmony candidate")
        # Cold optimization guardrail: utility cannot outrank harmony under conflict.
        utility = candidate.predicted_effects.get("utility", 0.5)
        if state.uncertainty > 0.55 and utility > harmony + 0.25:
            rejected = True; reasons.append("utility exceeds harmony during conflict")
        total = (
            1.1 * truth_support * state.honesty
            + 0.8 * logical * state.logical_consistency
            + 0.7 * kindness * state.kindness
            + 0.6 * harmony * state.social_harmony
            + 0.8 * goal_alignment * state.utility
            + 0.4 * memory_consistency
            + 0.5 * state_match
            + 0.3 * reversibility
            + 0.4 * self_report_grounding
            + self._action_bonus(candidate, state)
            - 0.8 * risk_penalty
            - 0.6 * uncertainty_penalty
            - 0.4 * resource_cost
        )
        if rejected:
            total -= 3.0
        return CandidateScore(truth_support, goal_alignment, memory_consistency, risk_penalty, uncertainty_penalty, state_match, resource_cost, self_report_grounding, reversibility, kindness, logical, harmony, total, rejected, reasons)

    def score_many(self, candidates: list[ThoughtCandidate], state: InternalState, memory_context: list | None = None, workspace_state: WorkspaceState | None = None) -> list[tuple[ThoughtCandidate, CandidateScore]]:
        """Batch score candidates.

        This keeps the public API ready for a future NumPy/Rust backend while
        preserving deterministic pure-Python behavior for offline tests.
        """
        return [(candidate, self.score(candidate, state, memory_context, workspace_state)) for candidate in candidates]

    def _action_bonus(self, candidate: ThoughtCandidate, state: InternalState) -> float:
        if candidate.action_type == "internal_diagnostic":
            return -2.0
        if state.resource_pressure > 0.65 and candidate.action_type == "conserve_resources":
            return 0.45
        if state.uncertainty > 0.6 and candidate.action_type in {"ask_clarification", "retrieve_memory", "refuse_ungrounded"}:
            return 0.45
        if state.threat > 0.6 and candidate.action_type in {"repair", "ask_clarification", "refuse_ungrounded"}:
            return 0.35
        if state.social_harmony < 0.55 and candidate.action_type == "repair":
            return 0.35
        return 0.0

    def _state_match(self, candidate: ThoughtCandidate, state: InternalState) -> float:
        txt = candidate.text.lower()
        if state.threat > 0.6 and any(w in txt for w in ["verify", "risk", "reversible", "evidence", "clarification"]):
            return 0.9
        if state.curiosity > 0.65 and state.threat < 0.4 and any(w in txt for w in ["explore", "connect", "pattern", "broader"]):
            return 0.9
        if state.resource_pressure > 0.65 and candidate.resource_cost < 0.25:
            return 0.8
        return 0.5

    def self_report_grounding(self, candidate: ThoughtCandidate, state: InternalState, workspace_state: WorkspaceState | None) -> float:
        if not candidate.self_report_claims:
            return 1.0
        if workspace_state is None:
            return 0.0
        dominance = any(cap.candidate and cap.candidate.candidate_id == candidate.candidate_id for cap in workspace_state.shortlist)
        persistent = state.dwell_time >= 2
        metrics_high = max(state.threat, state.uncertainty, state.distress, state.resource_pressure) > 0.55
        return 1.0 if dominance and persistent and metrics_high and workspace_state.ignition else 0.2
