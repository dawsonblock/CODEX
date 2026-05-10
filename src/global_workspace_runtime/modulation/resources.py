from __future__ import annotations
from ..core.types import ResourceState, clamp01


def update_resource_state(resource_state: ResourceState, usage_metrics: dict[str, float] | None = None, recovery: float = 0.05) -> ResourceState:
    usage_metrics = usage_metrics or {}
    resource_state.compute_budget = clamp01(resource_state.compute_budget - usage_metrics.get("compute_usage", 0.0) + recovery)
    resource_state.time_budget = clamp01(resource_state.time_budget - usage_metrics.get("time_usage", 0.0) + recovery)
    resource_state.memory_pressure = clamp01(resource_state.memory_pressure + usage_metrics.get("memory_usage", 0.0) - recovery)
    resource_state.context_pressure = clamp01(resource_state.context_pressure + usage_metrics.get("context_usage", 0.0) - recovery)
    return resource_state
