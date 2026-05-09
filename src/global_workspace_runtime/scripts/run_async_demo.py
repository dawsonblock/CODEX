"""Run the async GlobalWorkspaceRuntime demo."""
from __future__ import annotations

import asyncio
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from global_workspace_runtime import AsyncGlobalWorkspaceRuntime, RuntimeConfig


async def main() -> None:
    rt = AsyncGlobalWorkspaceRuntime(RuntimeConfig())
    result = await rt.run_cycle_async(
        "How should we make this LLM more mind-like without pretending it has subjective experience?",
        force_slow=True,
    )
    print(json.dumps({
        "cycle_id": result["cycle_id"],
        "async_runtime": result.get("async_runtime", False),
        "candidate_budget": result.get("candidate_budget"),
        "raw_analytic_count": result.get("raw_analytic_count"),
        "raw_associative_count": result.get("raw_associative_count"),
        "workspace_shortlist": result.get("workspace_shortlist"),
        "selected_text": result.get("selected_text"),
        "trace_path": result.get("trace_path"),
    }, indent=2))


if __name__ == "__main__":
    asyncio.run(main())
