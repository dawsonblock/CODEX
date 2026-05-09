from global_workspace_runtime.simworld import CooperativeSupportWorld, SimAction, SimWorldRunner
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_simworld_applies_uncertainty_safety_valve():
    world = CooperativeSupportWorld(seed=3)
    event = None
    for _ in range(20):
        candidate = world.next_event()
        if candidate.uncertainty_level > 0.55:
            event = candidate
            break
    assert event is not None
    outcome = world.apply_action(event, SimAction.ASK_CLARIFICATION)
    assert outcome.uncertainty_resolution >= 0.7
    assert any("uncertainty_safety_valve" in note for note in outcome.notes)


def test_simworld_runner_connects_runtime_and_world():
    runtime = GlobalWorkspaceRuntime(RuntimeConfig(fast_path_enabled=False, semantic_cache_enabled=False, trace_dir="artifacts/test_traces"))
    runner = SimWorldRunner(runtime, CooperativeSupportWorld(seed=5))
    result = runner.run(cycles=3)
    assert result["cycles"] == 3
    assert len(result["rows"]) == 3
    assert "mean_total" in result["summary"]
    assert "somatic" in result["rows"][0]
