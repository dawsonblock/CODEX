from pathlib import Path
from global_workspace_runtime.cognition import LLMAdapter, CreativeAssociativeStream, ConceptualBlender
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig
from global_workspace_runtime.core.types import InternalState, MemoryEpisode
from global_workspace_runtime.memory import JsonlArchive, MemoryAbstractor, EpisodicMemory, SemanticMemory


def test_jsonl_archive_appends_queries_and_rewinds(tmp_path):
    archive = JsonlArchive(tmp_path / "test.gwlog")
    f1 = archive.append_frame("Kind evidence-first repair pattern", frame_type="virtue_milestone", tags=["Weld"])
    f2 = archive.append_frame("Creative clarification pattern", frame_type="principle", tags=["Fold"])
    hits = archive.query("clarification repair", limit=5)
    assert hits
    assert any(h["frame_id"] == f2.frame_id for h in hits)
    rewind = archive.rewind(frame_id=f1.frame_id)
    assert [f.frame_id for f in rewind] == [f1.frame_id]


def test_creative_stream_generates_memory_blending_candidates():
    state = InternalState(curiosity=0.9, threat=0.1, uncertainty=0.4)
    memory = [{"type": "archive_frame", "frame_id": "frame-1", "text": "Prefer clarification under contradiction."}]
    stream = CreativeAssociativeStream(LLMAdapter())
    assert stream.should_activate(state, memory, "blend this with a new design problem")
    out = stream.generate({"text": "blend this with a new design problem"}, memory, state, 2)
    assert out.stream_name == "creative"
    assert len(out.candidates) == 2
    assert all(c.mode == "deconstruction" for c in out.candidates)


def test_conceptual_blender_adds_bounded_candidate():
    blender = ConceptualBlender()
    state = InternalState(curiosity=0.8, uncertainty=0.3, social_harmony=0.8)
    candidate = blender.blend(
        current_problem="Need a safer runtime route",
        memory_context=[{"type": "semantic", "value": "Choose reversible next steps under uncertainty."}],
        state=state,
    )
    assert candidate is not None
    assert candidate.stream_source == "merged"
    assert candidate.mode == "conceptual_blend"
    assert candidate.predicted_effects["social_harmony"] >= 0.8


def test_memory_abstractor_writes_principle_and_archive(tmp_path):
    episodic = EpisodicMemory()
    semantic = SemanticMemory()
    archive = JsonlArchive(tmp_path / "abstract.gwlog")
    ep = MemoryEpisode(
        episode_id="ep-1",
        timestamp=1.0,
        input_summary="conflicting evidence",
        workspace_winners=["verify evidence"],
        analytic_candidates=["verify evidence"],
        associative_candidates=["connect patterns"],
        selected_candidate="verify evidence before action",
        rejected_candidates=[],
        internal_state_snapshot={},
        resonance_tags=[],
        prediction_error=0.6,
        outcome_score=0.5,
    )
    episodic.append(ep)
    principle = MemoryAbstractor().abstract_recent(episodic, semantic, archive, window=5)
    assert principle is not None
    assert semantic.get(principle.key)
    assert any(frame.frame_type == "principle" for frame in archive.frames())


def test_runtime_writes_long_term_archive_and_creative_candidates(tmp_path):
    cfg = RuntimeConfig(
        trace_dir=str(tmp_path / "traces"),
        long_term_archive_path=str(tmp_path / "runtime.gwlog"),
        abstraction_interval=1,
        fast_path_enabled=False,
        semantic_cache_enabled=False,
    )
    rt = GlobalWorkspaceRuntime(cfg)
    # Seed archive with a prior principle so the creative stream can activate.
    rt.state.long_term_archive.append_frame(
        "When a problem is ambiguous, preserve kindness and ask for evidence.",
        frame_type="principle",
        tags=["Fold"],
    )
    out = rt.run_cycle("Build a creative safer runtime that improves memory and kindness.", force_slow=True)
    assert Path(cfg.long_term_archive_path).exists()
    assert cfg.long_term_archive_path.endswith(".gwlog")
    assert out["creative_candidates"]
    assert rt.state.semantic_memory.query("uncertainty contradiction")
