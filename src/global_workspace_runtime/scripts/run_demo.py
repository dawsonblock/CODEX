from __future__ import annotations
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def main() -> None:
    rt = GlobalWorkspaceRuntime(RuntimeConfig())
    result = rt.run_cycle("How should we make this LLM more mind-like without pretending it has mental status?", force_slow=True)
    print("=== GlobalWorkspaceRuntime Demo ===")
    print("Internal state:")
    print(json.dumps(result["internal_state"], indent=2, sort_keys=True))
    print("\nResonance tags:")
    for tag in result["resonance_tags"]:
        print(f"- {tag['name']} / {tag['texture']} / intensity={tag['intensity']:.2f}")
    print(f"\nCandidate budget: {result['candidate_budget']}")
    print("\nAnalytic candidates:")
    for c in result["analytic_candidates"]:
        print(f"- {c}")
    print("\nAssociative candidates:")
    for c in result["associative_candidates"]:
        print(f"- {c}")
    print("\nBridge:")
    print(json.dumps({k: result['bridge'][k] for k in ['agreement_score','contradiction_score','novelty_delta','evidence_gap','hemispheric_tension','conflicts']}, indent=2, sort_keys=True))
    print("\nWorkspace shortlist:")
    for c in result["workspace_shortlist"]:
        print(f"- {c}")
    print("\nSelected candidate:")
    print(result["selected_text"])
    print("\nRejected candidates:")
    for c in result["rejected_candidates"]:
        print(f"- {c}")
    print("\nMemory write:")
    print(result["memory_episode_id"])
    print("Scratchpad:", result["scratchpad_summary"])
    print("Trace:", result["trace_path"])


if __name__ == "__main__":
    main()
