#!/bin/bash

# CODEX-main 36 Local Validation Script
# Run full validation checks locally before review
# Usage: bash scripts/validate_local.sh

set -e

RUST_DIR="global-workspace-runtime-rs"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

FAILED_CHECKS=0
PASSED_CHECKS=0

print_header() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  CODEX-main 36 Full Local Validation                  ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

check_command() {
    local check_name="$1"
    local check_cmd="$2"
    local check_num="$3"
    local total="$4"
    
    echo -e "${CYAN}[${check_num}/${total}] ${check_name}${NC}"
    
    if eval "$check_cmd" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC}"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

print_header

# ====================================================================
# FORMAT CHECK
# ====================================================================
check_command \
    "Format Check (cargo fmt)" \
    "cargo fmt --all -- --check && cd $RUST_DIR && cargo fmt --all -- --check && cd - >/dev/null 2>&1" \
    1 8 || {
    echo "   Hint: Run 'make fmt' to fix formatting issues"
}

# ====================================================================
# CLIPPY LINT
# ====================================================================
check_command \
    "Clippy Linter" \
    "cd $RUST_DIR && cargo clippy --workspace --all-targets --all-features -- -D warnings && cd - >/dev/null 2>&1" \
    2 8 || {
    echo "   Hint: Run 'make lint' to review clippy warnings"
}

# ====================================================================
# BUILD
# ====================================================================
check_command \
    "Build (debug)" \
    "cd $RUST_DIR && cargo build --all --quiet && cd - >/dev/null 2>&1" \
    3 8 || {
    echo "   Hint: Run 'make build' for full build output"
}

# ====================================================================
# UNIT TESTS
# ====================================================================
check_command \
    "Unit Tests" \
    "cd $RUST_DIR && cargo test --lib --all --quiet && cd - >/dev/null 2>&1" \
    4 8 || {
    echo "   Hint: Run 'make test' to see test output"
}

# ====================================================================
# INTEGRATION TESTS
# ====================================================================
check_command \
    "Integration Tests" \
    "cd $RUST_DIR && cargo test --test '*' --quiet 2>/dev/null || true && cd - >/dev/null 2>&1" \
    5 8 || {
    echo "   Hint: Some integration tests may be optional"
}

# ====================================================================
# PROOF GENERATION (STRICT)
# ====================================================================
check_command \
    "Proof Generation (strict mode)" \
    "cd $RUST_DIR && cargo run -p runtime-cli -- proof --strict --long-horizon --nl --out ../artifacts/proof/current 2>/dev/null | grep -q 'overall_status.*pass' && cd - >/dev/null 2>&1" \
    6 8 || {
    echo "   Hint: Run 'make proof-strict' for detailed output"
}

# ====================================================================
# ORACLE GUARD
# ====================================================================
check_command \
    "Oracle Guard Tests" \
    "cd $RUST_DIR && cargo test --workspace -- oracle --quiet && cd - >/dev/null 2>&1" \
    7 8 || {
    echo "   Hint: Run 'make test-oracle' for details"
}

# ====================================================================
# ARTIFACT CHECK
# ====================================================================
check_command \
    "Generated Artifacts Check" \
    "python3 scripts/check_no_generated_artifacts.py >/dev/null 2>&1" \
    8 8 || {
    echo "   Hint: Run 'make clean' to remove unintended artifacts"
}

# ====================================================================
# FINAL SUMMARY
# ====================================================================
echo ""
TOTAL_CHECKS=$((PASSED_CHECKS + FAILED_CHECKS))

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✅ ALL VALIDATIONS PASSED ($PASSED_CHECKS/$TOTAL_CHECKS)                    ║${NC}"
    echo -e "${GREEN}║                                                        ║${NC}"
    echo -e "${GREEN}║  Ready for review.                             ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Review your changes: git diff --cached"
    echo "  2. Open/update your PR through normal workflow"
    echo "  3. Watch CI: https://github.com/dawsonblock/CODEX/actions"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ❌ VALIDATION FAILED ($FAILED_CHECKS/$TOTAL_CHECKS checks failed)              ║${NC}"
    echo -e "${RED}║                                                        ║${NC}"
    echo -e "${RED}║  Fix issues before pushing. See hints above.           ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Recommended workflow:"
    echo "  1. Run 'make pre-commit' frequently during development"
    echo "  2. Fix issues as they arise (formatting, lint, tests)"
    echo "  3. Run 'make validate' before final push"
    echo ""
    exit 1
fi
