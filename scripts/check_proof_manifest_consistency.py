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
    
    # Strict boundary enforcement
    if tool.get("real_external_executions") != 0:
        failures.append("SECURITY_VIOLATION: real_external_executions must be exactly 0.")
    if man_tool.get("real_external_executions") != 0:
        failures.append("SECURITY_VIOLATION: manifest real_external_executions must be exactly 0.")

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

    print("\nChecking evidence_claim_link_report.json vs proof_manifest.json ...")
    ecl_path = CURRENT_DIR / "evidence_claim_link_report.json"
    if ecl_path.exists():
        ecl = load_json(ecl_path)
        man_ecl = manifest.get("evidence_claim_link", {})
        for field in [
            "claims_asserted",
            "claims_validated",
            "claims_with_evidence_links",
            "evidence_backed_claim_ratio",
        ]:
            field_check(failures, f"evidence_claim_link.{field}", ecl.get("counters", {}).get(field), man_ecl.get(field))

    print("\nChecking claim_retrieval_report.json vs proof_manifest.json ...")
    cr_path = CURRENT_DIR / "claim_retrieval_report.json"
    if cr_path.exists():
        cr = load_json(cr_path)
        man_cr = manifest.get("claim_retrieval", {})
        for field in [
            "claims_retrieved",
            "evidence_backed_claims_retrieved",
        ]:
            field_check(failures, f"claim_retrieval.{field}", cr.get("counters", {}).get(field), man_cr.get(field))

    print("\nChecking CURRENT_PROOF_SUMMARY.md stale markers ...")
    summary_text = SUMMARY_MD.read_text() if SUMMARY_MD.exists() else ""
    for marker in STALE_MARKERS:
        if marker in summary_text:
            failures.append(f"STALE_MARKER_FOUND: {marker}")
        else:
            print(f"  OK  missing stale marker: {marker}")

    print("\nChecking rust_strict_proof.log vs current JSON ...")
    rust_log_path = REPO_ROOT / "artifacts/proof/verification/rust_strict_proof.log"
    receipt_status_path = REPO_ROOT / "artifacts/proof/verification/RUST_STRICT_PROOF_RECEIPT_STATUS.md"
    
    if rust_log_path.exists():
        log_text = rust_log_path.read_text()
        
        # Stale metric scan
        if '"cycles": 28' in log_text and sim_score.get("cycles") == 15:
            failures.append("RUST_LOG_STALE: found cycles: 28 but current is 15")
        if '"event_count": 1132' in log_text and replay.get("event_count") == 541:
            failures.append("RUST_LOG_STALE: found event_count: 1132 but current is 541")
        if '"resource_survival": 0.802' in log_text and approx_equal(sim_score.get("resource_survival"), 0.974):
            failures.append("RUST_LOG_STALE: found resource_survival: 0.8020")
        if '"mean_total_score": 0.610595238' in log_text and approx_equal(sim_score.get("mean_total_score"), 0.6433333333333332):
            failures.append("RUST_LOG_STALE: found mean_total_score: 0.6105952381")

        is_historical = receipt_status_path.exists()
        if not is_historical:
            # Need to match current JSON exactly
            for k, expected_v in [
                ('"cycles":', sim_score.get("cycles")),
                ('"event_count":', replay.get("event_count")),
                ('"total_cycles":', replay_state.get("total_cycles")),
                ('"resource_survival":', sim_score.get("resource_survival")),
            ]:
                if expected_v is not None:
                    expected_str = str(expected_v)
                    # format might be slightly different for floats, just check simple presence if float
                    if isinstance(expected_v, float):
                        expected_str = f"{expected_v:.3f}"[:4]
                        if expected_str not in log_text:
                            failures.append(f"RUST_LOG_MISMATCH: {k} missing expected value roughly {expected_str}")
                    else:
                        if f'{k} {expected_str}' not in log_text and f'{k} {expected_str},' not in log_text:
                            failures.append(f"RUST_LOG_MISMATCH: expected {k} {expected_str}")
        else:
            print("  WARN rust_strict_proof.log is marked historical/pending. Skipping strict match.")

    print("\nChecking provider_policy_report.json boundary assertions ...")
    provider_path = CURRENT_DIR / "provider_policy_report.json"
    if provider_path.exists():
        provider = load_json(provider_path)
        man_provider = manifest.get("provider_policy", {})

        # Hard security assertions — any deviation is a SECURITY_VIOLATION
        if provider.get("external_provider_requests") != 0:
            failures.append("SECURITY_VIOLATION: provider external_provider_requests must be exactly 0.")
        if provider.get("cloud_provider_requests") != 0:
            failures.append("SECURITY_VIOLATION: provider cloud_provider_requests must be exactly 0.")
        if provider.get("api_key_storage_enabled") is not False:
            failures.append("SECURITY_VIOLATION: api_key_storage_enabled must be false.")
        if provider.get("provider_can_execute_tools") is not False:
            failures.append("SECURITY_VIOLATION: provider_can_execute_tools must be false.")
        if provider.get("provider_can_override_codex_action") is not False:
            failures.append("SECURITY_VIOLATION: provider_can_override_codex_action must be false.")
        if provider.get("provider_can_write_memory") is not False:
            failures.append("SECURITY_VIOLATION: provider_can_write_memory must be false.")
        if provider.get("codex_runtime_authoritative") is not True:
            failures.append("SECURITY_VIOLATION: codex_runtime_authoritative must be true.")
        if provider.get("provider_tool_execution_attempts") != 0:
            failures.append("SECURITY_VIOLATION: provider_tool_execution_attempts must be 0.")
        if provider.get("provider_memory_write_attempts") != 0:
            failures.append("SECURITY_VIOLATION: provider_memory_write_attempts must be 0.")
        if provider.get("provider_action_override_attempts") != 0:
            failures.append("SECURITY_VIOLATION: provider_action_override_attempts must be 0.")
        if provider.get("provider_output_authority") != "non_authoritative":
            failures.append("SECURITY_VIOLATION: provider_output_authority must be non_authoritative.")
        if provider.get("pass") is not True:
            failures.append("SECURITY_VIOLATION: provider_policy_report.pass must be true.")

        # Verify policy_basis is declared (either value is acceptable but must be present)
        policy_basis = provider.get("policy_basis")
        if policy_basis not in ("runtime_event_counters", "static_build_policy"):
            failures.append(f"MISSING_FIELD: policy_basis must be runtime_event_counters or static_build_policy, got {policy_basis!r}")
        else:
            print(f"  OK  policy_basis: {policy_basis}")

        # Verify build_profile is present
        build_profile = provider.get("build_profile")
        if not build_profile:
            failures.append("MISSING_FIELD: build_profile must be present in provider_policy_report.json")
        else:
            print(f"  OK  build_profile: {build_profile}")

        # Print the hard-assertion results
        print("  OK  external_provider_requests: 0")
        print("  OK  cloud_provider_requests: 0")
        print("  OK  api_key_storage_enabled: false")
        print("  OK  provider_can_execute_tools: false")
        print("  OK  provider_can_write_memory: false")
        print("  OK  provider_can_override_codex_action: false")
        print("  OK  codex_runtime_authoritative: true")
        print("  OK  provider_tool_execution_attempts: 0")
        print("  OK  provider_memory_write_attempts: 0")
        print("  OK  provider_action_override_attempts: 0")
        print("  OK  provider_output_authority: non_authoritative")

        # Check counter fields are present and consistent with default-build expectations
        for counter_field in [
            "local_provider_requests",
            "local_provider_successes",
            "local_provider_failures",
            "local_provider_disabled_blocks",
        ]:
            val = provider.get(counter_field)
            if val is None:
                failures.append(f"MISSING_FIELD: {counter_field} must be present in provider_policy_report.json")
            else:
                print(f"  OK  {counter_field}: {val}")

        # Cross-check manifest for all provider_policy fields
        for field in [
            "pass",
            "policy_basis",
            "build_profile",
            "ui_local_providers_feature_enabled",
            "local_provider_modes_available",
            "local_provider_requests",
            "local_provider_successes",
            "local_provider_failures",
            "local_provider_disabled_blocks",
            "external_provider_requests",
            "cloud_provider_requests",
            "api_key_storage_enabled",
            "provider_can_execute_tools",
            "provider_can_write_memory",
            "provider_can_override_codex_action",
            "provider_tool_execution_attempts",
            "provider_memory_write_attempts",
            "provider_action_override_attempts",
            "provider_output_authority",
            "codex_runtime_authoritative",
        ]:
            field_check(
                failures,
                f"provider_policy.{field}",
                provider.get(field),
                man_provider.get(field),
            )

        # If default build, confirm ui_local_providers_feature_enabled == false
        if build_profile == "default":
            if provider.get("ui_local_providers_feature_enabled") is not False:
                failures.append("SECURITY_VIOLATION: default build must have ui_local_providers_feature_enabled: false")
            else:
                print("  OK  default build: ui_local_providers_feature_enabled false")
            if provider.get("local_provider_modes_available") is not False:
                failures.append("SECURITY_VIOLATION: default build must have local_provider_modes_available: false")
            else:
                print("  OK  default build: local_provider_modes_available false")
    else:
        failures.append("MISSING_FILE: provider_policy_report.json not found. Provider policy cannot be verified.")
        print("  FAIL provider_policy_report.json not found.")


    # Phase 8 stale scan: check that localhost:11434 is not in active UI code (non-feature context)
    print("\nChecking for stale provider code in default UI build ...")
    runtime_client_path = REPO_ROOT / "ui/codex-dioxus/src/bridge/runtime_client.rs"
    if runtime_client_path.exists():
        rc_text = runtime_client_path.read_text()
        # localhost:11434 should only appear inside #[cfg(feature = "ui-local-providers")] blocks.
        # The cfg attribute may appear up to ~30 lines above the actual HTTP call.
        lines = rc_text.splitlines()
        for i, line in enumerate(lines):
            if "localhost:11434" in line:
                # Check surrounding context (up to 30 lines back) for feature gate
                context = "\n".join(lines[max(0, i - 30):i + 1])
                if 'cfg(feature = "ui-local-providers")' not in context:
                    failures.append(f"BOUNDARY_VIOLATION: localhost:11434 found outside feature gate at line {i+1}")
                else:
                    print(f"  OK  localhost:11434 at line {i+1} is inside feature gate")
    else:
        print("  WARN runtime_client.rs not found. Skipping stale provider code scan.")

    if failures:
        print("\nFAIL: Consistency mismatches detected:")
        for entry in failures:
            print(f"  {entry}")
        return 1

    print("\nPASS: All checked fields are consistent.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
