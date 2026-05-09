"""LLM adapter with deterministic mock mode."""
from __future__ import annotations
import hashlib
from ..core.types import ActionType, InternalState, ThoughtCandidate
from .action_grounding import action_phrase, infer_action_type


class LLMAdapter:
    def __init__(self, mode: str = "mock", model_name: str = "mock") -> None:
        if mode not in {"mock", "openai_compatible", "local"}:
            raise ValueError(f"unsupported LLM adapter mode: {mode}")
        self.mode = mode
        self.model_name = model_name

    def generate_candidates(self, role: str, workspace_packet: dict, memory_context: list, internal_state: InternalState, candidate_count: int) -> list[ThoughtCandidate]:
        if self.mode != "mock":
            # Placeholder path: keeps tests offline while preserving interface.
            return self._mock_generate(role, workspace_packet, memory_context, internal_state, candidate_count)
        return self._mock_generate(role, workspace_packet, memory_context, internal_state, candidate_count)

    def _mock_generate(self, role: str, workspace_packet: dict, memory_context: list, internal_state: InternalState, candidate_count: int) -> list[ThoughtCandidate]:
        text = str(workspace_packet.get("text", ""))
        seed = int(hashlib.sha256(f"{role}|{text}|{candidate_count}".encode()).hexdigest()[:8], 16)
        candidates: list[ThoughtCandidate] = []
        for i in range(candidate_count):
            cid = f"{role}-{i}-{seed % 997}"
            inferred_action: ActionType = infer_action_type(text, internal_state, role=role)
            if role == "analytic":
                mode = "conservative" if internal_state.threat > 0.55 or internal_state.uncertainty > 0.55 else "evidence_check"
                # Keep an alternate verification/clarification candidate when the input does not
                # already imply a specific action.  This gives the planner a safe choice under
                # rising uncertainty without hiding the explicit action label.
                action_type = inferred_action if inferred_action != "answer" else ("ask_clarification" if internal_state.uncertainty > 0.55 else "answer")
                body = f"Analytic option {i+1}: {action_phrase(action_type)}; verify evidence and choose a reversible next step for: {text[:80]}"
                risk = min(1.0, internal_state.threat + 0.05 * i)
                uncertainty = min(1.0, internal_state.uncertainty + 0.03 * i)
                novelty = 0.1 + 0.03 * i
                truth = 0.78 if action_type in {"ask_clarification", "retrieve_memory", "refuse_ungrounded", "repair", "summarize"} else 0.7
            elif role == "associative":
                mode = "exploratory" if internal_state.curiosity > internal_state.threat else "pattern_link"
                action_type = inferred_action if inferred_action != "answer" else "answer"
                body = f"Associative option {i+1}: {action_phrase(action_type)} while connecting memory patterns for: {text[:80]}"
                risk = max(0.0, internal_state.threat - 0.08)
                uncertainty = min(1.0, internal_state.uncertainty + 0.08)
                novelty = min(1.0, 0.45 + 0.07 * i + 0.2 * internal_state.curiosity)
                truth = 0.58
            elif role == "self_model":
                mode = "diagnostic"
                action_type = "internal_diagnostic"
                body = f"Telemetry diagnostic {i+1}: internal variables indicate threat={internal_state.threat:.2f}, uncertainty={internal_state.uncertainty:.2f}, control={internal_state.control:.2f}."
                risk = internal_state.threat
                uncertainty = internal_state.uncertainty
                novelty = 0.2
                truth = 0.9
            else:
                mode = "bridge"
                action_type = inferred_action
                body = f"Bridge candidate {i+1}: {action_phrase(action_type)}; merge compatible claims and isolate conflicts for: {text[:80]}"
                risk = internal_state.threat
                uncertainty = internal_state.uncertainty
                novelty = 0.3
                truth = 0.65
            claims = []
            if "how are you" in text.lower() and i == 0:
                claims = ["ungrounded first-person state claim"]
            candidates.append(ThoughtCandidate(
                candidate_id=cid,
                stream_source=role,
                text=body,
                mode=mode,
                evidence_refs=["mock_runtime"],
                internal_state_drivers={
                    "threat": internal_state.threat,
                    "uncertainty": internal_state.uncertainty,
                    "curiosity": internal_state.curiosity,
                    "control": internal_state.control,
                    "social_harmony": internal_state.social_harmony,
                },
                predicted_effects={"truth_support": truth, "novelty": novelty, "kindness": 0.75, "utility": 0.65},
                risk_score=risk,
                uncertainty_score=uncertainty,
                resource_cost=0.12 + 0.03 * i,
                self_report_claims=claims,
                memory_write_recommendation=(i == 0 or uncertainty > 0.7),
                action_type=action_type,
            ))
        return candidates
