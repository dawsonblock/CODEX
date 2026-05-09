"""Configuration for the runtime."""
from __future__ import annotations
from dataclasses import dataclass


@dataclass
class RuntimeConfig:
    base_candidate_budget: int = 4
    min_candidate_budget: int = 2
    max_candidate_budget: int = 12
    workspace_capacity: int = 5
    prescreen_fraction: float = 0.5
    fast_path_enabled: bool = True
    async_consolidation: bool = True
    trace_dir: str = "artifacts/traces"
    ablation_dir: str = "artifacts/ablation"
    self_report_persistence_cycles: int = 2
    max_memory_context: int = 5
    random_seed: int = 7
    semantic_cache_enabled: bool = True
    long_term_archive_enabled: bool = True
    long_term_archive_path: str = "artifacts/memory/runtime.gwlog"
    creative_deconstruction_enabled: bool = True
    abstraction_interval: int = 3
    abstraction_window: int = 50
    play_mode_threshold: float = 0.8
