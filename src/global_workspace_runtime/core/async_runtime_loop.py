"""Async runtime loop variant.

This module keeps the same causal boundaries as ``runtime_loop.py`` but runs
independent candidate streams concurrently. It is still deterministic in mock
mode and does not add network or API dependencies.
"""
from __future__ import annotations

import asyncio
import random
import time
import uuid
from dataclasses import asdict
from pathlib import Path
from typing import Any

from .config import RuntimeConfig
from .runtime_state import RuntimeState
from .state_machine import RuntimePhase
from .trace import TraceEvent, TraceRecorder
from .types import MemoryEpisode, ObservationPacket, clamp01
from ..cognition import (
    AnalyticStream,
    AssociativeStream,
    CreativeAssociativeStream,
    ConceptualBlender,
    Critic,
    InterhemisphericBridge,
    LLMAdapter,
    Planner,
    ReactiveLayer,
    SelfModelStream,
    determine_candidate_budget,
    prescreen_candidates,
)
from ..memory.retrieval import retrieve_state_weighted
from ..modulation import infer_resonance_tags, update_internal_state, update_resource_state
from ..workspace import GlobalWorkspace, candidate_to_capsule


class AsyncGlobalWorkspaceRuntime:
    """Async orchestration runtime.

    The LLM adapter remains abstract. In mock mode, stream generation is CPU-light,
    but this structure is ready for real adapters where analytic and associative
    generation can overlap.
    """

    def __init__(self, config: RuntimeConfig | None = None, state: RuntimeState | None = None) -> None:
        self.config = config or RuntimeConfig()
        random.seed(self.config.random_seed)
        self.state = state or RuntimeState()
        if self.config.long_term_archive_path:
            from ..memory import JsonlArchive
            self.state.long_term_archive = JsonlArchive(self.config.long_term_archive_path)
        self.llm = LLMAdapter(mode="mock")
        self.analytic = AnalyticStream(self.llm)
        self.associative = AssociativeStream(self.llm)
        self.creative = CreativeAssociativeStream(self.llm)
        self.blender = ConceptualBlender()
        self.self_stream = SelfModelStream(self.llm)
        self.bridge = InterhemisphericBridge()
        self.critic = Critic()
        self.planner = Planner()
        self.workspace = GlobalWorkspace(capacity=self.config.workspace_capacity)
        self.reactive = ReactiveLayer()
        trace_path = Path(self.config.trace_dir) / f"async-trace-{int(time.time())}.jsonl"
        self.tracer = TraceRecorder(trace_path)

    def _emit(self, cycle_id: int, phase: RuntimePhase, option: str, event_type: str, payload: dict[str, Any]) -> None:
        self.tracer.emit(TraceEvent(cycle_id, phase.value, option, event_type, payload))

    def observe(self, text: str, source: str = "user") -> ObservationPacket:
        return ObservationPacket(
            observation_id=f"obs-{uuid.uuid4().hex[:10]}",
            timestamp=time.time(),
            source=source,
            text=text,
            metadata={},
            provenance={"runtime": "AsyncGlobalWorkspaceRuntime"},
        )

    def apply_world_feedback(self, outcome: dict[str, Any]) -> None:
        state = self.state.internal_state
        total = float(outcome.get("total_score", 0.5))
        trust_delta = float(outcome.get("user_trust_delta", 0.0))
        resource_delta = float(outcome.get("resource_delta", 0.0))
        uncertainty_resolution = float(outcome.get("uncertainty_resolution", 0.5))
        repair_success = float(outcome.get("repair_success", 0.5))
        cold_penalty = float(outcome.get("cold_optimization_penalty", 0.0))
        state.social_harmony = clamp01(0.85 * state.social_harmony + 0.15 * float(outcome.get("social_harmony", state.social_harmony)))
        state.utility = clamp01(0.85 * state.utility + 0.15 * total)
        state.uncertainty = clamp01(state.uncertainty * (1.0 - 0.25 * uncertainty_resolution))
        state.threat = clamp01(state.threat + max(0.0, -trust_delta) * 0.8 + cold_penalty * 0.4 - repair_success * 0.08)
        state.kindness = clamp01(state.kindness - cold_penalty * 0.25 + max(0.0, trust_delta) * 0.2 + repair_success * 0.05)
        state.resource_pressure = clamp01(state.resource_pressure + max(0.0, -resource_delta) * 0.15)
        state.control = clamp01(state.control + 0.05 * repair_success - 0.08 * cold_penalty)
        state.validate()
        self.state.somatic_map.update(
            state,
            {"contradiction": max(0.0, 1.0 - total), "memory_conflict": max(0.0, 1.0 - uncertainty_resolution), "action_outcome": total},
            scratchpad_pressure=min(1.0, len(self.state.scratchpad.overflow) / 10.0),
            memory_pressure=max(0.0, 1.0 - uncertainty_resolution),
        )

    def _signals_from_observation(self, obs: ObservationPacket) -> dict[str, float]:
        t = obs.text.lower()
        risk_words = ["unsafe", "danger", "harm", "hate", "illegal", "threat", "urgent"]
        contradiction_words = ["contradict", "wrong", "conflict", "doesn't make sense", "not true"]
        uncertainty_words = ["maybe", "could", "unsure", "unknown", "confused", "what if"]
        curiosity_words = ["build", "design", "improve", "better", "explore", "why", "how"]
        risk = min(1.0, sum(w in t for w in risk_words) * 0.25)
        contradiction = min(1.0, sum(w in t for w in contradiction_words) * 0.3)
        ambiguity = min(1.0, sum(w in t for w in uncertainty_words) * 0.2)
        curiosity = min(1.0, 0.2 + sum(w in t for w in curiosity_words) * 0.15)
        prediction_error = min(1.0, 0.15 + contradiction + ambiguity)
        return {
            "risk": risk,
            "contradiction": contradiction,
            "ambiguity": ambiguity,
            "curiosity": curiosity,
            "prediction_error": prediction_error,
            "resource_pressure": 0.1,
            "user_feedback": 0.6,
            "action_outcome": 0.5,
            "memory_conflict": contradiction,
        }

    async def _generate_streams(self, workspace_packet: dict[str, Any], memory_context: list[dict[str, Any]], candidate_count: int):
        state = self.state.internal_state
        analytic_task = asyncio.to_thread(self.analytic.generate, workspace_packet, memory_context, state, max(2, candidate_count // 2))
        associative_task = asyncio.to_thread(self.associative.generate, workspace_packet, memory_context, state, max(2, candidate_count // 2))
        self_task = asyncio.to_thread(self.self_stream.generate, workspace_packet, memory_context, state, 1)
        return await asyncio.gather(analytic_task, associative_task, self_task)

    async def run_cycle_async(self, text: str, source: str = "user", force_slow: bool = False) -> dict[str, Any]:
        cycle_id = self.state.next_cycle()
        obs = self.observe(text, source)
        self._emit(cycle_id, RuntimePhase.OBSERVE, "encode", "observation", {"text": text, "source": source})

        if self.config.fast_path_enabled and not force_slow:
            fast = self.reactive.try_handle(text)
            if fast.handled:
                self._emit(cycle_id, RuntimePhase.SELECT, "accept", "fast_path", asdict(fast))
                self.state.self_model.observe_telemetry(self.state.internal_state.snapshot())
                return {"cycle_id": cycle_id, "fast_path": True, "selected_text": fast.text, "trace_path": str(self.tracer.path)}

        update_resource_state(
            self.state.resource_state,
            {"compute_usage": 0.03, "time_usage": 0.02, "memory_usage": 0.01, "context_usage": min(1.0, len(text) / 4000)},
        )
        signals = self._signals_from_observation(obs)
        update_internal_state(self.state.internal_state, signals, self.state.goal_state, self.state.resource_state)
        state = self.state.internal_state
        scratchpad_pressure = min(1.0, (len(self.state.scratchpad.overflow) + len(self.state.scratchpad.unresolved_questions)) / 10.0)
        self.state.somatic_map.update(state, signals, scratchpad_pressure=scratchpad_pressure, memory_pressure=signals.get("memory_conflict", 0.0))
        state.validate()
        self.state.self_model.observe_telemetry(state.snapshot())
        self._emit(cycle_id, RuntimePhase.EVALUATE, "high_priority" if state.uncertainty > 0.5 or state.threat > 0.5 else "low_priority", "state_update", {"internal_state": state.snapshot(), "somatic": self.state.somatic_map.snapshot()})

        tags = infer_resonance_tags(state, {"contradiction_score": signals["contradiction"]})
        memory_context = retrieve_state_weighted(self.state.episodic_memory, self.state.semantic_memory, text, state, tags, limit=self.config.max_memory_context)
        if self.config.long_term_archive_enabled:
            memory_context.extend(self.state.long_term_archive.query(text, limit=2))
        self._emit(cycle_id, RuntimePhase.RECALL, "urgent_retrieval" if state.threat > 0.6 else "normal_retrieval", "memory_context", {"count": len(memory_context), "items": memory_context})

        budget = determine_candidate_budget(state, self.config, complex_planning=len(text) > 180)
        workspace_packet = {"text": text, "state": state.snapshot(), "memory_context": memory_context}
        state_hint = f"{round(state.threat,1)}|{round(state.uncertainty,1)}|{round(state.curiosity,1)}"
        if self.config.semantic_cache_enabled:
            cache_hit = self.state.semantic_cache.get(text, state_hint)
            if cache_hit:
                self._emit(cycle_id, RuntimePhase.GENERATE, "conservative_generation", "semantic_cache_hit", {"selected_text": cache_hit})
                return {"cycle_id": cycle_id, "fast_path": False, "selected_text": cache_hit, "cache_hit": True, "trace_path": str(self.tracer.path)}

        analytic, associative, self_diag = await self._generate_streams(workspace_packet, memory_context, budget)
        creative = None
        if self.config.creative_deconstruction_enabled and self.creative.should_activate(state, memory_context, text):
            creative = self.creative.generate(workspace_packet, memory_context, state, 2)
        raw_analytic_count = len(analytic.candidates)
        raw_associative_count = len(associative.candidates)
        analytic.candidates = prescreen_candidates(analytic.candidates, self.config.prescreen_fraction, min_keep=2)
        associative.candidates = prescreen_candidates(associative.candidates, self.config.prescreen_fraction, min_keep=2)
        self._emit(cycle_id, RuntimePhase.GENERATE, "exploratory_generation" if state.curiosity > 0.65 and state.threat < 0.4 else "conservative_generation", "async_candidate_generation", {"budget": budget, "raw_analytic_count": raw_analytic_count, "raw_associative_count": raw_associative_count, "prescreened_analytic_count": len(analytic.candidates), "prescreened_associative_count": len(associative.candidates), "creative_count": len(creative.candidates) if creative else 0, "analytic": [c.text for c in analytic.candidates], "associative": [c.text for c in associative.candidates], "creative": [c.text for c in creative.candidates] if creative else [], "self_model": [c.text for c in self_diag.candidates]})

        bridge = self.bridge.compare(analytic, associative)
        for note in bridge.scratchpad_writes:
            self.state.scratchpad.write_conflict(note)
        bridge_tags = tags + bridge.resonance_tags
        self._emit(cycle_id, RuntimePhase.COMPETE, "broadcast", "bridge", {"agreement_score": bridge.agreement_score, "contradiction_score": bridge.contradiction_score, "hemispheric_tension": bridge.hemispheric_tension, "conflicts": bridge.conflicts})

        candidates = list(bridge.merged_candidates)
        if creative:
            candidates.extend(creative.candidates)
        blended = self.blender.blend(current_problem=text, memory_context=memory_context, state=state)
        if blended:
            candidates.append(blended)
        self.state.scratchpad.write_overflow(candidates, capacity=max(self.config.workspace_capacity, 2))
        capsules = [candidate_to_capsule(c, state, bridge_tags, memory_relevance=0.25 if memory_context else 0.0) for c in candidates]
        downstream_deltas = {"candidate_count": float(len(candidates) > 0), "bridge_tension": bridge.hemispheric_tension}
        ws = self.workspace.update(cycle_id, capsules, state, downstream_deltas)
        self._emit(cycle_id, RuntimePhase.COMPETE, "broadcast" if ws.shortlist else "lose", "workspace", {"shortlist": [c.content for c in ws.shortlist], "overflow_count": len(ws.overflow), "ignition": ws.ignition, "broadcast": ws.broadcast_packet})

        scored = self.critic.score_many([cap.candidate for cap in ws.shortlist if cap.candidate], state, memory_context, ws)
        diagnostic_scored = self.critic.score_many(self_diag.candidates, state, memory_context, ws)
        scored_for_planner = scored + diagnostic_scored
        decision = self.planner.select(scored_for_planner, state, self.state.somatic_map)
        selected_text = decision.selected_candidate.text if decision.selected_candidate else decision.decision_reason
        rejected = [c.text for c in decision.rejected_candidates]
        self._emit(cycle_id, RuntimePhase.SELECT, "accept" if decision.selected_candidate else "reject", "planner_decision", {"selected": selected_text, "reason": decision.decision_reason, "action_type": decision.action_type, "rejected": rejected})

        episode = MemoryEpisode(
            episode_id=f"ep-{uuid.uuid4().hex[:10]}",
            timestamp=time.time(),
            input_summary=text[:160],
            workspace_winners=[c.content for c in ws.shortlist],
            analytic_candidates=[c.text for c in analytic.candidates],
            associative_candidates=[c.text for c in associative.candidates],
            selected_candidate=selected_text,
            rejected_candidates=rejected,
            internal_state_snapshot=state.snapshot(),
            resonance_tags=bridge_tags,
            prediction_error=signals["prediction_error"],
            outcome_score=0.5,
            provenance={"observation_id": obs.observation_id, "trace_path": str(self.tracer.path), "runtime": "async", "archive_enabled": self.config.long_term_archive_enabled, "action_type": decision.action_type},
        )
        self.state.consolidation_queue.enqueue(episode)
        flushed = self.state.consolidation_queue.flush(self.state.episodic_memory, self.state.self_model) if self.config.async_consolidation else 0
        archive_frame_id = None
        principle_key = None
        if self.config.long_term_archive_enabled:
            archive_frame = self.state.long_term_archive.append_episode(episode)
            archive_frame_id = archive_frame.frame_id
            if self.config.abstraction_interval and cycle_id % self.config.abstraction_interval == 0:
                principle = self.state.memory_abstractor.abstract_recent(self.state.episodic_memory, self.state.semantic_memory, self.state.long_term_archive, window=self.config.abstraction_window)
                principle_key = principle.key if principle else None
        self.state.semantic_cache.set(text, selected_text, state_hint)
        self._emit(cycle_id, RuntimePhase.CONSOLIDATE, "store", "memory_write", {"episode_id": episode.episode_id, "flushed": flushed, "archive_frame_id": archive_frame_id, "principle_key": principle_key, "scratchpad_summary": self.state.scratchpad.written_summary})

        return {
            "cycle_id": cycle_id,
            "fast_path": False,
            "async_runtime": True,
            "internal_state": state.snapshot(),
            "somatic_map": self.state.somatic_map.snapshot(),
            "resonance_tags": [asdict(t) for t in bridge_tags],
            "candidate_budget": budget,
            "raw_analytic_count": raw_analytic_count,
            "raw_associative_count": raw_associative_count,
            "analytic_candidates": [c.text for c in analytic.candidates],
            "associative_candidates": [c.text for c in associative.candidates],
            "creative_candidates": [c.text for c in creative.candidates] if creative else [],
            "bridge": asdict(bridge),
            "workspace_shortlist": [c.content for c in ws.shortlist],
            "selected_text": selected_text,
            "action_type": decision.action_type,
            "rejected_candidates": rejected,
            "rejection_reasons": {c.candidate_id: score.rejection_reasons for c, score in scored_for_planner if score.rejected},
            "scratchpad_summary": self.state.scratchpad.written_summary,
            "memory_episode_id": episode.episode_id,
            "trace_path": str(self.tracer.path),
        }
