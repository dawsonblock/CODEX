# PATCH_NOTES_CODEX_MAIN_22_CONSISTENCY_CLEANUP.md

## CODEX-main 22 → CODEX-main 36 Hardening Candidate: Consistency Cleanup

### Package identity

- Package label: **CODEX-main 36 hardening candidate**
- Current package SHA-256: `582c25e54b6219e17f0a7a2af049e7f10ef9a7aa681e5f7b79f86f51740d4f33`
- Previous package SHA-256 (CODEX-main 19, historical): `44d56855d242ced21286841ce1f42b65b8924794f486b699ec32d73f8123ddca`

### What was fixed in this cleanup

1. **Regenerated Rust verification logs** (Phase 1)
   - `rust_test.log` now reflects current source tree: 289 passed, 0 failed, 0 ignored
   - Includes 5 new runtime-core tests not previously in the log:
     - `runtime_loop::tests::spoofed_override_prompt_refuses`
     - `runtime_loop::tests::root_credentials_prompt_refuses`
     - `runtime_loop::tests::internal_diagnostic_prompt_without_mode_uses_safe_memory_lookup`
     - `runtime_loop::tests::internal_diagnostic_available_in_explicit_mode`
     - `runtime_loop::tests::contradiction_prompt_prefers_clarification`
   - `rust_fmt.log`, `rust_clippy.log`, `rust_toolchain.log`, `rust_strict_proof.log` regenerated
   - Toolchain: rustc 1.95.0, cargo 1.95.0

2. **Regenerated proof artifacts** (Phase 2)
   - `artifacts/proof/current/` regenerated via `cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current`
   - NL benchmark confirmed: curated 15, held_out 59, adversarial 2; all action_match_rate 1.0, 0 failures

3. **Updated stale package SHA references** (Phase 3)
   - Replaced old SHA `44d56855...` with `582c25e5...` in:
     - `VALIDATION_READINESS.md`
     - `artifacts/proof/verification/FINAL_VERIFICATION_REPORT.md`
     - `docs/HARDENING_REPORT_CODEX_MAIN_36.md`

4. **Updated stale held-out benchmark text** (Phase 4)
   - Removed references to `0.9152542372881356`, `5 failures`, `0.8983050847457628`, `6 failures`
   - Updated files:
     - `artifacts/proof/current/ALL_PHASES_COMPLETE_FINAL_REPORT.md`
     - `artifacts/proof/current/CODEX_MAIN_36_HARDENING_COMPLETE.md`
     - `artifacts/proof/current/FINAL_COMPLETION_SUMMARY.md`
     - `artifacts/proof/current/SESSION_FINAL_REPORT.md`
     - `docs/REPO_INVENTORY.md`
     - `PHASE_15_IMPLEMENTATION_COMPLETE.md`
     - `STATUS.md`, `CURRENT_PROOF_SUMMARY.md`

5. **Clarified artifacts/proof/test as fixture** (Phase 5)
   - Added `artifacts/proof/test/README.md` marking directory as non-authoritative fixture data

6. **Extended proof consistency checker** (Phase 6)
   - `scripts/check_proof_manifest_consistency.py` now detects:
     - Stale SHA `44d56855...` in current-status docs (Phase J)
     - Stale held-out rates `0.9152542372881356` / `0.8983050847457628` (Phase K)
     - Stale failure phrases in current-status docs (Phase K)
     - Missing fixture README in `artifacts/proof/test/` (Phase L)

7. **Updated Rust test counts** across docs to 289

### Current NL benchmark (authoritative)

- curated: 15 scenarios, action_match_rate: 1.0, failures: 0
- held_out: 59 scenarios, action_match_rate: 1.0, failures: 0
- adversarial: 2 scenarios, action_match_rate: 1.0, failures: 0
- total diagnostic scenarios: 76

**This is a bounded diagnostic benchmark over known scenario classes. It does not prove broad natural-language intelligence or general safety.**

### Validation results after cleanup

- `python3 scripts/check_no_generated_artifacts.py`: pass
- `python3 -m pytest -q`: 35 passed
- `python3 architecture_guard.py`: pass
- `python3 scripts/check_proof_manifest_consistency.py`: pass (including new Phase J/K/L checks)
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_action_types`: pass
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_sentience_claims`: pass
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_no_mv2 .`: pass
- `PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_resource_recovery`: pass
- `cargo fmt --check`: pass
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: pass
- `cargo test --workspace --all-targets --all-features`: 289 passed, 0 failed, 0 ignored

### Required honesty statement

This is a bounded research scaffold. It is not AGI, not sentient, not autonomous, not production-ready, not deployment-ready, not release-ready, and not fully verified. Provider execution remains disabled by default. Real external tool execution remains disabled or dry-run by default. Operational deployment requires separate engineering, security, legal, and safety review.
