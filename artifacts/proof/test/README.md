# artifacts/proof/test — Non-Authoritative Test Fixtures

This directory contains **non-authoritative test fixtures**.

It is **not current proof**. Current authoritative proof lives in:

    artifacts/proof/current/

The files here preserve older NL benchmark snapshots used as reference fixtures during development. They show stale values (e.g., `held_out action_match_rate: 0.9152542372881356`) that reflect pre-improvement state and are intentionally retained as historical comparison data.

**Do not use these files as evidence of current system performance.**

```json
{
  "fixture": true,
  "authoritative_current_proof": false,
  "current_proof_directory": "artifacts/proof/current/"
}
```
