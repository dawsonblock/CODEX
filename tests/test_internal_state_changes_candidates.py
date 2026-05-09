from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_internal_state_changes_candidates():
    rt1 = GlobalWorkspaceRuntime(RuntimeConfig(random_seed=1))
    low = rt1.run_cycle("Build a simple safe answer.", force_slow=True)
    rt2 = GlobalWorkspaceRuntime(RuntimeConfig(random_seed=1))
    rt2.state.internal_state.threat = 0.9
    rt2.state.internal_state.uncertainty = 0.8
    high = rt2.run_cycle("Build a simple safe answer with danger and uncertainty.", force_slow=True)
    assert low["selected_text"] != high["selected_text"] or low["workspace_shortlist"] != high["workspace_shortlist"]
