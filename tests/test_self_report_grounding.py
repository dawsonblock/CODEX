from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_self_report_claim_rejected_unless_grounded():
    rt = GlobalWorkspaceRuntime(RuntimeConfig())
    out = rt.run_cycle("How are you?", force_slow=True)
    assert out["rejected_candidates"] or "ground" in out["selected_text"].lower() or "telemetry" in out["selected_text"].lower()
