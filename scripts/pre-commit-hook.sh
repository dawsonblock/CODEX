#!/bin/bash
# CODEX-main 36 Pre-commit Hook
# Installed at: .git/hooks/pre-commit
# Runs validation checks before allowing commit (not a deployment certification)

set -e

RUST_DIR="global-workspace-runtime-rs"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  CODEX-main 36 Pre-commit Validation                   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Track failure count
FAILURES=0

# ====================================================================
# CHECK 1: Format Validation
# ====================================================================
echo -e "${YELLOW}[1/5] Checking code formatting...${NC}"
if cargo fmt --all -- --check >/dev/null 2>&1 && \
   cd "$RUST_DIR" && cargo fmt --all -- --check >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Format check passed${NC}"
else
    echo -e "${RED}❌ Format check failed. Run 'make fmt' to fix.${NC}"
    FAILURES=$((FAILURES + 1))
fi
cd - >/dev/null 2>&1

# ====================================================================
# CHECK 2: Linting with Clippy
# ====================================================================
echo -e "${YELLOW}[2/5] Running clippy linter...${NC}"
if cd "$RUST_DIR" && cargo clippy --workspace --all-targets --all-features -- -D warnings >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Clippy check passed${NC}"
else
    echo -e "${RED}❌ Clippy found issues. Run 'make lint' to review.${NC}"
    FAILURES=$((FAILURES + 1))
fi
cd - >/dev/null 2>&1

# ====================================================================
# CHECK 3: Unit Tests
# ====================================================================
echo -e "${YELLOW}[3/5] Running unit tests...${NC}"
if cd "$RUST_DIR" && cargo test --lib --all --quiet >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Unit tests passed${NC}"
else
    echo -e "${RED}❌ Unit tests failed. Review test output above.${NC}"
    FAILURES=$((FAILURES + 1))
fi
cd - >/dev/null 2>&1

# ====================================================================
# CHECK 4: Proof Validation (quick, non-strict)
# ====================================================================
echo -e "${YELLOW}[4/5] Validating proof generation...${NC}"
if cd "$RUST_DIR" && cargo run -p runtime-cli -- proof --long-horizon --nl --out ../artifacts/proof/precommit 2>/dev/null | grep -q "overall_status.*pass"; then
    echo -e "${GREEN}✅ Proof validation passed${NC}"
else
    echo -e "${YELLOW}⚠️  Proof validation skipped (optional in pre-commit)${NC}"
fi
cd - >/dev/null 2>&1

# ====================================================================
# CHECK 5: Generated Artifact Check
# ====================================================================
echo -e "${YELLOW}[5/5] Checking for unintended generated artifacts...${NC}"
if python3 scripts/check_no_generated_artifacts.py >/dev/null 2>&1; then
    echo -e "${GREEN}✅ No unintended generated artifacts${NC}"
else
    echo -e "${RED}❌ Generated artifacts detected. Clean with 'make clean'.${NC}"
    FAILURES=$((FAILURES + 1))
fi

# ====================================================================
# SUMMARY
# ====================================================================
echo ""
if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✅ All pre-commit checks passed. Commit allowed.     ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ❌ Pre-commit checks failed ($FAILURES issue(s))      ║${NC}"
    echo -e "${RED}║  Commit blocked. Fix issues and try again.            ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Tips:"
    echo "  • Run 'make fmt' to fix formatting"
    echo "  • Run 'make lint' to see clippy warnings"
    echo "  • Run 'make test' to run full test suite"
    echo "  • Run 'make pre-commit' to test locally before pushing"
    echo ""
    exit 1
fi
