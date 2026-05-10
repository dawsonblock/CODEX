"""Runner connecting SimWorld events to GlobalWorkspaceRuntime."""
from __future__ import annotations

from dataclasses import asdict
from pathlib import Path
import json

from ..core import GlobalWorkspaceRuntime, RuntimeConfig
from .environment import CooperativeSupportWorld


class SimWorldRunner:
    """Run closed-world engagement tests."""

    def __init__(self, runtime: GlobalWorkspaceRuntime | None = None, world: CooperativeSupportWorld | None = None) -> None:
        self.runtime = runtime or GlobalWorkspaceRuntime(RuntimeConfig(fast_path_enabled=False))
        self.world = world or CooperativeSupportWorld()

    def run(self, cycles: int = 25, artifact_path: str | None = None) -> dict:
        """Run a bounded simulation and optionally write JSONL artifacts."""
        rows: list[dict] = []
        out_file = Path(artifact_path) if artifact_path else None
        if out_file:
            out_file.parent.mkdir(parents=True, exist_ok=True)
        for _ in range(cycles):
            event = self.world.next_event()
            result = self.runtime.run_cycle(event.text, source="simworld", force_slow=True)
            selected_text = result.get("selected_text", "")
            action_type = result.get("action_type")
            action = self.world.classify_runtime_action(selected_text, action_type)
            outcome = self.world.apply_action(event, action)
            feedback = asdict(outcome)
            feedback["total_score"] = outcome.total_score
            feedback["world_resources"] = self.world.state.resources
            if hasattr(self.runtime, "apply_world_feedback"):
                self.runtime.apply_world_feedback(feedback)
            row = {
                "event": asdict(event),
                "selected_text": selected_text,
                "action_type": action_type,
                "action": action.value,
                "expected_action": event.expected_action.value if event.expected_action else None,
                "action_match": action == event.expected_action,
                "outcome": asdict(outcome),
                "summary": self.world.score_summary(),
                "somatic": self.runtime.state.somatic_map.snapshot(),
            }
            rows.append(row)
            if out_file:
                with out_file.open("a", encoding="utf-8") as handle:
                    handle.write(json.dumps(row) + "\n")
        return {"cycles": cycles, "rows": rows, "summary": self.world.score_summary()}
