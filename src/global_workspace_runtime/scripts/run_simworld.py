"""Run the closed cooperative SimWorld test environment."""
from __future__ import annotations

import argparse
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
PROJECT_PARENT = ROOT.parent
if str(PROJECT_PARENT) not in sys.path:
    sys.path.insert(0, str(PROJECT_PARENT))

from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig
from global_workspace_runtime.simworld import CooperativeSupportWorld, SimWorldRunner


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cycles", type=int, default=25)
    parser.add_argument("--seed", type=int, default=7)
    parser.add_argument("--artifact", type=str, default="artifacts/simworld/simworld.jsonl")
    args = parser.parse_args()

    runtime = GlobalWorkspaceRuntime(RuntimeConfig(fast_path_enabled=False, semantic_cache_enabled=False))
    world = CooperativeSupportWorld(seed=args.seed)
    runner = SimWorldRunner(runtime, world)
    result = runner.run(cycles=args.cycles, artifact_path=args.artifact)

    print("=== SimWorld Summary ===")
    for key, value in result["summary"].items():
        print(f"{key}: {value}")
    print(f"Wrote: {args.artifact}")


if __name__ == "__main__":
    main()
