# Retrieval Policy Status (Current)

## Current behavior

Retrieval intent routing and policy inspection exist. The current package does **not** claim complete blocking enforcement in all retrieval paths.

- `governance_only` is advisory/inspection-only in ClaimStore compatibility paths.
- Some provider-facing paths enforce policy gates.
- Custom full-blocking rule enforcement is not complete across all retrieval call paths.

## Evidence reference

- `artifacts/proof/current/retrieval_policy_enforcement_report.json`
  - `status: advisory_inspection_only`
  - `enforcement_level: no_blocking_enforcement`

## Claim boundary

This document does not claim full enforcement for rejected/stale/disputed/superseded/archived filtering unless explicitly proven by code+tests.
