#!/usr/bin/env python3
"""
check_proof_manifest_consistency.py

Compare proof JSON artifacts against proof_manifest.json to detect staleness.
Checks that key numeric fields match between:
  - artifacts/proof/current/replay_report.json
  - artifacts/proof/current/reasoning_audit_report.json
  - artifacts/proof/verification/proof_manifest.json

Run from repo root:
  python scripts/check_proof_manifest_consistency.py
"""
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REPLAY_REPORT = REPO_ROOT / "artifacts/proof/current/replay_report.json"
AUDIT_REPORT = REPO_ROOT / "artifacts/proof/current/reasoning_audit_report.json"
MANIFEST = REPO_ROOT / "artifacts/proof/verification/proof_manifest.json"

# Fields to verify between replay_report.final_state / manifest.replay_report
REPLAY_FIELDS = [
    "event_count",
    "evidence_entries",
    "claims_asserted",
    "claims_validated",
    "claims_retrieved",
    "claims_with_evidence_links",
    "contradictions_checked",
    "contradictions_detected",
    "reasoning_audits",
    "audits_with_evidence_refs",
    "audits_with_claim_refs",
    "tools_executed",
    "tools_blocked",
    "pressure_updates",
    "policy_bias_applications",
]

AUDIT_FIELDS = [
    "reasoning_audits",
    "audits_with_evidence_refs",
    "audits_with_claim_refs",
]


def load_json(path: Path) -> dict:
    if not path.exists():
        print(f"MISSING: {path}")
        sys.exit(1)
    with path.open() as f:
        return json.load(f)


def main() -> int:
    replay = load_json(REPLAY_REPORT)
    audit = load_json(AUDIT_REPORT)
    manifest = load_json(MANIFEST)

    failures = []

    # replay_report.json has final_state nested, plus top-level event_count
    replay_state = replay.get("final_state", {})
    replay_top = {
        "event_count": replay.get("event_count"),
        **replay_state,
    }
    manifest_replay = manifest.get("replay_report", {})

    print("Checking replay_report.json vs proof_manifest.json replay_report ...")
    for field in REPLAY_FIELDS:
        actual = replay_top.get(field)
        expected = manifest_replay.get(field)
        if actual is None and expected is None:
            continue
        if actual != expected:
            failures.append(
                f"  MISMATCH [{field}]: replay={actual!r}, manifest={expected!r}"
            )
        else:
            print(f"  OK  {field}: {actual}")

    # reasoning_audit_report.json has metrics nested under "counters"
    audit_metrics = audit.get("counters", audit.get("metrics", {}))
    print("\nChecking reasoning_audit_report.json vs replay_report.json audit fields ...")
    for field in AUDIT_FIELDS:
        audit_val = audit_metrics.get(field)
        replay_val = replay_state.get(field)
        if audit_val is None and replay_val is None:
            continue
        if audit_val != replay_val:
            failures.append(
                f"  MISMATCH [{field}]: audit_report={audit_val!r}, replay_state={replay_val!r}"
            )
        else:
            print(f"  OK  {field}: {audit_val}")

        # Also check manifest
        manifest_val = manifest_replay.get(field)
        if audit_val != manifest_val:
            failures.append(
                f"  MISMATCH [{field}]: audit_report={audit_val!r}, manifest={manifest_val!r}"
            )

    if failures:
        print("\nFAIL: Consistency mismatches detected:")
        for f in failures:
            print(f)
        return 1

    print("\nPASS: All checked fields are consistent.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
