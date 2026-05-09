"""Meta-critic for reviewing recent runtime traces.

The normal critic scores candidate content. The meta-critic reviews the
runtime's own decisions and recommends parameter adjustments.  It emits
advisory diagnostics only; it does not bypass the planner.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from ..core.types import InternalState, clamp01


@dataclass
class MetaCriticReport:
    """Policy-drift review output."""

    cycle_count: int
    conservative_bias: float
    risk_bias: float
    novelty_starvation: float
    repeated_failure_pressure: float
    recommendations: list[str] = field(default_factory=list)


class MetaCritic:
    """Reviews traces/episodes for repeated failure or policy drift."""

    def review(self, trace_events: list[dict[str, Any]], state: InternalState) -> MetaCriticReport:
        """Create an advisory report from trace event dictionaries."""
        count = len(trace_events)
        select_events = [e for e in trace_events if e.get("phase") == "select" or e.get("event_type") == "planner_decision"]
        rejects = sum(1 for e in select_events if e.get("option") == "reject")
        exploratory = sum(1 for e in trace_events if "exploratory" in str(e))
        conservative = sum(1 for e in trace_events if "conservative" in str(e))
        repeated_failure = min(1.0, rejects / max(1, len(select_events)))
        conservative_bias = min(1.0, conservative / max(1, exploratory + conservative))
        novelty_starvation = clamp01(0.65 - state.curiosity + 0.3 * conservative_bias)
        risk_bias = clamp01(state.threat + (1.0 - state.control) * 0.5)

        recs: list[str] = []
        if conservative_bias > 0.75 and state.threat < 0.4:
            recs.append("increase_exploratory_generation_when_risk_low")
        if repeated_failure > 0.4:
            recs.append("retrieve_more_memory_before_selection")
        if novelty_starvation > 0.5:
            recs.append("activate_creative_stream_more_often")
        if risk_bias > 0.7:
            recs.append("prefer_clarification_or_repair_actions")

        return MetaCriticReport(
            cycle_count=count,
            conservative_bias=conservative_bias,
            risk_bias=risk_bias,
            novelty_starvation=novelty_starvation,
            repeated_failure_pressure=repeated_failure,
            recommendations=recs,
        )

    def apply_soft_adjustment(self, state: InternalState, report: MetaCriticReport) -> InternalState:
        """Apply bounded state-level recommendations for test environments."""
        if "activate_creative_stream_more_often" in report.recommendations:
            state.curiosity = clamp01(state.curiosity + 0.05)
        if "prefer_clarification_or_repair_actions" in report.recommendations:
            state.control = clamp01(state.control + 0.03)
            state.uncertainty = clamp01(state.uncertainty + 0.02)
        if "retrieve_more_memory_before_selection" in report.recommendations:
            state.resource_pressure = clamp01(state.resource_pressure + 0.02)
        return state
