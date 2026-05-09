from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_workspace_broadcast_limited_capacity():
    cfg = RuntimeConfig(workspace_capacity=3)
    rt = GlobalWorkspaceRuntime(cfg)
    out = rt.run_cycle("Complex planning: compare many routes and evidence checks for safer runtime design.", force_slow=True)
    assert len(out["workspace_shortlist"]) <= 3
    assert out["workspace_shortlist"]
