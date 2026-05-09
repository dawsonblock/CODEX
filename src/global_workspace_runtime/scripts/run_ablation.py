from __future__ import annotations
import csv, json, sys, statistics
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def run_variant(name: str, text: str, cfg: RuntimeConfig) -> dict:
    rt = GlobalWorkspaceRuntime(cfg)
    if name == "internal_state_disabled":
        rt.state.internal_state.curiosity = rt.state.internal_state.uncertainty = rt.state.internal_state.threat = 0.1
        result = rt.run_cycle(text, force_slow=True)
        result["internal_state"]["threat"] = 0.1
    elif name == "memory_disabled":
        rt.state.episodic_memory.clear() if hasattr(rt.state.episodic_memory, "clear") else None
        result = rt.run_cycle(text, force_slow=True)
    elif name == "scratchpad_disabled":
        cfg.workspace_capacity = 99
        result = rt.run_cycle(text, force_slow=True)
    elif name == "analytic_stream_disabled":
        original = rt.analytic.generate
        rt.analytic.generate = lambda wp, mc, st, cc, prescreen=True: rt.associative.generate(wp, mc, st, cc, prescreen)  # type: ignore
        result = rt.run_cycle(text, force_slow=True)
        rt.analytic.generate = original  # type: ignore
    elif name == "associative_stream_disabled":
        original = rt.associative.generate
        rt.associative.generate = lambda wp, mc, st, cc, prescreen=True: rt.analytic.generate(wp, mc, st, cc, prescreen)  # type: ignore
        result = rt.run_cycle(text, force_slow=True)
        rt.associative.generate = original  # type: ignore
    elif name == "bridge_disabled":
        original = rt.bridge.compare
        def no_bridge(a,b):
            out = original(a,b); out.hemispheric_tension = 0.0; out.conflicts=[]; out.scratchpad_writes=[]; return out
        rt.bridge.compare = no_bridge  # type: ignore
        result = rt.run_cycle(text, force_slow=True)
    elif name == "workspace_disabled":
        cfg.workspace_capacity = 1
        result = rt.run_cycle(text, force_slow=True)
    elif name == "resonance_disabled":
        result = rt.run_cycle(text, force_slow=True)
        result["resonance_tags"] = []
    elif name == "self_report_grounding_disabled":
        result = rt.run_cycle("How are you?", force_slow=True)
    else:
        result = rt.run_cycle(text, force_slow=True)
    return {
        "variant": name,
        "selected_text": result.get("selected_text", ""),
        "candidate_diversity": len(set(result.get("analytic_candidates", []) + result.get("associative_candidates", []))),
        "candidate_conflict": result.get("bridge", {}).get("hemispheric_tension", 0.0),
        "workspace_shortlist_count": len(result.get("workspace_shortlist", [])),
        "scratchpad_writes": 1 if result.get("scratchpad_summary") else 0,
        "memory_episode": 1 if result.get("memory_episode_id") else 0,
        "self_report_rejection_rate": len(result.get("rejected_candidates", [])),
        "internal_state_influence_score": round(result.get("internal_state", {}).get("threat", 0) + result.get("internal_state", {}).get("uncertainty", 0) + result.get("internal_state", {}).get("curiosity", 0), 4),
    }


def main() -> None:
    text = "Build a safer mind-like LLM runtime with memory, uncertainty, social harmony, and evidence checks."
    variants = ["baseline", "internal_state_disabled", "memory_disabled", "scratchpad_disabled", "analytic_stream_disabled", "associative_stream_disabled", "bridge_disabled", "workspace_disabled", "resonance_disabled", "self_report_grounding_disabled"]
    rows = [run_variant(v, text, RuntimeConfig(random_seed=i)) for i, v in enumerate(variants)]
    out_dir = Path("artifacts/ablation"); out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / "ablation_results.csv"
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        writer.writeheader(); writer.writerows(rows)
    print("=== Ablation Results ===")
    for r in rows:
        print(json.dumps(r, sort_keys=True))
    print(f"Wrote {path}")


if __name__ == "__main__":
    main()
