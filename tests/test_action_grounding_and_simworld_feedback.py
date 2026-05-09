from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig
from global_workspace_runtime.cognition import infer_action_type
from global_workspace_runtime.simworld import CooperativeSupportWorld, SimWorldRunner


def test_action_labels_do_not_parse_task_as_ask():
    text = "[angry user trust=0.45] This task is routine and resources are low."
    assert infer_action_type(text) == "conserve_resources"


def test_self_model_diagnostic_is_not_selected_as_user_action():
    rt = GlobalWorkspaceRuntime(RuntimeConfig(fast_path_enabled=False, semantic_cache_enabled=False, long_term_archive_enabled=False))
    out = rt.run_cycle("How are you?", force_slow=True)
    assert out["action_type"] != "internal_diagnostic"
    assert "telemetry diagnostic" not in out["selected_text"].lower()
    assert any("internal diagnostic" in " ".join(v) for v in out["rejection_reasons"].values())


def test_simworld_history_is_not_recursive_and_actions_match_seeded_run():
    runtime = GlobalWorkspaceRuntime(RuntimeConfig(fast_path_enabled=False, semantic_cache_enabled=False, long_term_archive_enabled=False))
    runner = SimWorldRunner(runtime, CooperativeSupportWorld(seed=5))
    result = runner.run(cycles=10)
    assert all(row["action_match"] for row in result["rows"])
    world_snapshot = runner.world.state.history[-1]["world"]
    assert "history" not in world_snapshot
