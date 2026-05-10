from global_workspace_runtime.cognition import MetaCritic, PredictiveGoalModel
from global_workspace_runtime.core.types import InternalState, ThoughtCandidate


def test_meta_critic_recommends_retrieval_after_rejections():
    events = [
        {"phase": "select", "option": "reject", "event_type": "planner_decision"},
        {"phase": "select", "option": "reject", "event_type": "planner_decision"},
        {"phase": "generate", "option": "conservative_generation", "event_type": "candidate_generation"},
    ]
    state = InternalState(threat=0.2, curiosity=0.2)
    report = MetaCritic().review(events, state)
    assert report.repeated_failure_pressure > 0.5
    assert "retrieve_more_memory_before_selection" in report.recommendations


def test_predictive_goal_model_penalizes_risky_unsupported_candidate():
    model = PredictiveGoalModel()
    candidate = ThoughtCandidate(
        candidate_id="c1",
        stream_source="analytic",
        text="Give an unsupported answer quickly.",
        mode="test",
        risk_score=0.8,
        uncertainty_score=0.8,
        resource_cost=0.2,
    )
    state = InternalState(uncertainty=0.9, threat=0.6)
    predicted = model.predict(candidate, state)
    assert predicted.surprise > 0.8
    assert predicted.utility_delta < 0.0
