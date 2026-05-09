from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_memory_retrieval_changes_with_state():
    rt = GlobalWorkspaceRuntime(RuntimeConfig())
    first = rt.run_cycle("Store a risky conflict memory with contradiction and danger.", force_slow=True)
    rt.state.internal_state.threat = 0.9
    high = rt.run_cycle("Retrieve relevant memory for a risky conflict.", force_slow=True)
    rt.state.internal_state.threat = 0.0
    low = rt.run_cycle("Retrieve relevant memory for a calm routine task.", force_slow=True)
    assert high["memory_episode_id"] != low["memory_episode_id"]
    assert len(rt.state.episodic_memory) >= 3
