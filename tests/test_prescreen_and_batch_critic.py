
from global_workspace_runtime.cognition import LLMAdapter, Critic, prescreen_candidates
from global_workspace_runtime.core.types import InternalState


def test_prescreen_keeps_best_candidate_subset():
    llm = LLMAdapter()
    state = InternalState(curiosity=0.9, threat=0.1)
    candidates = llm.generate_candidates("associative", {"text": "build options"}, [], state, 8)
    kept = prescreen_candidates(candidates, keep_fraction=0.5, min_keep=2)
    assert 2 <= len(kept) <= 4
    assert {c.candidate_id for c in kept}.issubset({c.candidate_id for c in candidates})


def test_critic_score_many_matches_single_scoring_count():
    llm = LLMAdapter()
    state = InternalState()
    candidates = llm.generate_candidates("analytic", {"text": "verify facts"}, [], state, 4)
    critic = Critic()
    scored = critic.score_many(candidates, state, [], None)
    assert len(scored) == len(candidates)
    assert all(score.total_score != 0 for _, score in scored)
