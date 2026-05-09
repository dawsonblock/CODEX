from global_workspace_runtime.core import RuntimeState, BackgroundProcessor
from global_workspace_runtime.modulation.somatic import SomaticMap, SOMATIC_DIMENSIONS
from global_workspace_runtime.core.types import InternalState


def test_somatic_map_updates_repeatable_pressure_pattern():
    state = InternalState(threat=0.8, uncertainty=0.7, control=0.2, resource_pressure=0.6, kindness=0.4)
    somatic = SomaticMap(hysteresis=0.0)
    somatic.update(state, {"contradiction": 0.9, "memory_conflict": 0.8}, scratchpad_pressure=0.5)
    assert len(somatic.values) == len(SOMATIC_DIMENSIONS)
    assert somatic.pressure("contradiction_pressure") == 0.9
    assert somatic.pressure("control_loss") > 0.7
    assert somatic.predicts_bad_outcome()


def test_background_processor_reviews_unresolved_questions():
    state = RuntimeState()
    state.internal_state.uncertainty = 0.8
    state.scratchpad.write_conflict("How should conflict be resolved?")
    result = BackgroundProcessor().tick(state)
    assert result.ran
    assert "scratchpad_review" in result.actions
    assert state.scratchpad.written_summary
