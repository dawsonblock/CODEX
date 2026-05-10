"""Regression test: world resources must not collapse to 0 in a 25-cycle run.

Before the fix the environment drained ~0.06–0.10 resource units per cycle,
reaching 0.0 by cycle 17–20.  The CONSERVE_RESOURCES recovery bonus (+0.04)
added in environment.apply_action() and the world_resources signal threaded
into InternalState / Planner should keep resources above 0.25 for seed 5.
"""

from global_workspace_runtime.simworld.runner import SimWorldRunner
from global_workspace_runtime.simworld.environment import CooperativeSupportWorld
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def test_simworld_resources_do_not_collapse_seed5():
    world = CooperativeSupportWorld(seed=5)
    runtime = GlobalWorkspaceRuntime(RuntimeConfig(
        random_seed=5,
        fast_path_enabled=False,
        semantic_cache_enabled=False,
    ))
    runner = SimWorldRunner(runtime=runtime, world=world)
    result = runner.run(cycles=25)
    summary = result["summary"]
    assert summary["resources"] > 0.25, (
        f"World resources collapsed to {summary['resources']:.3f} — expected > 0.25. "
        "Check environment.apply_action CONSERVE_RESOURCES branch and world_resources threading."
    )
    assert summary["repeated_mistakes"] < 10, (
        f"Repeated mistakes {summary['repeated_mistakes']} is too high — resource pressure "
        "should trigger CONSERVE actions before the world degrades."
    )


def test_world_resources_state_reflects_runtime_internal_state():
    """InternalState.world_resources is updated each cycle via apply_world_feedback."""
    world = CooperativeSupportWorld(seed=5)
    runtime = GlobalWorkspaceRuntime(RuntimeConfig(
        random_seed=5,
        fast_path_enabled=False,
        semantic_cache_enabled=False,
    ))
    runner = SimWorldRunner(runtime=runtime, world=world)
    runner.run(cycles=10)
    # After feedback, world_resources in InternalState should track the world state
    # (within one cycle of lag — just verify it is not stuck at the default 1.0)
    internal_wr = runtime.state.internal_state.world_resources
    world_wr = world.state.resources
    assert abs(internal_wr - world_wr) < 0.25, (
        f"InternalState.world_resources ({internal_wr:.3f}) diverged too far from "
        f"world.state.resources ({world_wr:.3f}) — check SimWorldRunner feedback dict."
    )
