from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_scratchpad_overflow_written():
    cfg = RuntimeConfig(workspace_capacity=2)
    rt = GlobalWorkspaceRuntime(cfg)
    out = rt.run_cycle("Complex planning with many options and uncertainty about evidence and risk.", force_slow=True)
    assert "kept" in out["scratchpad_summary"].lower() or rt.state.scratchpad.overflow
