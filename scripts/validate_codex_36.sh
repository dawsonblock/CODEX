#!/bin/bash

# CODEX-main 36 Validation Suite (review scope)
set -euo pipefail

echo "=========================================="
echo "CODEX-main 36 Validation Suite"
echo "=========================================="
echo ""

GREEN=$'\033[0;32m'
RED=$'\033[0;31m'
YELLOW=$'\033[0;33m'
NC=$'\033[0m'

if ! CODEX_ROOT="$(cd "$(dirname "$0")/.." && pwd)"; then
    echo "❌ Unable to resolve CODEX_ROOT from script location."
    exit 1
fi
RUNTIME_ROOT="$CODEX_ROOT/global-workspace-runtime-rs"

FAILED=0

echo "[1/7] Running proof-manifest consistency check..."
if (cd "$CODEX_ROOT" && python3 scripts/check_proof_manifest_consistency.py >/dev/null 2>&1); then
    echo "✅ Proof-manifest consistency check passed"
else
    echo "❌ Proof-manifest consistency check failed"
    FAILED=$((FAILED + 1))
fi

echo "[2/7] Running claim guard..."
if (cd "$CODEX_ROOT" && PYTHONPATH=src python3 -m global_workspace_runtime.scripts.check_sentience_claims >/dev/null 2>&1); then
    echo "✅ Claim guard passed"
else
    echo "❌ Claim guard failed"
    FAILED=$((FAILED + 1))
fi

echo "[3/7] Checking Rust workspace test command availability..."
if command -v cargo >/dev/null 2>&1; then
    echo "✅ cargo available (run full Rust tests separately for fresh verification)"
else
    echo "⚠️  cargo unavailable in this environment"
fi

echo "[4/7] Checking packaged default UI log registration..."
if grep -q "artifacts/proof/verification/ui_tests.log" "$CODEX_ROOT/artifacts/proof/verification/proof_manifest.json"; then
    echo "✅ Packaged default UI log is registered"
else
    echo "❌ Packaged default UI log registration missing"
    FAILED=$((FAILED + 1))
fi

echo "[5/7] Checking packaged provider-feature UI log registration..."
if grep -q "artifacts/proof/verification/ui_provider_feature_tests.log" "$CODEX_ROOT/artifacts/proof/verification/proof_manifest.json"; then
    echo "✅ Packaged provider-feature UI log is registered"
else
    echo "❌ Packaged provider-feature UI log registration missing"
    FAILED=$((FAILED + 1))
fi

echo "[6/7] Scope note for provider-feature UI tests..."
echo "ℹ️  This script does not execute: cargo test --all-targets --features ui-local-providers"
echo "ℹ️  Provider-feature status here is based on packaged logs unless rerun manually."

echo "[7/7] Generated-artifact check..."
if (cd "$CODEX_ROOT" && python3 scripts/check_no_generated_artifacts.py >/dev/null 2>&1); then
    echo "✅ Generated-artifact check passed"
else
    echo "❌ Generated-artifact check failed"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "=========================================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ VALIDATION CHECKS PASSED${NC}"
    echo "Ready for review and controlled test integration."
    echo "This is not a production certification or deployment authorization."
    exit 0
else
    echo -e "${RED}❌ ${FAILED} validation check(s) failed${NC}"
    exit 1
fi
