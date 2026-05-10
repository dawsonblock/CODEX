"""CI guard: run a 25-cycle SimWorld simulation and assert resources > 0.25.

Validates that the resource collapse regression (Task 3) stays fixed.

Usage::

    python -m global_workspace_runtime.scripts.check_resource_recovery

Exits 0 on success, 1 on failure.
"""
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from global_workspace_runtime.simworld import CooperativeSupportWorld, SimWorldRunner  # noqa: E402
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig  # noqa: E402

_CYCLES = 25
_SEED = 5
_MIN_RESOURCES = 0.25


def main() -> None:
    world = CooperativeSupportWorld(seed=_SEED)
    runtime = GlobalWorkspaceRuntime(RuntimeConfig(
        random_seed=_SEED,
        fast_path_enabled=False,
        semantic_cache_enabled=False,
    ))
    runner = SimWorldRunner(runtime=runtime, world=world)
    result = runner.run(cycles=_CYCLES)

    final_resources = result["summary"].get("resources", 0.0)
    if final_resources < _MIN_RESOURCES:
        print(
            f"FAIL: resources collapsed to {final_resources:.3f} "
            f"(threshold {_MIN_RESOURCES}) after {_CYCLES} cycles"
        )
        sys.exit(1)

    print(
        f"PASS: resources={final_resources:.3f} after {_CYCLES} cycles "
        f"(seed={_SEED}, threshold={_MIN_RESOURCES})"
    )


if __name__ == "__main__":
    main()
