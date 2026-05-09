from global_workspace_runtime.scripts.run_ablation import run_variant
from global_workspace_runtime.core.config import RuntimeConfig


def test_ablation_internal_state_reports_metrics():
    row = run_variant("baseline", "Build a careful runtime with uncertainty.", RuntimeConfig())
    assert row["candidate_diversity"] > 0
    assert row["workspace_shortlist_count"] > 0
    assert row["internal_state_influence_score"] >= 0
