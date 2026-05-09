from __future__ import annotations
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


def fail(msg: str) -> None:
    raise SystemExit(f"FAIL: {msg}")


def main() -> None:
    rt = GlobalWorkspaceRuntime(RuntimeConfig())
    r1 = rt.run_cycle("Design the runtime with high uncertainty and careful evidence checks.", force_slow=True)
    r2 = rt.run_cycle("Design the runtime with high curiosity and low risk exploration.", force_slow=True)
    if not Path(r1["trace_path"]).exists(): fail("trace missing")
    if len(r1["workspace_shortlist"]) > rt.config.workspace_capacity: fail("workspace capacity exceeded")
    if not r1["scratchpad_summary"] and len(r1["analytic_candidates"] + r1["associative_candidates"]) > rt.config.workspace_capacity: fail("scratchpad overflow missing")
    if r1["selected_text"] == r2["selected_text"] and r1["internal_state"] != r2["internal_state"]: fail("state influence not visible")
    if len(r1["analytic_candidates"]) == 0 or len(r1["associative_candidates"]) == 0: fail("stream output missing")
    if r1["analytic_candidates"] == r1["associative_candidates"]: fail("streams collapsed into identical outputs")
    print("PASS: architecture integrity checks passed")


if __name__ == "__main__":
    main()
