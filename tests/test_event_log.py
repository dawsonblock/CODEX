"""Tests for core/event_log.py (Task 5 — Python event sourcing)."""
from __future__ import annotations

import json
import tempfile
from pathlib import Path

from global_workspace_runtime.core.event_log import EventLog, RuntimeEvent
from global_workspace_runtime.core import GlobalWorkspaceRuntime, RuntimeConfig


# ---------------------------------------------------------------------------
# Unit tests for EventLog directly
# ---------------------------------------------------------------------------

def test_event_log_append_and_query_in_memory():
    log = EventLog(path=None)
    log.append(1, "cycle_start", {"text": "hello"})
    log.append(1, "candidate_selected", {"action_type": "answer"})
    log.append(2, "cycle_start", {"text": "world"})

    assert len(log) == 3
    c1_events = log.by_cycle(1)
    assert len(c1_events) == 2
    assert c1_events[0].event_type == "cycle_start"
    assert c1_events[1].event_type == "candidate_selected"


def test_event_log_filter_by_event_type():
    log = EventLog(path=None)
    for i in range(1, 4):
        log.append(i, "cycle_start", {"text": f"turn {i}"})
        log.append(i, "candidate_selected", {"action_type": "answer"})

    starts = log.events(event_type="cycle_start")
    assert len(starts) == 3
    for evt in starts:
        assert evt.event_type == "cycle_start"


def test_event_log_persists_to_disk():
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "sub" / "test.evlog"
        log = EventLog(path=path)
        log.append(1, "cycle_start", {"text": "persisted"})
        log.append(1, "candidate_selected", {"action_type": "defer_insufficient_evidence"})

        assert path.exists()
        lines = path.read_text().splitlines()
        assert len(lines) == 2

        # Each line must be valid JSON with required keys
        for line in lines:
            d = json.loads(line)
            for key in ("event_id", "cycle_id", "event_type", "timestamp", "payload"):
                assert key in d, f"Missing key {key!r} in event line"


def test_event_log_roundtrip_from_disk():
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "roundtrip.evlog"
        log = EventLog(path=path)
        log.append(7, "world_feedback", {"world_resources": 0.85})

        lines = path.read_text().splitlines()
        evt = RuntimeEvent.from_json(lines[0])
        assert evt.cycle_id == 7
        assert evt.event_type == "world_feedback"
        assert evt.payload["world_resources"] == 0.85


def test_event_log_empty_payload_default():
    log = EventLog(path=None)
    evt = log.append(3, "custom_event")
    assert evt.payload == {}


# ---------------------------------------------------------------------------
# Integration test: EventLog emits cycle_start + candidate_selected via runtime
# ---------------------------------------------------------------------------

def test_runtime_emits_cycle_events():
    cfg = RuntimeConfig(fast_path_enabled=False, long_term_archive_enabled=False)
    rt = GlobalWorkspaceRuntime(config=cfg)

    rt.run_cycle("What is 2+2?")
    rt.run_cycle("Tell me about the sky")

    log = rt.state.event_log
    assert len(log) == 4, f"Expected 4 events (2x cycle_start + 2x candidate_selected), got {len(log)}"

    starts = log.events(event_type="cycle_start")
    assert len(starts) == 2
    assert starts[0].payload["text"] == "What is 2+2?"
    assert starts[1].payload["text"] == "Tell me about the sky"

    selected = log.events(event_type="candidate_selected")
    assert len(selected) == 2
    for s in selected:
        assert "action_type" in s.payload
        assert "selected_text" in s.payload


def test_runtime_cycle_ids_are_sequential_in_event_log():
    cfg = RuntimeConfig(fast_path_enabled=False, long_term_archive_enabled=False)
    rt = GlobalWorkspaceRuntime(config=cfg)

    for i in range(1, 4):
        rt.run_cycle(f"turn {i}")

    starts = rt.state.event_log.events(event_type="cycle_start")
    assert [e.cycle_id for e in starts] == [1, 2, 3]
