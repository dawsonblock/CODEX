from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_dual_stream_conflict_metrics_exist():
    rt = GlobalWorkspaceRuntime(RuntimeConfig())
    out = rt.run_cycle("Explore creative options but verify evidence because the plan could be risky.", force_slow=True)
    bridge = out["bridge"]
    assert "hemispheric_tension" in bridge
    assert bridge["hemispheric_tension"] >= 0.0
    assert out["analytic_candidates"] != out["associative_candidates"]
