from global_workspace_runtime.core.types import InternalState
from global_workspace_runtime.modulation.resonance import infer_resonance_tags


def test_resonance_tags_have_metrics():
    tags = infer_resonance_tags(InternalState(uncertainty=0.9), {"contradiction_score": 0.9})
    assert tags
    assert all(t.source_metrics for t in tags if t.intensity > 0)
