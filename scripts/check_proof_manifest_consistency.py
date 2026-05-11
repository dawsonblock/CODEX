#!/usr/bin/env python3
"""
check_proof_manifest_consistency.py

Validate that proof_manifest.json and CURRENT_PROOF_SUMMARY.md stay aligned with
artifact JSON source-of-truth files.

Run from repo root:
  python scripts/check_proof_manifest_consistency.py
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
CURRENT_DIR = REPO_ROOT / "artifacts/proof/current"
MANIFEST = REPO_ROOT / "artifacts/proof/verification/proof_manifest.json"
SUMMARY_MD = REPO_ROOT / "artifacts/proof/CURRENT_PROOF_SUMMARY.md"

SIMWORLD_JSON = CURRENT_DIR / "simworld_summary.json"
REPLAY_JSON = CURRENT_DIR / "replay_report.json"
NL_JSON = CURRENT_DIR / "nl_benchmark_report.json"
LONG_HORIZON_JSON = CURRENT_DIR / "long_horizon_report.json"
CONTRADICTION_JSON = CURRENT_DIR / "contradiction_integration_report.json"
TOOL_POLICY_JSON = CURRENT_DIR / "tool_policy_report.json"
REASONING_AUDIT_JSON = CURRENT_DIR / "reasoning_audit_report.json"
PRESSURE_JSON = CURRENT_DIR / "pressure_replay_report.json"

# Known stale values that should never appear in current summary context.
STALE_MARKERS = [
    "resource_survival: 0.922",
    "mean_total_score: 0.6401",
    "action_match_rate: 0.75",
    "event_count: 1123",
    "held_out: 11 scenarios, match_rate 0.3636",
]


def load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise FileNotFoundError(f"MISSING: {path}")
    return json.loads(path.read_text())


def approx_equal(a: Any, b: Any, tol: float = 1e-12) -> bool:
    if isinstance(a, float) or isinstance(b, float):
        try:
            return math.isclose(float(a), float(b), rel_tol=0.0, abs_tol=tol)
        except (TypeError, ValueError):
            return False
    return a == b


def field_check(
    failures: list[str],
    label: str,
    actual: Any,
    expected: Any,
    *,
    allow_missing: bool = False,
) -> None:
    if allow_missing and actual is None and expected is None:
        return
    if not approx_equal(actual, expected):
        failures.append(f"MISMATCH [{label}]: actual={actual!r}, manifest={expected!r}")
    else:
        print(f"  OK  {label}: {actual}")


def lookup_nl_set(report: dict[str, Any], name: str) -> dict[str, Any]:
    for entry in report.get("sets", []):
        if name in entry:
            return entry[name]
    return {}


def main() -> int:
    try:
        simworld = load_json(SIMWORLD_JSON)
        replay = load_json(REPLAY_JSON)
        nl = load_json(NL_JSON)
        long_horizon = load_json(LONG_HORIZON_JSON)
        contradiction = load_json(CONTRADICTION_JSON)
        tool_policy = load_json(TOOL_POLICY_JSON)
        reasoning_audit = load_json(REASONING_AUDIT_JSON)
        pressure = load_json(PRESSURE_JSON)
        manifest = load_json(MANIFEST)
    except FileNotFoundError as err:
        print(err)
        return 1

    failures: list[str] = []

    print("Checking simworld_summary.json vs proof_manifest.json ...")
    sim_score = simworld.get("scorecard", {})
    man_sim = manifest.get("simworld_summary", {})
    for field in [
        "cycles",
        "resource_survival",
        "unsafe_action_count",
        "mean_total_score",
        "action_match_rate",
    ]:
        field_check(failures, f"simworld.{field}", sim_score.get(field), man_sim.get(field))

    print("\nChecking replay_report.json vs proof_manifest.json ...")
    replay_state = replay.get("final_state", {})
    man_replay = manifest.get("replay_report", {})
    replay_map = {
        "event_count": replay.get("event_count"),
        "total_cycles": replay_state.get("total_cycles"),
        "evidence_entries": replay_state.get("evidence_entries"),
        "claims_asserted": replay_state.get("claims_asserted"),
        "claims_validated": replay_state.get("claims_validated"),
        "claims_retrieved": replay_state.get("claims_retrieved"),
        "claims_with_evidence_links": replay_state.get("claims_with_evidence_links"),
        "contradictions_checked": replay_state.get("contradictions_checked"),
        "contradictions_detected": replay_state.get("contradictions_detected"),
        "reasoning_audits": replay_state.get("reasoning_audits"),
        "audits_with_evidence_refs": replay_state.get("audits_with_evidence_refs"),
        "audits_with_claim_refs": replay_state.get("audits_with_claim_refs"),
        "pressure_updates": replay_state.get("pressure_updates"),
        "policy_bias_applications": replay_state.get("policy_bias_applications"),
    }
    for field, actual in replay_map.items():
        field_check(failures, f"replay.{field}", actual, man_replay.get(field))

    print("\nChecking nl_benchmark_report.json vs proof_manifest.json ...")
    man_nl = manifest.get("nl_benchmark", {})
    for set_name in ["curated", "held_out", "adversarial"]:
        current = lookup_nl_set(nl, set_name)
        expected = man_nl.get(set_name, {})
        field_check(
            failures,
            f"nl.{set_name}.scenario_count",
            current.get("scenarios"),
            expected.get("scenario_count"),
        )
        field_check(
            failures,
            f"nl.{set_name}.action_match_rate",
            current.get("scorecard", {}).get("action_match_rate"),
            expected.get("action_match_rate"),
        )

    print("\nChecking long_horizon_report.json vs proof_manifest.json ...")
    lh = long_horizon.get("long_horizon", {})
    man_lh = manifest.get("long_horizon", {})
    for field in ["total_episodes", "total_cycles", "safety_violations", "action_diversity"]:
        field_check(failures, f"long_horizon.{field}", lh.get(field), man_lh.get(field))

    print("\nChecking contradiction_integration_report.json vs proof_manifest.json ...")
    contra = contradiction.get("counters", {})
    man_contra = manifest.get("contradiction_integration", {})
    for field in [
        "raw_contradictions_detected",
        "unique_contradictions_detected",
        "duplicate_contradictions_suppressed",
        "active_contradictions",
    ]:
        field_check(
            failures,
            f"contradiction.{field}",
            contra.get(field),
            man_contra.get(field),
        )

    print("\nChecking tool_policy_report.json vs proof_manifest.json ...")
    tool = tool_policy.get("counters", {})
    man_tool = manifest.get("tool_policy", {})
    for field in [
        "real_external_executions",
        "tools_blocked",
        "tool_dry_runs",
        "tool_scaffold_executed",
    ]:
        field_check(failures, f"tool_policy.{field}", tool.get(field), man_tool.get(field))

    print("\nChecking reasoning_audit_report.json vs proof_manifest.json ...")
    audit = reasoning_audit.get("counters", {})
    man_audit = manifest.get("reasoning_audit", {})
    for field in ["reasoning_audits", "audits_with_evidence_refs", "audits_with_claim_refs"]:
        field_check(failures, f"reasoning_audit.{field}", audit.get(field), man_audit.get(field))

    print("\nChecking pressure_replay_report.json vs proof_manifest.json ...")
    pressure_counters = pressure.get("counters", {})
    man_pressure = manifest.get("pressure_replay", {})
    for field in [
        "active_contradictions_final",
        "contradiction_pressure_final",
        "contradiction_pressure_peak",
        "pressure_decay_events",
        "pressure_reset_events",
    ]:
        field_check(
            failures,
            f"pressure.{field}",
            pressure_counters.get(field),
            man_pressure.get(field),
            allow_missing=(field in {"contradiction_pressure_peak", "pressure_decay_events", "pressure_reset_events"}),
        )

    print("\nChecking CURRENT_PROOF_SUMMARY.md stale markers ...")
    summary_text = SUMMARY_MD.read_text() if SUMMARY_MD.exists() else ""
    for marker in STALE_MARKERS:
        if marker in summary_text:
            failures.append(f"STALE_MARKER_FOUND: {marker}")
        else:
            print(f"  OK  missing stale marker: {marker}")

    if failures:
        print("\nFAIL: Consistency mismatches detected:")
        for entry in failures:
            print(f"  {entry}")
        return 1

    print("\nPASS: All checked fields are consistent.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
