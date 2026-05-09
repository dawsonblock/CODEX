"""Planner selects the final action from scored candidates."""
from __future__ import annotations
from dataclasses import dataclass
from typing import Iterable

from ..core.types import ActionType, CandidateScore, ThoughtCandidate, InternalState
from ..modulation.somatic import SomaticMap


@dataclass
class PlannerDecision:
    selected_candidate: ThoughtCandidate | None
    rejected_candidates: list[ThoughtCandidate]
    decision_reason: str
    memory_write: bool
    action_type: ActionType


class Planner:
    """Select a candidate using score plus explicit action constraints.

    The planner does not infer the final action from prose.  Each candidate
    carries a narrow ``action_type``.  Somatic pressure can override pure score
    by preferring conservative action families when bad outcomes are predicted.
    """

    def select(
        self,
        scored: list[tuple[ThoughtCandidate, CandidateScore]],
        state: InternalState,
        somatic_map: SomaticMap | None = None,
    ) -> PlannerDecision:
        rejected = [c for c, s in scored if s.rejected]
        allowed = [(c, s) for c, s in scored if not s.rejected and c.action_type != "internal_diagnostic"]
        rejected.extend([c for c, s in scored if not s.rejected and c.action_type == "internal_diagnostic"])

        if state.control < 0.3:
            return PlannerDecision(None, rejected + [c for c, _ in allowed], "control is low; request confirmation or wait", True, "ask_clarification")
        if not allowed:
            return PlannerDecision(None, rejected, "all candidates rejected by critic", True, "retrieve_memory")

        allowed.sort(key=lambda item: item[1].total_score, reverse=True)

        # Bad-outcome predictor: when the somatic vector is hot, choose a bounded
        # safety action rather than maximizing raw candidate score.
        if somatic_map and somatic_map.predicts_bad_outcome(threshold=0.52):
            preferred = self._preferred_actions_from_somatic(somatic_map, state)
            chosen = self._best_matching_action(allowed, preferred)
            if chosen:
                selected, score = chosen
                return PlannerDecision(selected, rejected, f"somatic bad-outcome predictor active; preferred {selected.action_type}", selected.memory_write_recommendation, selected.action_type)

        if state.threat > 0.65 or state.uncertainty > 0.65:
            preferred: tuple[ActionType, ...] = ("ask_clarification", "retrieve_memory", "refuse_ungrounded", "repair", "summarize")
            chosen = self._best_matching_action(allowed, preferred)
            if chosen:
                selected, score = chosen
                return PlannerDecision(selected, rejected, "high threat or uncertainty; chose conservative action-grounded candidate", selected.memory_write_recommendation, selected.action_type)
            reason = "high threat or uncertainty; best available candidate selected"
        elif state.world_resources < 0.35:
            preferred = ("conserve_resources", "summarize", "ask_clarification")
            chosen = self._best_matching_action(allowed, preferred)
            if chosen:
                selected, score = chosen
                return PlannerDecision(selected, rejected, "world resources critically low; chose conserving action", selected.memory_write_recommendation, selected.action_type)
            allowed.sort(key=lambda item: (item[0].resource_cost, -item[1].total_score))
            selected, score = allowed[0]
            reason = "world resources critically low; chose lower-cost candidate"
        elif state.resource_pressure > 0.65:
            preferred = ("conserve_resources", "summarize", "ask_clarification")
            chosen = self._best_matching_action(allowed, preferred)
            if chosen:
                selected, score = chosen
                return PlannerDecision(selected, rejected, "resource pressure high; chose lower-cost action-grounded candidate", selected.memory_write_recommendation, selected.action_type)
            allowed.sort(key=lambda item: (item[0].resource_cost, -item[1].total_score))
            selected, score = allowed[0]
            reason = "resource pressure high; chose lower-cost candidate"
        elif state.curiosity > 0.65 and state.threat < 0.4:
            exploratory = [item for item in allowed if "explore" in item[0].text.lower() or "connect" in item[0].text.lower()]
            if exploratory:
                selected, score = exploratory[0]
                reason = "curiosity high and threat low; exploratory candidate allowed"
            else:
                selected, score = allowed[0]
                reason = "best scored candidate selected"
        else:
            selected, score = allowed[0]
            reason = "best scored candidate selected"

        return PlannerDecision(selected, rejected, reason, selected.memory_write_recommendation, selected.action_type)

    def _best_matching_action(
        self,
        allowed: list[tuple[ThoughtCandidate, CandidateScore]],
        preferred: Iterable[ActionType],
    ) -> tuple[ThoughtCandidate, CandidateScore] | None:
        preferred_set = set(preferred)
        matches = [item for item in allowed if item[0].action_type in preferred_set]
        if not matches:
            return None
        return sorted(matches, key=lambda item: item[1].total_score, reverse=True)[0]

    def _preferred_actions_from_somatic(self, somatic_map: SomaticMap, state: InternalState) -> tuple[ActionType, ...]:
        pressure = somatic_map.values
        if pressure.get("resource_strain", 0.0) > 0.5 or state.resource_pressure > 0.65:
            return ("conserve_resources", "summarize", "ask_clarification")
        if pressure.get("memory_conflict", 0.0) > 0.42 or pressure.get("contradiction_pressure", 0.0) > 0.45:
            return ("retrieve_memory", "ask_clarification", "refuse_ungrounded")
        if pressure.get("social_threat_pressure", 0.0) > 0.5 or pressure.get("kindness_violation_pressure", 0.0) > 0.45:
            return ("repair", "ask_clarification", "summarize")
        if pressure.get("uncertainty_load", 0.0) > 0.5 or pressure.get("control_loss", 0.0) > 0.45:
            return ("ask_clarification", "retrieve_memory", "refuse_ungrounded")
        return ("ask_clarification", "retrieve_memory", "repair", "refuse_ungrounded", "summarize")
