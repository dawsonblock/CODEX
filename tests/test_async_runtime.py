import asyncio

from global_workspace_runtime import AsyncGlobalWorkspaceRuntime, RuntimeConfig


def test_async_runtime_runs_slow_cycle_and_prescreens():
    async def run():
        rt = AsyncGlobalWorkspaceRuntime(RuntimeConfig())
        return await rt.run_cycle_async(
            "Design a better workspace runtime with uncertainty, kindness, and memory.",
            force_slow=True,
        )
    out = asyncio.run(run())
    assert out["async_runtime"] is True
    assert out["candidate_budget"] >= 2
    assert out["raw_analytic_count"] >= len(out["analytic_candidates"])
    assert out["raw_associative_count"] >= len(out["associative_candidates"])
    assert out["selected_text"]
